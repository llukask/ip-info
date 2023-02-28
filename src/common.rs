use std::io::Read;
use tiny_http::{Header, Request, Response};

pub(crate) type HttpHandler<Ctx> = fn(ctx: &Ctx, request: Request);

pub(crate) fn get_real_ip(req: &Request) -> String {
    for header in req.headers() {
        if header.field.as_str() == "X-Real-Ip" {
            let ip = header.value.as_str();
            let ip = ip.split(',').next().unwrap();
            let ip = ip.trim();
            return ip.to_string();
        }
    }

    let ip = req.remote_addr().ip();
    ip.to_string()
}

pub(crate) fn log_response<R: std::io::Read>(
    request: &Request,
    response: &Response<R>,
    real_ip: &str,
) {
    println!(
        "{ip} {method} {url} {status} {response_size}",
        ip = real_ip,
        method = request.method().as_str(),
        url = request.url(),
        status = response.status_code().0,
        response_size = response.data_length().unwrap_or(0)
    );
}

pub(crate) fn header_value(request: &Request, header_name: &str) -> Option<Header> {
    request
        .headers()
        .iter()
        .find(|h| h.field.as_str() == header_name)
        .cloned()
}

pub(crate) fn send_response<R: Read>(request: Request, response: Response<R>) {
    log_response(&request, &response, &get_real_ip(&request));
    request.respond(response).unwrap_or_else(|err| {
        eprintln!("could not send response {err}");
    });
}
