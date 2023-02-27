use base64::{engine::general_purpose, Engine};
use html_escape;
use sha3::{
    digest::{ExtendableOutput, Update},
    Shake128,
};
use std::{env, include_str, io::Read};
use tiny_http::{Header, Request, Response, Server};

const MAIN_CSS: &'static str = include_str!("main.css");

fn get_real_ip(req: &Request) -> String {
    for header in req.headers() {
        if header.field.as_str() == "X-Real-Ip" {
            let ip = header.value.as_str();
            let ip = ip.split(',').next().unwrap();
            let ip = ip.trim();
            return ip.to_string();
        }
    }

    let ip = req.remote_addr().ip();
    let ip = ip.to_string();
    ip
}

fn compute_etag(data: &str) -> String {
    let mut hasher = Shake128::default();
    hasher.update(data.as_bytes());

    let mut reader = hasher.finalize_xof();
    let mut result = [0u8; 16];
    reader.read(&mut result).expect("Could compute ETag");

    format!(
        "W/\"{hash}\"",
        hash = general_purpose::STANDARD_NO_PAD.encode(result)
    )
}

fn log_response<R: std::io::Read>(request: &Request, response: &Response<R>, real_ip: &str) {
    println!(
        "{ip} {method} {url} {status} {response_size}",
        ip = real_ip,
        method = request.method().as_str(),
        url = request.url(),
        status = response.status_code().0,
        response_size = response.data_length().unwrap_or(0)
    );
}

fn main() {
    let css_etag = compute_etag(MAIN_CSS);

    let port: i32 = env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or_else(|| 8000);

    let endpoint = format!("0.0.0.0:{port}");

    let server = Server::http(&endpoint)
        .expect(format!("could not start http server @ {}", &endpoint).as_str());

    println!("server running @ {}", &endpoint);

    for request in server.incoming_requests() {
        let real_ip = get_real_ip(&request);

        match request.url() {
            "/main.css" => {
                let etag_matches = request
                    .headers()
                    .iter()
                    .find(|h| h.field.as_str().to_ascii_lowercase() == "if-none-match")
                    .map(|h| h.value == css_etag)
                    .unwrap_or(false);

                if etag_matches {
                    let response = Response::empty(304)
                        .with_header(
                            Header::from_bytes(
                                &b"Content-Type"[..],
                                &b"text/css; charset=utf-8"[..],
                            )
                            .unwrap(),
                        )
                        .with_header(
                            Header::from_bytes(&b"Cache-Control"[..], &b"max-age=86400"[..])
                                .unwrap(),
                        )
                        .with_header(Header::from_bytes(&b"ETag"[..], css_etag.as_str()).unwrap());

                    log_response(&request, &response, &real_ip);
                    request.respond(response);
                } else {
                    let response = Response::from_string(MAIN_CSS)
                        .with_header(
                            Header::from_bytes(
                                &b"Content-Type"[..],
                                &b"text/css; charset=utf-8"[..],
                            )
                            .unwrap(),
                        )
                        .with_header(
                            Header::from_bytes(&b"Cache-Control"[..], &b"max-age=86400"[..])
                                .unwrap(),
                        )
                        .with_header(Header::from_bytes(&b"ETag"[..], css_etag.as_str()).unwrap());

                    log_response(&request, &response, &real_ip);
                    request.respond(response);
                }
            }
            "/" => {
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

                    html_escape::encode_safe_to_string(
                        header.field.as_str(),
                        &mut encoded_header_field,
                    );
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
                    Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                        .unwrap(),
                );
                log_response(&request, &response, &real_ip);
                request.respond(response);
            }
            _ => {
                let response = Response::from_string("404 not found").with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"text/plain; charset=utf-8"[..])
                        .unwrap(),
                );
                log_response(&request, &response, &real_ip);
                request.respond(response);
            }
        };
    }
}
