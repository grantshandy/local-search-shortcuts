use std::collections::HashMap;

use compact_str::CompactString;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

type StringIndex = usize;

pub type SearchEngineRef<'a> = InternalSearchEngine<&'a CompactString, Option<&'a CompactString>>;
type DiskSearchEngine = InternalSearchEngine<CompactString, StringIndex>;

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct InternalSearchEngine<S, C> {
    pub name: S,
    pub url: S,
    pub category: C,
    pub subcategory: C,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchEngineDatabase {
    shortcuts: HashMap<CompactString, usize>,
    engines: IndexSet<DiskSearchEngine>,
    categories: IndexSet<CompactString>,
}

impl SearchEngineDatabase {
    pub fn new() -> Self {
        let mut me = Self {
            shortcuts: HashMap::new(),
            engines: IndexSet::new(),
            categories: IndexSet::new(),
        };

        me.categories.insert(CompactString::default());

        // index '0' represents a null value in the database.
        assert!(me
            .categories
            .get_index(0)
            .is_some_and(CompactString::is_empty));

        me.insert(
            &"info".into(),
            InternalSearchEngine {
                name: "View This Page".into(),
                url: "/info".into(),
                category: None,
                subcategory: None,
            },
        );

        me
    }

    pub fn insert(
        &mut self,
        shortcut: &CompactString,
        engine: InternalSearchEngine<CompactString, Option<CompactString>>,
    ) {
        let disk = DiskSearchEngine {
            name: engine.name,
            url: engine.url,
            category: self.insert_category(engine.category),
            subcategory: self.insert_category(engine.subcategory),
        };

        let (idx, _) = self.engines.insert_full(disk);
        self.shortcuts.insert(shortcut.to_lowercase(), idx);
    }

    pub fn get<'a>(&'a self, shortcut: &str) -> Option<SearchEngineRef<'a>> {
        self.shortcuts
            .get(shortcut.to_lowercase().as_str())
            .and_then(|idx| self.engines.get_index(*idx))
            .map(|disk| self.construct_engine(disk))
    }

    #[allow(dead_code)]
    pub fn engines(&self) -> impl Iterator<Item = (Vec<&CompactString>, SearchEngineRef<'_>)> {
        self.engines.iter().enumerate().map(|(idx, disk)| {
            (
                self.shortcuts
                    .iter()
                    .filter(|(_, &i)| i == idx)
                    .map(|(s, _)| s)
                    .collect(),
                self.construct_engine(disk),
            )
        })
    }

    fn construct_engine<'a>(&'a self, disk: &'a DiskSearchEngine) -> SearchEngineRef<'a> {
        SearchEngineRef {
            name: &disk.name,
            url: &disk.url,
            category: self.get_category(disk.category),
            subcategory: self.get_category(disk.subcategory),
        }
    }

    fn get_category(&self, idx: StringIndex) -> Option<&CompactString> {
        if idx != 0 {
            self.categories.get_index(idx)
        } else {
            None
        }
    }

    fn insert_category(&mut self, s: Option<CompactString>) -> StringIndex {
        if let Some(s) = s {
            self.categories.insert_full(s).0
        } else {
            0
        }
    }

    pub fn count(&self) -> usize {
        self.engines.len()
    }
}

pub mod default {
    pub fn engine() -> String {
        "DuckDuckGo".to_string()
    }

    #[allow(dead_code)]
    pub fn port() -> u16 {
        9321
    }
}
