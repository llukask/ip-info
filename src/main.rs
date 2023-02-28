use std::env;
use tiny_http::{Header, Response, Server};

use crate::common::{send_response, HttpHandler};
use crate::content_negotiation::{content_negotiate, MediaType};

mod common;
mod content_negotiation;
mod handle_css;
mod handle_index;

fn main() {
    let port: i32 = env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(8000);

    let endpoint = format!("0.0.0.0:{port}");

    let server = Server::http(&endpoint)
        .unwrap_or_else(|_| panic!("could not start http server @ {}", &endpoint));

    println!("server running @ {}", &endpoint);

    let index_handlers: Vec<(MediaType, HttpHandler<()>)> = vec![
        ("text/plain".into(), handle_index::handle_index_plain),
        ("application/json".into(), handle_index::handle_index_json),
        ("text/html".into(), handle_index::handle_index_html),
    ];

    let css_handlers: Vec<(MediaType, HttpHandler<()>)> =
        vec![("text/css".into(), handle_css::handle_css)];

    for request in server.incoming_requests() {
        match request.url() {
            "/main.css" => content_negotiate(&(), request, &css_handlers),
            "/" => content_negotiate(&(), request, &index_handlers),
            _ => {
                let response = Response::from_string("404 not found").with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"text/plain; charset=utf-8"[..])
                        .unwrap(),
                );
                send_response(request, response)
            }
        };
    }
}
