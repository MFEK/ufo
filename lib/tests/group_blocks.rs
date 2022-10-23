mod common;

#[test]
fn test() {
    use libmfekufo::{glyphs, blocks};
    common::init();

    let gvec = glyphs::for_ufo("../test_data/KJV1611.ufo".to_string());
    let unique_encodings = glyphs::to_unique_codepoints(&gvec);
    let blocks = blocks::for_unicode_data(&unique_encodings);
    let _grouped_by = blocks::grouped_by(&gvec, &blocks);
}

#[test]
#[should_panic]
fn should_panic() {
    use libmfekufo::glyphs;
    common::init();

    glyphs::for_ufo("nonexistent.ufo".to_string());
}
