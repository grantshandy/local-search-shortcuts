use std::{collections::HashMap, env, fmt::Debug, fs};

#[path = "src/shared.rs"]
mod shared;

use shared::SearchEngine;

#[derive(Debug, Clone, serde::Deserialize)]
struct ParsedEngine {
    #[serde(rename = "u")]
    url: String,
    #[serde(rename = "s")]
    name: String,
    #[serde(rename = "t")]
    shortcut: String,
    #[serde(rename = "c")]
    category: Option<String>,
    #[serde(rename = "sc")]
    subcategory: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parsed: Vec<ParsedEngine> = serde_json::from_slice(&fs::read("bang.json")?)?;

    let mut buf: HashMap<String, SearchEngine> = HashMap::new();

    for parse in parsed {
        let url = parse
            .url
            .replace("&ie={inputEncoding}", "")
            .replace("{", "{{")
            .replace("}", "}}")
            // ridiculous, I know
            .replace("{{{{{{s}}}}}}", "{s}")
            .replace("\"", "\\\"");

        // shortcuts to the duckduckgo website itself
        if url.starts_with("/") {
            continue;
        }

        buf.insert(
            parse.shortcut,
            SearchEngine {
                name: parse.name,
                category: parse.category,
                subcategory: parse.subcategory,
                url,
            },
        );
    }

    assert!(!buf.is_empty(), "No search engines found in bang.json");
    assert!(
        buf.contains_key(&shared::default_engine()),
        "Default engine not found in bang.json"
    );

    fs::write(
        format!("{}/generated.bin", env::var("OUT_DIR")?),
        bincode::encode_to_vec(&buf, bincode::config::standard())?,
    )?;

    Ok(())
}
