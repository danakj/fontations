use super::*;
use crate::assert_hex_eq;
use font_test_data::gsub as test_data;

#[test]
fn singlesubstformat1() {
    // https://learn.microsoft.com/en-us/typography/opentype/spec/gsub#example-2-singlesubstformat1-subtable
    let table = SingleSubstFormat1::read(test_data::SINGLESUBSTFORMAT1_TABLE.into()).unwrap();
    let dumped = crate::write::dump_table(&table).unwrap();
    assert_hex_eq!(test_data::SINGLESUBSTFORMAT1_TABLE, &dumped);
}

#[test]
fn singlesubstformat2() {
    // https://learn.microsoft.com/en-us/typography/opentype/spec/gsub#example-3-singlesubstformat2-subtable
    let table = SingleSubstFormat2::read(test_data::SINGLESUBSTFORMAT2_TABLE.into()).unwrap();
    let dumped = crate::write::dump_table(&table).unwrap();
    assert_hex_eq!(test_data::SINGLESUBSTFORMAT2_TABLE, &dumped);
}

#[test]
fn multiplesubstformat1() {
    // https://learn.microsoft.com/en-us/typography/opentype/spec/gsub#example-4-multiplesubstformat1-subtable
    let table = MultipleSubstFormat1::read(test_data::MULTIPLESUBSTFORMAT1_TABLE.into()).unwrap();
    let dumped = crate::write::dump_table(&table).unwrap();
    assert_hex_eq!(test_data::MULTIPLESUBSTFORMAT1_TABLE, &dumped);
}

#[test]
fn alternatesubstformat1() {
    // https://learn.microsoft.com/en-us/typography/opentype/spec/gsub#example-5-alternatesubstformat-1-subtable
    let table = AlternateSubstFormat1::read(test_data::ALTERNATESUBSTFORMAT1_TABLE.into()).unwrap();
    let dumped = crate::write::dump_table(&table).unwrap();
    assert_hex_eq!(test_data::ALTERNATESUBSTFORMAT1_TABLE, &dumped);
}

#[test]
fn ligaturesubstformat1() {
    // https://learn.microsoft.com/en-us/typography/opentype/spec/gsub#example-6-ligaturesubstformat1-subtable
    let table = LigatureSubstFormat1::read(test_data::LIGATURESUBSTFORMAT1_TABLE.into()).unwrap();
    let dumped = crate::write::dump_table(&table).unwrap();
    assert_hex_eq!(test_data::LIGATURESUBSTFORMAT1_TABLE, &dumped);
}

#[cfg(feature = "serde")]
#[test]
fn test_serde() {
    use crate::tables::layout::*;
    let gsub = Gsub::new(
        ScriptList::new(vec![ScriptRecord::new(
            Tag::new(b"Piqd"),
            Script::new(
                None,
                vec![LangSysRecord::new(Tag::new(b"KLI"), LangSys::new(vec![0]))],
            ),
        )]),
        FeatureList::new(vec![FeatureRecord::new(
            Tag::new(b"kern"),
            Feature::new(None, vec![0]),
        )]),
        SubstitutionLookupList::new(vec![SubstitutionLookup::Single(Lookup::new(
            LookupFlag::empty(),
            vec![SingleSubst::Format1(SingleSubstFormat1::new(
                CoverageTable::format_1(vec![GlyphId::new(101)]),
                -1,
            ))],
            0,
        ))]),
    );

    let dumped = bincode::serialize(&gsub).unwrap();
    let loaded: Gsub = bincode::deserialize(&dumped).unwrap();

    assert_eq!(loaded.script_list.script_records[0].script_tag, "Piqd");
    assert_eq!(
        loaded.script_list.script_records[0].script.lang_sys_records[0].lang_sys_tag,
        "KLI "
    );
}
