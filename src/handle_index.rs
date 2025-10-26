use askama::Template;
use axum::http::StatusCode;
use axum::{extract::ConnectInfo, http::HeaderMap, response::IntoResponse, Json};
use serde::Serialize;
use std::{collections::BTreeMap, net::SocketAddr};

use crate::content_negotiation::{parse_accept, MediaType};

#[derive(Debug, Serialize, Template)]
#[template(path = "index.html")]
pub struct IpResponse {
    pub ip: String,
    pub headers: std::collections::BTreeMap<String, String>,
}

pub async fn handle_index(
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let accept_header = headers
        .get("Accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("*/*");
    let directives = parse_accept(accept_header);

    let json_mt: MediaType = "application/json".try_into().unwrap();
    let html_mt: MediaType = "text/html".try_into().unwrap();
    let plain_mt: MediaType = "text/plain".try_into().unwrap();

    let ip = real_ip(&headers, addr.ip());

    for d in directives {
        if plain_mt.matches(&d.media_type) {
            return handle_index_plain(ip).into_response();
        } else if html_mt.matches(&d.media_type) {
            return handle_index_html(headers, ip).into_response();
        } else if json_mt.matches(&d.media_type) {
            return handle_index_json(headers, ip).into_response();
        }
    }

    handle_index_plain(ip).into_response()
}

pub fn handle_index_plain(ip: String) -> impl IntoResponse {
    format!("{}\n", ip).into_response()
}

fn handle_index_html(headers: HeaderMap, ip: String) -> impl IntoResponse {
    let template = IpResponse {
        ip,
        headers: used_headers_axum(&headers),
    };

    match template.render() {
        Ok(html) => {
            let mut response_headers = HeaderMap::new();
            response_headers.insert("Content-Type", "text/html; charset=utf-8".parse().unwrap());
            (response_headers, html).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Template rendering failed",
        )
            .into_response(),
    }
}

pub fn handle_index_json(headers: HeaderMap, ip: String) -> impl IntoResponse {
    let headers = used_headers_axum(&headers);
    let response_body = IpResponse { ip, headers };

    let mut response_headers = HeaderMap::new();
    response_headers.insert("Content-Type", "application/json".parse().unwrap());

    (response_headers, Json(response_body))
}

fn format_ip(ip: std::net::IpAddr) -> String {
    match ip {
        std::net::IpAddr::V4(ip) => ip.to_string(),
        std::net::IpAddr::V6(ip) => {
            // IPv4-mapped IPv6 address
            let segs = ip.segments();
            if segs[0] == 0
                && segs[1] == 0
                && segs[2] == 0
                && segs[3] == 0
                && segs[4] == 0
                && segs[5] == 0xFFFF
            {
                let v4 = std::net::Ipv4Addr::new(
                    (segs[6] >> 8) as u8,
                    (segs[6] & 0xFF) as u8,
                    (segs[7] >> 8) as u8,
                    (segs[7] & 0xFF) as u8,
                );
                v4.to_string()
            } else {
                ip.to_string()
            }
        }
    }
}

pub fn real_ip(headers: &HeaderMap, conn_ip: std::net::IpAddr) -> String {
    let real_ip_hdr = headers.get("x-real-ip").and_then(|v| v.to_str().ok());
    match real_ip_hdr {
        None => format_ip(conn_ip),
        Some(real_ip) => {
            let ip = real_ip;
            let ip = ip.split(',').next().unwrap();
            let ip = ip.trim();
            ip.to_string()
        }
    }
}

fn used_headers_axum(headers: &HeaderMap) -> BTreeMap<String, String> {
    headers
        .iter()
        .filter(|(k, _)| k.as_str() != "x-real-ip" && !k.as_str().starts_with("x-forwarded-"))
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
        .collect()
}
