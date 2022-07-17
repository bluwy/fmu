use fmu::{get_js_syntax, JsSyntax};
use hyper::{body::to_bytes, Body, Client, Uri};
use hyper_tls::HttpsConnector;
use std::{fs, str::FromStr};

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
    assert_eq!(
        get_js_syntax(&rs("cjs_create_require_scope")),
        JsSyntax::CJS
    );
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

#[tokio::test]
async fn npm_esm_svelte() -> Result<(), Box<dyn std::error::Error>> {
    let res = fetch_unpkg(
        "esm_svelte",
        "https://unpkg.com/svelte@3.49.0/internal/index.mjs",
    )
    .await?;
    assert_eq!(get_js_syntax(&res), JsSyntax::ESM);
    Ok(())
}

#[tokio::test]
async fn npm_esm_vite() -> Result<(), Box<dyn std::error::Error>> {
    let res = fetch_unpkg(
        "esm_vite",
        "https://unpkg.com/vite@3.0.0/dist/node/chunks/dep-07a79996.js",
    )
    .await?;
    assert_eq!(get_js_syntax(&res), JsSyntax::ESM);
    Ok(())
}

#[tokio::test]
async fn npm_esm_vue() -> Result<(), Box<dyn std::error::Error>> {
    let res = fetch_unpkg(
        "esm_vue",
        "https://unpkg.com/vue@3.2.37/dist/vue.esm-browser.prod.js",
    )
    .await?;
    assert_eq!(get_js_syntax(&res), JsSyntax::ESM);
    Ok(())
}

#[tokio::test]
async fn npm_cjs_svelte() -> Result<(), Box<dyn std::error::Error>> {
    let res = fetch_unpkg(
        "cjs_svelte",
        "https://unpkg.com/svelte@3.49.0/internal/index.js",
    )
    .await?;
    assert_eq!(get_js_syntax(&res), JsSyntax::CJS);
    Ok(())
}

#[tokio::test]
async fn npm_cjs_vue() -> Result<(), Box<dyn std::error::Error>> {
    let res = fetch_unpkg(
        "cjs_vue",
        "https://unpkg.com/vue@3.2.37/dist/vue.cjs.prod.js",
    )
    .await?;
    assert_eq!(get_js_syntax(&res), JsSyntax::CJS);
    Ok(())
}

// read sample. shorten so assertions are all single-line.
fn rs(name: &str) -> String {
    let s = match fs::read_to_string(format!("tests/samples/{}.js", name)) {
        Err(err) => panic!("Couldn't open file: {}", err),
        Ok(value) => value,
    };
    s
}

async fn fetch_unpkg(name: &str, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let cache_file_path = format!("tests/samples/npm/{}.js", name);
    let result = match fs::read_to_string(&cache_file_path) {
        Err(_) => {
            // if fail to read, assume no exist. fetch and save to cache
            // TODO: skip if have no permissions instead for some reason
            let https = HttpsConnector::new();
            let client = Client::builder().build::<_, Body>(https);
            let resp = client.get(Uri::from_str(&url)?).await?;
            let body_bytes = to_bytes(resp.into_body()).await?;
            let content = String::from_utf8(body_bytes.to_vec()).unwrap();
            fs::create_dir("tests/samples/npm").ok();
            match fs::write(&cache_file_path, &content) {
                Err(err) => panic!("Couldn't write to {}: {}", cache_file_path, err),
                Ok(_) => (),
            }
            content
        }
        Ok(value) => value,
    };
    Ok(result)
}
