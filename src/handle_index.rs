use serde::Serialize;
use tiny_http::{Header, Request, Response};

use crate::common::{get_real_ip, send_response};

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
    for header in request.headers() {
        let mut encoded_header_field = String::new();
        let mut encoded_header_value = String::new();

        html_escape::encode_safe_to_string(header.field.as_str(), &mut encoded_header_field);
        html_escape::encode_safe_to_string(&header.value, &mut encoded_header_value);

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
    headers: std::collections::HashMap<String, String>,
}

pub(crate) fn handle_index_json<Ctx>(_: &Ctx, request: Request) {
    let real_ip = get_real_ip(&request);

    let headers = request
        .headers()
        .iter()
        .map(|h| (h.field.to_string(), h.value.to_string()))
        .collect();

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
