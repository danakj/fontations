//! Extraction of glyph outlines.
//!
//! Scaling is the process of decoding an outline, applying variation deltas,
//! and executing [hinting](https://en.wikipedia.org/wiki/Font_hinting) instructions
//! for a glyph of a particular size.
//!
//! The process starts with construction of a [`Context`] that can be used to build
//! and configure a [`Scaler`] which can generate scaled paths for glyphs.
//!
//! Read on for more detail.
//!
//! ## Building a scaler
//!
//! Scaling requires temporary memory allocations and, in the case of hinting,
//! also benefits from caching the state of the hinting setup programs. To amortize the
//! cost of heap allocations and support caching, the [`Context`] type is provided.
//! This type is opaque and offers a single method called [`new_scaler`](Context::new_scaler)
//! that produces a [`ScalerBuilder`] to configure and builder a [`Scaler`].
//!
//! Assuming you have some `font` (any type that implements [`TableProvider`](read_fonts::TableProvider)),
//! this will build a scaler for a size of 16px:
//!
//! ```
//! # use skrifa::{scale::*, Size};
//! # fn build_scaler(font: read_fonts::FontRef) {
//! let mut context = Context::new();
//! let mut scaler = context.new_scaler()
//!     .size(Size::new(16.0))
//!     .build(&font);
//! # }
//! ```
//!
//! For variable fonts, the [`variation_settings`](ScalerBuilder::variation_settings) method can
//! be used to specify user coordinates for selecting an instance:
//!
//! ```
//! # use skrifa::{scale::*, Size};
//! # fn build_scaler(font: read_fonts::FontRef) {
//! let mut context = Context::new();
//! let mut scaler = context.new_scaler()
//!     .size(Size::new(16.0))
//!     .variation_settings(&[("wght", 720.0), ("wdth", 75.0)])
//!     .build(&font);
//! # }
//! ```
//!
//! If you already have coordinates in normalized design space, you can specify those directly
//! with the [`normalized_coords`](ScalerBuilder::normalized_coords) method.
//!
//! See the [`ScalerBuilder`] type for all available configuration options.
//!
//! ## Getting an outline
//!
//! Once we have a configured scaler, extracting an outline is fairly simple. The
//! [`outline`](Scaler::outline) method on scaler uses a callback approach where the user
//! provides an implementation of the [`Pen`] trait and the appropriate methods are invoked for
//! each resulting path element of the scaled outline.
//!
//! Assuming we constructed a scaler as above, let's load a glyph and convert it into an SVG path:
//!
//! ```
//! # use skrifa::{scale::*, GlyphId, Size};
//! # fn build_scaler(font: read_fonts::FontRef) {
//! # let mut context = Context::new();
//! # let mut scaler = context.new_scaler()
//! #    .size(Size::new(16.0))
//! #    .build(&font);
//! // Create a type for holding our SVG path.
//! #[derive(Default)]
//! struct SvgPath(String);
//!
//! // Implement the Pen trait for this type. This emits the appropriate
//! // SVG path commands for each element type.
//! impl Pen for SvgPath {
//!     fn move_to(&mut self, x: f32, y: f32) {
//!         self.0.push_str(&format!("M{x:.1},{y:.1} "));
//!     }
//!
//!     fn line_to(&mut self, x: f32, y: f32) {
//!         self.0.push_str(&format!("L{x:.1},{y:.1} "));
//!     }
//!
//!     fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
//!         self.0
//!             .push_str(&format!("Q{cx0:.1},{cy0:.1} {x:.1},{y:.1} "));
//!     }
//!
//!     fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
//!         self.0.push_str(&format!(
//!             "C{cx0:.1},{cy0:.1} {cx1:.1},{cy1:.1} {x:.1},{y:.1} "
//!         ));
//!     }
//!
//!     fn close(&mut self) {
//!         self.0.push_str("z ");
//!     }
//! }
//!
//! let mut path = SvgPath::default();
//!
//! // Scale an outline for glyph 20 and invoke the appropriate methods
//! // to build an SVG path.
//! scaler.outline(GlyphId::new(20), &mut path);
//!
//! // Print our pretty new path.
//! println!("{}", path.0);
//! # }
//! ```
//!
//! The pen based interface is designed to be flexible. Output can be sent directly to a
//! software rasterizer for scan conversion, converted to an owned path representation (such as
//! a kurbo [`BezPath`](https://docs.rs/kurbo/latest/kurbo/struct.BezPath.html)) for further
//! analysis and transformation, or fed into other crates like [vello](https://github.com/linebender/vello),
//! [lyon](https://github.com/nical/lyon) or [pathfinder](https://github.com/servo/pathfinder)
//! for GPU rendering.

mod error;
mod scaler;

#[cfg(test)]
mod test;

// This will go away in the future when we add tracing support. Hide it
// for now.
#[doc(hidden)]
pub mod glyf;

pub use read_fonts::types::Pen;

pub use error::{Error, Result};
pub use scaler::{Scaler, ScalerBuilder};

use super::{GlyphId, NormalizedCoord, UniqueId, VariationSetting};
use core::str::FromStr;
use read_fonts::types::Tag;

/// Limit for recursion when loading TrueType composite glyphs.
const GLYF_COMPOSITE_RECURSION_LIMIT: usize = 32;

/// Modes for hinting.
///
/// Only the `glyf` source supports all hinting modes.
#[cfg(feature = "hinting")]
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub enum Hinting {
    /// "Full" hinting mode. May generate rough outlines and poor horizontal
    /// spacing.
    Full,
    /// Light hinting mode. This prevents most movement in the horizontal direction
    /// with the exception of a per-font backward compatibility opt in.
    Light,
    /// Same as light, but with additional support for RGB subpixel rendering.
    LightSubpixel,
    /// Same as light subpixel, but always prevents adjustment in the horizontal
    /// direction. This is the default mode.
    #[default]
    VerticalSubpixel,
}

/// Context for scaling glyphs.
///
/// This type contains temporary memory buffers and various internal caches to
/// accelerate the glyph scaling process. You'll generally want to keep one
/// (or more, in the multi-threaded case) of these around and reuse them for
/// scaling batches of glyphs.
#[derive(Clone, Default, Debug)]
pub struct Context {
    /// Inner context for loading TrueType outlines.
    glyf: glyf::Context,
    /// Internal storage for TrueType outlines.
    glyf_outline: glyf::Outline,
    /// Storage for normalized variation coordinates.
    coords: Vec<NormalizedCoord>,
    /// Storage for variation settings.
    variations: Vec<VariationSetting>,
}

impl Context {
    /// Creates a new glyph scaling context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a builder for configuring a glyph scaler.
    pub fn new_scaler(&mut self) -> ScalerBuilder {
        ScalerBuilder::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{test, Context, GlyphId, Pen, Scaler};
    use crate::Size;
    use read_fonts::{test_data::test_fonts, FontRef};

    #[test]
    fn vazirmatin_var() {
        let font = FontRef::new(test_fonts::VAZIRMATN_VAR).unwrap();
        let outlines = test::parse_glyph_outlines(test_fonts::VAZIRMATN_VAR_GLYPHS);
        let mut cx = Context::new();
        let mut path = test::Path::default();
        for expected_outline in &outlines {
            path.0.clear();
            let mut scaler = cx
                .new_scaler()
                .size(Size::new(expected_outline.size))
                .normalized_coords(&expected_outline.coords)
                .build(&font);
            scaler
                .outline(expected_outline.glyph_id, &mut path)
                .unwrap();
            if path.0 != expected_outline.path {
                panic!(
                    "mismatch in glyph path for id {} (size: {}, coords: {:?}): path: {:?} expected_path: {:?}",
                    expected_outline.glyph_id,
                    expected_outline.size,
                    expected_outline.coords,
                    &path.0,
                    &expected_outline.path
                );
            }
        }
    }
}
