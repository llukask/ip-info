use base64::{engine::general_purpose, Engine};
use lazy_static::lazy_static;
use sha3::{
    digest::{ExtendableOutput, Update},
    Shake128,
};
use std::{include_str, io::Read};
use tiny_http::{Header, Request, Response};

use crate::common::send_response;

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

pub fn handle_css<Ctx>(_: &Ctx, request: Request) {
    let etag_matches = request
        .headers()
        .iter()
        .find(|h| h.field.as_str().to_ascii_lowercase() == "if-none-match")
        .map(|h| h.value == CSS_ETAG.as_str())
        .unwrap_or(false);

    if etag_matches {
        let response = Response::empty(304)
            .with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"text/css; charset=utf-8"[..]).unwrap(),
            )
            .with_header(Header::from_bytes(&b"Cache-Control"[..], &b"max-age=86400"[..]).unwrap())
            .with_header(Header::from_bytes(&b"ETag"[..], CSS_ETAG.as_str()).unwrap());

        send_response(request, response)
    } else {
        let response = Response::from_string(MAIN_CSS)
            .with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"text/css; charset=utf-8"[..]).unwrap(),
            )
            .with_header(Header::from_bytes(&b"Cache-Control"[..], &b"max-age=86400"[..]).unwrap())
            .with_header(Header::from_bytes(&b"ETag"[..], CSS_ETAG.as_str()).unwrap());

        send_response(request, response)
    }
}
