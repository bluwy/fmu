use fmu::{get_js_syntax, JsSyntax};
use hyper::{body::to_bytes, Body, Client, Uri};
use hyper_tls::HttpsConnector;
use std::{fs, str::FromStr};

#[tokio::test]
async fn esm_svelte() -> Result<(), Box<dyn std::error::Error>> {
    let res = fetch_unpkg(
        "esm_svelte",
        "https://unpkg.com/svelte@3.49.0/internal/index.mjs",
    )
    .await?;
    assert_eq!(get_js_syntax(&res), JsSyntax::ESM);
    Ok(())
}

#[tokio::test]
async fn esm_vite() -> Result<(), Box<dyn std::error::Error>> {
    let res = fetch_unpkg(
        "esm_vite",
        "https://unpkg.com/vite@3.0.0/dist/node/chunks/dep-07a79996.js",
    )
    .await?;
    assert_eq!(get_js_syntax(&res), JsSyntax::ESM);
    Ok(())
}

#[tokio::test]
async fn esm_vue() -> Result<(), Box<dyn std::error::Error>> {
    let res = fetch_unpkg(
        "esm_vue",
        "https://unpkg.com/vue@3.2.37/dist/vue.esm-browser.prod.js",
    )
    .await?;
    assert_eq!(get_js_syntax(&res), JsSyntax::ESM);
    Ok(())
}

#[tokio::test]
async fn cjs_svelte() -> Result<(), Box<dyn std::error::Error>> {
    let res = fetch_unpkg(
        "cjs_svelte",
        "https://unpkg.com/svelte@3.49.0/internal/index.js",
    )
    .await?;
    assert_eq!(get_js_syntax(&res), JsSyntax::CJS);
    Ok(())
}

#[tokio::test]
async fn cjs_vue() -> Result<(), Box<dyn std::error::Error>> {
    let res = fetch_unpkg(
        "cjs_vue",
        "https://unpkg.com/vue@3.2.37/dist/vue.cjs.prod.js",
    )
    .await?;
    assert_eq!(get_js_syntax(&res), JsSyntax::CJS);
    Ok(())
}

async fn fetch_unpkg(name: &str, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let cache_file_path = format!("tests/integration/samples/{}.js", name);
    let result = match fs::read_to_string(&cache_file_path) {
        Err(_) => {
            // if fail to read, assume no exist. fetch and save to cache
            // TODO: skip if have no permissions instead for some reason
            let https = HttpsConnector::new();
            let client = Client::builder().build::<_, Body>(https);
            let resp = client.get(Uri::from_str(&url)?).await?;
            let body_bytes = to_bytes(resp.into_body()).await?;
            let content = String::from_utf8(body_bytes.to_vec()).unwrap();
            fs::create_dir("tests/integration/samples").ok();
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
