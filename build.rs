use std::{env, fs};

#[path = "src/shared.rs"]
mod shared;

use compact_str::CompactString;
use shared::{InternalSearchEngine, SearchEngineDatabase};

#[derive(serde::Deserialize)]
struct ParsedEngine {
    #[serde(rename = "u")]
    url: CompactString,
    #[serde(rename = "s")]
    name: CompactString,
    #[serde(rename = "t")]
    shortcut: CompactString,
    #[serde(rename = "c")]
    category: Option<CompactString>,
    #[serde(rename = "sc")]
    subcategory: Option<CompactString>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parsed: Vec<ParsedEngine> = serde_json::from_slice(&fs::read("bang.json")?)?;

    println!("cargo:rerun-if-changed=bang.json");

    let mut db = SearchEngineDatabase::new();

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

        db.insert(
            &parse.shortcut,
            InternalSearchEngine {
                name: parse.name,
                url: url.into(),
                category: parse.category,
                subcategory: parse.subcategory,
            },
        );
    }

    assert!(db.count() != 0, "No search engines found in bang.json");
    assert!(
        db.get(&shared::default::engine()).is_some(),
        "Default engine not found in bang.json"
    );

    let db_path = format!("{}/generated.bin", env::var("OUT_DIR")?);

    println!("cargo::rustc-env=LSS_DATABASE={db_path}");

    fs::write(
        db_path,
        bincode::serde::encode_to_vec(&db, bincode::config::standard())?,
    )?;

    Ok(())
}
