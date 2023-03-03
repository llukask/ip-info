use std::collections::BTreeMap;
use serde::Serialize;
use tiny_http::{Header, Request, Response};

use crate::common::{get_real_ip, send_response};

fn used_headers(request: &Request) -> BTreeMap<String, String> {
    let mut headers = BTreeMap::new();
    for header in request.headers() {
        let header_field = header.field.to_string();

        if header_field.starts_with("X-Forwarded-") || header_field == "X-Real-Ip" {
            continue;
        } 

        headers.insert(header_field, header.value.to_string());
    }
    headers
}

pub(crate) fn handle_index_html<Ctx>(_: &Ctx, request: Request) {
    let real_ip = get_real_ip(&request);

    let mut response_body = String::new();
    response_body.push_str(
        format!(
            r#"
<html>
    <head>
        <title>ip stats</title>
        <link rel="stylesheet" href="main.css">
        <link rel="shortcut icon" href="data:image/x-icon;," type="image/x-icon"> 
    </head>
    <body>
        <header>
            <h1>your ip is:</h1>
            <code>{ip}</code>
        </header>
        <main>
        "#,
            ip = &real_ip
        )
        .as_str(),
    );
    for (header_field, header_value) in used_headers(&request) {
        let mut encoded_header_field = String::new();
        let mut encoded_header_value = String::new();

        html_escape::encode_safe_to_string(header_field, &mut encoded_header_field);
        html_escape::encode_safe_to_string(header_value, &mut encoded_header_value);

        response_body.push_str(
            format!(
                r#"        
            <div class="header-container">
                <code>[{}]</code>
                <code>{}</code>
            </div>
                "#,
                &encoded_header_field, &encoded_header_value
            )
            .as_str(),
        );
    }
    response_body.push_str(
        r#"
        </main>
    </body>
</html>
        "#,
    );

    let response = Response::from_string(response_body).with_header(
        Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap(),
    );
    send_response(request, response)
}

pub(crate) fn handle_index_plain<Ctx>(_: &Ctx, request: Request) {
    let real_ip = get_real_ip(&request);

    let response = Response::from_string(format!("{real_ip}\n")).with_header(
        Header::from_bytes(&b"Content-Type"[..], &b"text/plain; charset=utf-8"[..]).unwrap(),
    );

    send_response(request, response)
}

#[derive(Debug, Serialize)]
struct IpResponse {
    ip: String,
    headers: std::collections::BTreeMap<String, String>,
}

pub(crate) fn handle_index_json<Ctx>(_: &Ctx, request: Request) {
    let real_ip = get_real_ip(&request);

    let headers = used_headers(&request);

    let response_body = IpResponse {
        ip: real_ip,
        headers,
    };

    let response = Response::from_string(serde_json::to_string(&response_body).unwrap())
        .with_header(
            Header::from_bytes(
                &b"Content-Type"[..],
                &b"application/json; charset=utf-8"[..],
            )
            .unwrap(),
        );

    send_response(request, response)
}
