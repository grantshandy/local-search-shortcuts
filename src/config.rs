use std::{collections::HashMap, fs, net::Ipv4Addr, path::PathBuf, sync::LazyLock};

use crate::shared::{default_engine, SearchEngine};

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let (c, port): (Config, Option<PathBuf>) = dirs::config_dir()
        .map(|dir| dir.join("lss").join("config.toml"))
        .and_then(Config::from_file)
        .or(Config::from_file(PathBuf::new().join("lss.toml")))
        .unwrap_or_default();

    if let Some(path) = port {
        tracing::info!("loaded config file {path:?}");
    } else {
        tracing::info!("no config file found, using defaults");
    }

    c
});

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub default: SearchEngine,
    pub broadcast: bool,
    pub engines: HashMap<String, SearchEngine>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: default_port(),
            broadcast: false,
            default: crate::ENGINES.get(&default_engine()).unwrap().clone(),
            engines: HashMap::new(),
        }
    }
}

impl Config {
    pub fn emit_ip(&self) -> Ipv4Addr {
        if self.broadcast {
            Ipv4Addr::UNSPECIFIED
        } else {
            Ipv4Addr::LOCALHOST
        }
    }

    fn from_file(path: PathBuf) -> Option<(Self, Option<PathBuf>)> {
        let file = fs::read(&path)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|s| toml::from_str::<ConfigFile>(&s).ok())?;

        let engines: HashMap<String, SearchEngine> = file
            .engines
            .into_iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    SearchEngine {
                        name: k,
                        category: Some("Custom".to_string()),
                        subcategory: None,
                        url: v,
                    },
                )
            })
            .collect();

        let default = engines
            .get(&file.default)
            .or(crate::ENGINES.get(&file.default))
            .unwrap_or_else(|| {
                tracing::warn!(
                    "config's default engine '{}' not found, using {}",
                    file.default,
                    default_engine()
                );
                crate::ENGINES.get(&default_engine()).unwrap()
            })
            .clone();

        Some((
            Self {
                port: file.port,
                default,
                broadcast: file.broadcast,
                engines,
            },
            Some(path),
        ))
    }
}

#[derive(serde::Deserialize)]
struct ConfigFile {
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_engine")]
    default: String,
    #[serde(default)]
    broadcast: bool,
    #[serde(default)]
    engines: HashMap<String, String>,
}

fn default_port() -> u16 {
    9321
}
