use std::{env, error, fs};

#[path = "src/engines.rs"]
mod shared;

use compact_str::CompactString;
use shared::{InternalSearchEngine, SearchEngineDatabase};
use time::UtcDateTime;

const BANG_PATH: &str = "res/bang.json";

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

fn main() -> Result<(), Box<dyn error::Error>> {
    let parsed: Vec<ParsedEngine> = serde_json::from_slice(&fs::read(BANG_PATH)?)?;

    println!("cargo:rerun-if-changed={BANG_PATH}");

    let mut db = SearchEngineDatabase::default();

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

    assert!(
        db.engine_count() != 0,
        "No search engines found in bang.json"
    );
    assert!(
        db.get_engine(&shared::default::engine()).is_some(),
        "Default engine not found in bang.json"
    );

    let out_dir = env::var("OUT_DIR")?;

    let db_path = format!("{out_dir}/generated.bin");
    println!("cargo::rustc-env=LSS_DATABASE={db_path}");
    fs::write(db_path, rkyv::to_bytes::<rkyv::rancor::Error>(&db)?)?;

    let l = UtcDateTime::from(fs::metadata(BANG_PATH)?.modified()?);
    let last_updated_path = format!("{out_dir}/last_updated");
    println!("cargo::rustc-env=LSS_LAST_UPDATED={last_updated_path}");
    fs::write(
        last_updated_path,
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            l.year(),
            l.month() as u8,
            l.day(),
            l.hour(),
            l.minute(),
            l.second()
        ),
    )?;

    Ok(())
}
