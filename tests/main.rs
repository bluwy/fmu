use fmu::{get_js_syntax, JsSyntax};
use std::fs;

#[test]
fn esm() {
    assert_eq!(get_js_syntax(&rs("esm_default_export")), JsSyntax::ESM);
    assert_eq!(get_js_syntax(&rs("esm_named_export")), JsSyntax::ESM);
    assert_eq!(get_js_syntax(&rs("esm_top_level_import")), JsSyntax::ESM);
    assert_eq!(get_js_syntax(&rs("esm_import_meta")), JsSyntax::ESM);
    assert_eq!(get_js_syntax(&rs("esm_create_require")), JsSyntax::ESM);
    assert_eq!(get_js_syntax(&rs("esm_entice_cjs")), JsSyntax::ESM);
}

#[test]
fn cjs() {
    assert_eq!(get_js_syntax(&rs("cjs_require")), JsSyntax::CJS);
    assert_eq!(get_js_syntax(&rs("cjs_require_in_string")), JsSyntax::CJS);
    assert_eq!(get_js_syntax(&rs("cjs_entice_esm")), JsSyntax::CJS);
}

#[test]
fn mixed() {
    assert_eq!(get_js_syntax(&rs("mixed")), JsSyntax::Mixed);
}

#[test]
fn unknown() {
    assert_eq!(get_js_syntax(&rs("unknown")), JsSyntax::Unknown);
}

// read sample. shorten so assertions are all single-line.
fn rs(name: &str) -> String {
    let s = match fs::read_to_string(format!("tests/samples/{}.js", name)) {
        Err(err) => panic!("Couldn't open file: {}", err),
        Ok(value) => value,
    };
    s
}
