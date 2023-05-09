//! compile-time representations of offsets

use super::write::{FontWrite, TableWriter};

/// The width in bytes of an Offset16
#[allow(dead_code)] // currently unused because of a serde bug?
                    // https://github.com/serde-rs/serde/issues/2449
pub const WIDTH_16: usize = 2;
/// The width in bytes of an Offset24
#[allow(dead_code)] // will be used one day :')
pub const WIDTH_24: usize = 3;
/// The width in bytes of an Offset32
pub const WIDTH_32: usize = 4;

/// An offset subtable.
///
/// The generic const `N` is the width of the offset, in bytes.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OffsetMarker<T, const N: usize = 2> {
    obj: T,
}

/// An offset subtable which may be null.
///
/// The generic const `N` is the width of the offset, in bytes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NullableOffsetMarker<T, const N: usize = 2> {
    obj: Option<T>,
}

impl<T, const N: usize> std::ops::Deref for OffsetMarker<T, N> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

impl<T, const N: usize> std::ops::DerefMut for OffsetMarker<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.obj
    }
}

impl<T, const N: usize> std::ops::Deref for NullableOffsetMarker<T, N> {
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

impl<T, const N: usize> std::ops::DerefMut for NullableOffsetMarker<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.obj
    }
}

impl<T, const N: usize> AsRef<T> for OffsetMarker<T, N> {
    fn as_ref(&self) -> &T {
        &self.obj
    }
}

impl<T, const N: usize> AsMut<T> for OffsetMarker<T, N> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.obj
    }
}

// NOTE: we don't impl AsRef/AsMut for NullableOffsetMarker, since it is less
// useful than the Option::as_ref and Option::as_mut methods available through deref

impl<const N: usize, T> OffsetMarker<T, N> {
    /// Create a new marker.
    pub fn new(obj: T) -> Self {
        OffsetMarker { obj }
    }

    //FIXME: maybe we get rid of this when we have From/Into impl'd?
    /// Set the contents of the marker, replacing any existing contents.
    pub fn set(&mut self, obj: T) {
        self.obj = obj;
    }

    /// Convert into the inner type
    pub fn into_inner(self) -> T {
        self.obj
    }
}

impl<const N: usize, T> NullableOffsetMarker<T, N> {
    /// Create a new marker.
    pub fn new(obj: Option<T>) -> Self {
        NullableOffsetMarker { obj }
    }

    /// Set the contents of the marker, replacing any existing contents.
    pub fn set(&mut self, obj: impl Into<Option<T>>) {
        self.obj = obj.into()
    }

    /// Convert into the inner type
    pub fn into_inner(self) -> Option<T> {
        self.obj
    }
}

impl<const N: usize, T: FontWrite> FontWrite for OffsetMarker<T, N> {
    fn write_into(&self, writer: &mut TableWriter) {
        writer.write_offset(&self.obj, N);
    }
}

impl<const N: usize, T: FontWrite> FontWrite for NullableOffsetMarker<T, N> {
    fn write_into(&self, writer: &mut TableWriter) {
        match self.obj.as_ref() {
            Some(obj) => writer.write_offset(obj, N),
            None => writer.write_slice([0u8; N].as_slice()),
        }
    }
}

impl<T, const N: usize> Default for NullableOffsetMarker<T, N> {
    fn default() -> Self {
        Self { obj: None }
    }
}

impl<T, const N: usize> From<T> for OffsetMarker<T, N> {
    fn from(src: T) -> Self {
        OffsetMarker::new(src)
    }
}

impl<T, const N: usize> From<T> for NullableOffsetMarker<T, N> {
    fn from(src: T) -> Self {
        NullableOffsetMarker::new(Some(src))
    }
}

impl<T, const N: usize> From<Option<T>> for NullableOffsetMarker<T, N> {
    fn from(src: Option<T>) -> Self {
        NullableOffsetMarker::new(src)
    }
}

// In case I still want to use these?

//impl<T: std::fmt::Debug, const N: usize> std::fmt::Debug for OffsetMarker<T, N> {
//fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//write!(f, "OffsetMarker({}, {:?})", N * 8, self.obj.as_ref(),)
//}
//}

//impl<const N: usize, T: std::fmt::Debug> std::fmt::Debug for NullableOffsetMarker<T, N> {
//fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//write!(
//f,
//"NullableOffsetMarker({}, {:?})",
//N * 8,
//self.obj.as_ref(),
//)
//}
//}
