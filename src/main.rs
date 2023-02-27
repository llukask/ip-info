use std::env;
use tiny_http::{Header, Request, Response, Server};

fn get_ip(req: &Request) -> String {
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

fn main() {
    let port: i32 = env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or_else(|| 8000);

    let endpoint = format!("0.0.0.0:{port}");

    let server = Server::http(&endpoint)
        .expect(format!("could not start http server @ {}", &endpoint).as_str());

    println!("server running @ {}", &endpoint);

    for request in server.incoming_requests() {
        match request.url() {
            "/main.css" => {
                let response = Response::from_string(
                    r#"
                *, *::before, *::after {
                    box-sizing: border-box;
                }

                * {
                    margin: 0;
                }

                html, body {
                    height: 100%;
                }

                body {
                    background-color: #292D3E;
                    color: #959DCB;
                    line-height: 1.5;
                    font-family: monospace;
                    font-size: 1.2rem;
                }

                header {
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    justify-content: center;
                    height: 100%;
                }

                h1 {
                    margin-block: 1rem;
                    text-align: center;
                    font-size: 2.5rem;
                }

                header > code {
                    text-decoration: underline;
                    font-size: 3rem;
                }

                main {
                    display: flex;
                    gap: 0.5rem;
                    flex-direction: column;
                    align-items: center;
                    padding-block: 1rem;
                }

                .header-container {
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    max-width: 90vw;
                }

                code {
                    max-width: min(80ch, 90vw);
                    text-align: center;
                    overflow-wrap: break-word;
                }
            "#,
                )
                .with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"text/css; charset=utf-8"[..])
                        .unwrap(),
                );

                #[allow(unused_must_use)]
                {
                    request.respond(response);
                }
            }
            "/" => {
                println!(
                    "received request! method: {:?}, url: {:?}, headers: {:?}, remote_addr: {}",
                    request.method(),
                    request.url(),
                    request.headers(),
                    request.remote_addr()
                );

                let mut response_body = String::new();
                response_body.push_str(
                    format!(
                        r#"
        <html>
            <head>
                <title>ip stats</title>
                <link rel="stylesheet" href="main.css">
            </head>
            <body>
                <header>
                    <h1>your ip is:</h1>
                    <code>{ip}</code>
                </header>
                <main>
        "#,
                        ip = get_ip(&request)
                    )
                    .as_str(),
                );
                for header in request.headers() {
                    response_body.push_str(
                        format!(
                            r#"        
                    <div class="header-container">
                        <code>[{}]</code>
                        <code>{}</code>
                    </div>
                "#,
                            header.field, header.value
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

                #[allow(unused_must_use)]
                {
                    request.respond(response);
                }
            }
            _ => {
                let response = Response::from_string("404 not found").with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"text/plain; charset=utf-8"[..])
                        .unwrap(),
                );

                #[allow(unused_must_use)]
                {
                    request.respond(response);
                }
            }
        }
    }
}
