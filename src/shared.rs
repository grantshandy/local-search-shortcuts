use compact_str::CompactString;
use indexmap::IndexSet;
use std::collections::HashMap;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub struct SearchEngine {
    pub name: CompactString,
    pub category: Option<CompactString>,
    pub subcategory: Option<CompactString>,
    pub url: CompactString,
}

pub fn default_engine() -> String {
    "duckduckgo".to_string()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchEngineDatabase {
    pub engines: IndexSet<SearchEngine>,
    pub shortcuts: HashMap<CompactString, usize>,
}

impl SearchEngineDatabase {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let mut me = Self {
            engines: IndexSet::new(),
            shortcuts: HashMap::new(),
        };

        me.insert(
            "info".to_string(),
            SearchEngine {
                name: "View This Page".into(),
                category: None,
                subcategory: None,
                url: "/info".into(),
            },
        );

        me
    }

    #[allow(dead_code)]
    pub fn insert(&mut self, shortcut: String, engine: SearchEngine) {
        let (idx, _) = self.engines.insert_full(engine);
        self.shortcuts.insert(shortcut.to_lowercase().into(), idx);
    }

    pub fn get<'a>(&'a self, shortcut: &str) -> Option<&'a SearchEngine> {
        self.shortcuts
            .get(shortcut.to_lowercase().as_str())
            .and_then(|idx| self.engines.get_index(*idx))
    }

    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.engines.len()
    }
}
