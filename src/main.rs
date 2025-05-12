use std::{collections::HashMap, sync::LazyLock};

use axum::{
    extract::Query,
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing, Router,
};
use color_eyre::eyre::Result;
use tokio::net::TcpListener;

mod config;
mod info;
mod shared;

use config::CONFIG;
use shared::SearchEngine;

static ENGINES: LazyLock<HashMap<String, SearchEngine>> = LazyLock::new(|| {
    let mut internal: HashMap<String, SearchEngine> = bincode::decode_from_slice(
        include_bytes!(concat!(env!("OUT_DIR"), "/generated.bin")),
        bincode::config::standard(),
    )
    // should never happen
    .expect("decode embedded resource")
    .0;

    internal.insert(
        "info".to_string(),
        SearchEngine {
            name: "View This Page".to_string(),
            category: None,
            subcategory: None,
            url: "/info".to_string(),
        },
    );

    internal
});

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    // just for a fast first search, isn't necessary
    let _ = LazyLock::force(&ENGINES);
    let _ = LazyLock::force(&CONFIG);

    tracing::info!(
        "launching service at http://{}:{}/",
        CONFIG.emit_ip(),
        CONFIG.port
    );

    axum::serve(
        TcpListener::bind((CONFIG.emit_ip(), CONFIG.port)).await?,
        Router::new()
            .route("/", routing::get(index))
            .route("/info", routing::get(|| async { Html(info::INFO.clone()) }))
            .fallback(routing::get(|| async {
                (
                    StatusCode::NOT_FOUND,
                    Html(info::base_html("<h2>Error 404: Page Doesn't Exist</h2>")),
                )
                    .into_response()
            })),
    )
    .await?;

    Ok(())
}

async fn index(Query(SearchQuery { q }): Query<SearchQuery>) -> Response {
    match q {
        None => Html(info::INDEX.clone()).into_response(),
        Some(q) => (
            StatusCode::FOUND,
            [
                (header::CONTENT_LENGTH, HeaderValue::from_static("0")),
                (
                    header::LOCATION,
                    HeaderValue::from_str(&parse_terms(&q)).unwrap(),
                ),
                (
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("no-cache, no-store, must-revalidate"),
                ),
            ],
        )
            .into_response(),
    }
}

#[derive(serde::Deserialize)]
struct SearchQuery {
    q: Option<String>,
}

fn parse_terms(terms: &str) -> String {
    let Some((shortcut, url)): Option<(&str, String)> = terms
        .split_whitespace()
        .find(|s| s.starts_with('!'))
        .and_then(|s| {
            let trimmed = s.trim_start_matches('!').to_lowercase();

            ENGINES
                .get(&trimmed)
                .or(CONFIG.engines.get(&trimmed))
                .map(|e| (s, e.url.clone()))
        })
    else {
        return CONFIG
            .default
            .url
            .replace("{s}", &urlencoding::encode(terms));
    };

    if url.contains("{s}") {
        url.replace(
            "{s}",
            &urlencoding::encode(terms.replace(shortcut, "").trim()),
        )
    } else {
        url
    }
}
