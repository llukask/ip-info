use std::sync::Arc;
use std::{env, thread};
use tiny_http::{Header, Response, Server, StatusCode};

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

    let thread_count: usize = env::var("THREAD_COUNT")
        .ok()
        .and_then(|thread_count| thread_count.parse().ok())
        .unwrap_or(4);

    let endpoint = format!("0.0.0.0:{port}");

    let server = Arc::new(
        Server::http(&endpoint)
            .unwrap_or_else(|_| panic!("could not start http server @ {}", &endpoint)),
    );

    let mut guards = Vec::with_capacity(thread_count);

    println!(
        "server running @ {} (with {thread_count} threads)",
        &endpoint
    );

    for _ in 0..thread_count {
        let server = server.clone();

        let guard = thread::spawn(move || {
            let index_handlers: &[(MediaType, HttpHandler<()>)] = &[
                ("text/plain".into(), handle_index::handle_index_plain),
                ("application/json".into(), handle_index::handle_index_json),
                ("text/html".into(), handle_index::handle_index_html),
            ];

            let css_handlers: &[(MediaType, HttpHandler<()>)] =
                &[("text/css".into(), handle_css::handle_css)];

            loop {
                let request = server.recv().expect("could not receive request");

                match request.url() {
                    "/main.css" => content_negotiate(&(), request, css_handlers),
                    "/" => content_negotiate(&(), request, index_handlers),
                    _ => respond_plain(request, 404, "404 not found"),
                };
            }
        });

        guards.push(guard);
    }

    guards.into_iter().for_each(|guard| guard.join().unwrap());
}

fn respond_plain<S: Into<StatusCode>>(request: tiny_http::Request, status: S, response: &str) {
    let content_type =
        Header::from_bytes(&b"Content-Type"[..], &b"text/plain; charset=utf-8"[..]).unwrap();

    let response = Response::from_string(response)
        .with_header(content_type)
        .with_status_code(status);

    send_response(request, response)
}
