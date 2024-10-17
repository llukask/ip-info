use axum::{
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use base64::{engine::general_purpose, Engine};
use lazy_static::lazy_static;
use sha3::{
    digest::{ExtendableOutput, Update},
    Shake128,
};
use std::{include_str, io::Read};

const MAIN_CSS: &str = include_str!("main.css");

lazy_static! {
    static ref CSS_ETAG: String = compute_etag(MAIN_CSS);
}

fn compute_etag(data: &str) -> String {
    let mut hasher = Shake128::default();
    hasher.update(data.as_bytes());

    let mut reader = hasher.finalize_xof();
    let mut result = [0u8; 16];
    reader
        .read_exact(&mut result)
        .expect("could not compute ETag");

    format!(
        "W/\"{hash}\"",
        hash = general_purpose::STANDARD_NO_PAD.encode(result)
    )
}

pub async fn axum_handle_css(headers: HeaderMap) -> impl IntoResponse {
    let etag_matches = headers
        .get("if-none-match")
        .map(|v| v == CSS_ETAG.as_str())
        .unwrap_or(false);

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "text/css; charset=utf-8".parse().unwrap());
    headers.insert("Cache-Control", "max-age=86400".parse().unwrap());
    headers.insert("ETag", CSS_ETAG.as_str().parse().unwrap());

    if etag_matches {
        (StatusCode::NOT_MODIFIED, headers).into_response()
    } else {
        (headers, MAIN_CSS).into_response()
    }
}
