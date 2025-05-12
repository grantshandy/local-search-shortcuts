use std::{
    collections::HashMap,
    fs, iter,
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::LazyLock,
};

use crate::shared::{default_engine, SearchEngine};

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
        .filter_map(Config::from_file)
        .next()
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
    pub default: SearchEngine,
    pub broadcast: bool,
    pub engines: HashMap<String, SearchEngine>,
    pub path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: default_port(),
            broadcast: false,
            // unwrap: it's asserted that default_engine() is in the engines map in build.rs
            default: crate::ENGINES.get(&default_engine()).unwrap().clone(),
            engines: HashMap::new(),
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
        let file = fs::read(path)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|s| toml::from_str::<ConfigFile>(&s).ok())?;

        let path = path.canonicalize().unwrap_or(path.clone());

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

        Some(Self {
            port: file.port,
            default,
            broadcast: file.broadcast,
            engines,
            path: Some(path),
        })
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
