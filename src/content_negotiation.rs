use crate::common::{header_value, send_response, HttpHandler};
use tiny_http::{Request, Response};

#[derive(Debug, PartialEq, Eq)]
pub struct MediaType {
    main_type: String,
    sub_type: String,

    parameter: Option<(String, String)>,
}

#[derive(Debug)]
pub struct Directive {
    media_type: MediaType,
    q: f32,
}

impl MediaType {
    fn part_matches(a: &str, b: &str) -> bool {
        a == b || a == "*" || b == "*"
    }

    pub fn matches(&self, other: &Self) -> bool {
        let main_matches = Self::part_matches(&self.main_type, &other.main_type);
        let sub_matches = Self::part_matches(&self.sub_type, &other.sub_type);

        main_matches && sub_matches
    }
}

impl From<&str> for MediaType {
    fn from(s: &str) -> Self {
        let (media_main, rest) = s.split_once('/').expect("a media type must contain a '/'");
        let (sub_type, parameter_str) = rest.split_once(';').unwrap_or((rest, ""));

        let param_opt = parameter_str
            .split_once('=')
            .map(|(k, v)| (k.to_string(), v.to_string()));

        MediaType {
            main_type: media_main.to_string(),
            sub_type: sub_type.to_string(),
            parameter: param_opt,
        }
    }
}

pub fn parse_accept_directive(directive_str: &str) -> Directive {
    let (media_main, rest) = directive_str
        .split_once('/')
        .expect("a media type must contain a '/'");
    let (rest, q_str) = rest.split_once(";q=").unwrap_or((rest, "1.0"));
    let (sub_type, parameter_str) = rest.split_once(';').unwrap_or((rest, ""));

    let param_opt = parameter_str
        .split_once('=')
        .map(|(k, v)| (k.to_string(), v.to_string()));

    let media_type = MediaType {
        main_type: media_main.to_string(),
        sub_type: sub_type.to_string(),
        parameter: param_opt,
    };

    let q = q_str.parse::<f32>().expect("invalid q-value");

    Directive { media_type, q }
}

pub fn parse_accept(header_value: &str) -> Vec<Directive> {
    let mut r = header_value
        .split(',')
        .map(parse_accept_directive)
        .collect::<Vec<Directive>>();
    r.sort_by(|a, b| b.q.partial_cmp(&a.q).unwrap());
    r
}

fn find_handler<'a, Ctx>(
    request: &Request,
    handlers: &'a [(MediaType, HttpHandler<Ctx>)],
) -> Option<&'a HttpHandler<Ctx>> {
    if let Some(accept_header) = header_value(request, "Accept") {
        let directives = parse_accept(accept_header.value.as_str());

        for d in directives {
            if let Some((_, handler)) = handlers.iter().find(|(mt, _)| mt.matches(&d.media_type)) {
                return Some(handler);
            }
        }

        None
    } else {
        Some(&handlers.first().unwrap().1)
    }
}

pub fn content_negotiate<Ctx>(
    ctx: &Ctx,
    request: Request,
    handlers: &[(MediaType, HttpHandler<Ctx>)],
) {
    match find_handler(&request, handlers) {
        Some(handler) => handler(ctx, request),
        None => {
            let response = Response::from_string("406 Not Acceptable").with_status_code(406);
            send_response(request, response)
        }
    }
}
