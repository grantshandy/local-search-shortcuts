use std::{io::Cursor, sync::LazyLock, thread};

use compact_str::CompactString;
use tiny_http::{Header, Request, Response, Server, StatusCode};

mod config;
mod engines;
mod info;

use config::CONFIG;

static ENGINES: LazyLock<engines::SearchEngineDatabase> = LazyLock::new(|| {
    bincode::serde::decode_from_slice(
        include_bytes!(env!("LSS_DATABASE")),
        bincode::config::standard(),
    )
    // should never happen
    .unwrap()
    .0
});

fn main() {
    tracing_subscriber::fmt::init();

    // just for a fast first search, isn't necessary
    let _ = LazyLock::force(&ENGINES);
    let _ = LazyLock::force(&CONFIG);

    tracing::info!(
        "loaded {} search engines",
        ENGINES.count() + CONFIG.engines.count()
    );
    tracing::info!("launching service at http://{}/", CONFIG.addr());

    let server = match Server::http(CONFIG.addr()) {
        Ok(server) => server,
        Err(e) => {
            tracing::error!("failed to start service: {e}");
            return;
        }
    };

    for request in server.incoming_requests() {
        thread::spawn(move || {
            let response = handle_request(&request);

            if let Err(e) = request.respond(response) {
                tracing::error!("error handling request: {e}");
            }
        });
    }
}

fn handle_request(request: &Request) -> Response<Cursor<Vec<u8>>> {
    if let Some(terms) = request.url().strip_prefix("/?q=") {
        let redirect = parse_terms(terms);

        return Response::new(
            StatusCode(302),
            vec![
                Header::from_bytes("Location", redirect.as_bytes()).unwrap(),
                Header::from_bytes("Cache-Control", "no-cache, no-store, must-revalidate").unwrap(),
            ],
            Cursor::new(Vec::with_capacity(0)),
            Some(0),
            None,
        );
    }

    let (data, code) = match request.url() {
        "/" => (info::INDEX.as_bytes(), 200),
        "/info" => (info::INFO.as_bytes(), 200),
        _ => (info::NOT_FOUND.as_bytes(), 404),
    };

    Response::from_data(data)
        .with_status_code(StatusCode(code))
        .with_header(Header::from_bytes("Content-Type", "text/html").unwrap())
}

fn parse_terms(encoded_terms: &str) -> String {
    let terms = urlencoding::decode(encoded_terms)
        .expect("url not encoded as utf8 (impossible)")
        .replace('+', " ");

    let Some((shortcut, url)): Option<(&str, &CompactString)> = terms
        .split_whitespace()
        .find(|s| s.starts_with('!'))
        .and_then(|s| {
            let trimmed = s.trim_start_matches('!');

            ENGINES
                .get(trimmed)
                .or(CONFIG.engines.get(trimmed))
                .map(|e| (s, e.url))
        })
    else {
        return CONFIG.default_engine.url.replace("{s}", encoded_terms);
    };

    if !url.contains("{s}") {
        return url.to_string();
    }

    url.replace(
        "{s}",
        urlencoding::encode(terms.replace(shortcut, "").trim()).as_ref(),
    )
}
