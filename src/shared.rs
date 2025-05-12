#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
pub struct SearchEngine {
    pub name: String,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub url: String,
}

pub fn default_engine() -> String {
    "duckduckgo".to_string()
}
