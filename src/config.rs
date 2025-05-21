use std::{
    collections::HashMap,
    fs, iter,
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::LazyLock,
};

use compact_str::CompactString;

use crate::engines::{default, InternalSearchEngine, SearchEngineDatabase};

pub static CONFIG_CHECKS: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
    dirs::config_dir()
        .into_iter()
        .map(|dir| dir.join("lss/config.toml"))
        .chain(iter::once("lss.toml".into()))
        .collect()
});

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let config = CONFIG_CHECKS
        .iter()
        .find_map(Config::from_file)
        .unwrap_or_default();

    if let Some(ref path) = config.path {
        tracing::info!("loaded config file {path:?}");
    } else {
        tracing::info!("no config file found, using defaults");
    }

    tracing::debug!("{config:#?}");

    config
});

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub default_engine: OwnedSearchEngine,
    pub broadcast: bool,
    pub engines: SearchEngineDatabase,
    pub path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: default::port(),
            broadcast: false,
            // unwrap: asserted in build.rs that default engine is present
            default_engine: force_clone(&crate::ENGINES.get(&default::engine()).unwrap()),
            engines: SearchEngineDatabase::new(),
            path: None,
        }
    }
}

impl Config {
    pub fn addr(&self) -> SocketAddr {
        let ip = if self.broadcast {
            Ipv4Addr::UNSPECIFIED
        } else {
            Ipv4Addr::LOCALHOST
        };

        (ip, self.port).into()
    }

    fn from_file(path: &PathBuf) -> Option<Self> {
        let file = match String::from_utf8(fs::read(path).ok()?)
            .map_err(|e| e.to_string())
            .and_then(|t| toml::from_str::<ConfigFile>(&t).map_err(|e| e.to_string()))
        {
            Ok(text) => text,
            Err(err) => {
                tracing::warn!("failed to parse config file {path:?}: {err}");
                return None;
            }
        };

        let path = path.canonicalize().unwrap_or(path.clone());

        let mut engines = SearchEngineDatabase::new();

        for (name, url) in file.engines {
            engines.insert(
                &name.clone().into(),
                InternalSearchEngine {
                    name: name.into(),
                    url: url.into(),
                    category: Some("Custom".into()),
                    subcategory: None,
                },
            );
        }

        let default_engine = engines
            .get(&file.default)
            .or(crate::ENGINES.get(&file.default))
            .unwrap_or_else(|| {
                tracing::warn!(
                    "config's default engine '{}' not found, using {}",
                    file.default,
                    default::engine()
                );
                // unwrap: asserted in build.rs that default engine is present
                crate::ENGINES.get(&default::engine()).unwrap()
            });

        Some(Self {
            port: file.port,
            default_engine: force_clone(&default_engine),
            broadcast: file.broadcast,
            engines,
            path: Some(path),
        })
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ConfigFile {
    #[serde(default = "default::port")]
    port: u16,
    #[serde(default = "default::engine")]
    default: String,
    #[serde(default)]
    broadcast: bool,
    #[serde(default)]
    engines: HashMap<String, String>,
}

pub type OwnedSearchEngine = InternalSearchEngine<CompactString, Option<CompactString>>;

fn force_clone(
    engine: &InternalSearchEngine<&'_ CompactString, Option<&'_ CompactString>>,
) -> OwnedSearchEngine {
    OwnedSearchEngine {
        name: engine.name.clone(),
        url: engine.url.clone(),
        category: engine.category.cloned(),
        subcategory: engine.subcategory.cloned(),
    }
}
