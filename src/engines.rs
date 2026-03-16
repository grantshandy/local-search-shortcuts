use std::collections::HashMap;

use compact_str::CompactString;
use indexmap::IndexSet;
use rkyv::{rend::u32_le, string::ArchivedString, Archive, Deserialize, Serialize};

type StringIndex = usize;

pub type SearchEngineRef<'a> = InternalSearchEngine<&'a str, Option<&'a str>>;
type DiskSearchEngine = InternalSearchEngine<CompactString, StringIndex>;

#[derive(Debug, Archive, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct InternalSearchEngine<S, C> {
    pub name: S,
    pub url: S,
    pub category: C,
    pub subcategory: C,
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub struct SearchEngineDatabase {
    shortcuts: HashMap<CompactString, usize>,
    engines: IndexSet<DiskSearchEngine>,
    categories: IndexSet<CompactString>,
}

impl Default for SearchEngineDatabase {
    fn default() -> Self {
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
}

impl SearchEngineDatabase {
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

    fn insert_category(&mut self, s: Option<CompactString>) -> StringIndex {
        if let Some(s) = s {
            self.categories.insert_full(s).0
        } else {
            0
        }
    }

    pub fn get_engine<'a>(&'a self, shortcut: &str) -> Option<SearchEngineRef<'a>> {
        self.shortcuts
            .get(shortcut.to_lowercase().as_str())
            .and_then(|idx| self.engines.get_index(*idx))
            .map(|disk| self.construct_engine(disk))
    }

    fn construct_engine<'a>(&'a self, disk: &'a DiskSearchEngine) -> SearchEngineRef<'a> {
        SearchEngineRef {
            name: disk.name.as_str(),
            url: disk.url.as_str(),
            category: self.get_category(disk.category),
            subcategory: self.get_category(disk.subcategory),
        }
    }

    fn get_category(&self, idx: StringIndex) -> Option<&str> {
        if idx != 0 {
            self.categories.get_index(idx).map(CompactString::as_str)
        } else {
            None
        }
    }

    pub fn engine_count(&self) -> usize {
        self.engines.len()
    }

    pub fn engines(&self) -> impl Iterator<Item = (Vec<&str>, SearchEngineRef<'_>)> {
        self.engines.iter().enumerate().map(|(idx, disk)| {
            (
                self.shortcuts
                    .iter()
                    .filter(|(_, &i)| i == idx)
                    .map(|(s, _)| s.as_str())
                    .collect(),
                self.construct_engine(disk),
            )
        })
    }
}

impl ArchivedSearchEngineDatabase {
    pub fn get_engine<'a>(&'a self, shortcut: &str) -> Option<SearchEngineRef<'a>> {
        self.shortcuts
            .get(shortcut.to_lowercase().as_str())
            .and_then(|idx| self.engines.get_index((*idx).to_native() as usize))
            .map(|disk| self.construct_engine(disk))
    }

    fn construct_engine<'a>(
        &'a self,
        disk: &'a ArchivedInternalSearchEngine<compact_str::CompactString, usize>,
    ) -> SearchEngineRef<'a> {
        SearchEngineRef {
            name: disk.name.as_str(),
            url: disk.url.as_str(),
            category: self.get_category(disk.category),
            subcategory: self.get_category(disk.subcategory),
        }
    }

    fn get_category(&self, idx: u32_le) -> Option<&str> {
        if idx != 0 {
            self.categories
                .get_index(idx.to_native() as usize)
                .map(ArchivedString::as_str)
        } else {
            None
        }
    }

    pub fn engine_count(&self) -> usize {
        self.engines.len()
    }

    pub fn engines(&self) -> impl Iterator<Item = (Vec<&str>, SearchEngineRef<'_>)> {
        self.engines.iter().enumerate().map(|(idx, disk)| {
            (
                self.shortcuts
                    .iter()
                    .filter(|(_, &i)| i.to_native() as usize == idx)
                    .map(|(s, _)| s.as_str())
                    .collect(),
                self.construct_engine(disk),
            )
        })
    }
}

#[allow(dead_code)]
pub mod default {
    pub fn engine() -> String {
        "DuckDuckGo".to_string()
    }

    pub fn port() -> u16 {
        9322
    }
}
