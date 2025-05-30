use std::{cmp::Ordering, collections::HashMap, fmt::Write, sync::LazyLock};

use compact_str::CompactString;
use maud::{html, Markup, PreEscaped};

use crate::{config::CONFIG_CHECKS, engines::SearchEngineRef, CONFIG};

pub static INDEX: LazyLock<String> = LazyLock::new(|| {
    base_html(
        "Local Search Shortcuts",
        html! {
            p {
                a href="/info" { "List of Available Shortcuts" }
                " - "
                i { "Search " code { "!info" } " to view this page at any time." }
            }
            hr;
            h2 { "Instructions:" }
            p { "Just set this as the search engine in your browser:" }
            pre { "http://localhost:8080/?q=[TERMS]" }
            p { "Then use the many search engine shortcuts like so:" }
            pre { "!w Hello World" }
            p { "This redirects to the relevant Wikipedia page or search results." }
            p { i { "(the placement of the shortcut is not important, and the first one found is always used)" } }
            hr;
            h2 { "Configuration" }
            h3 { "Current Configuration" }
            p {
                "Configuration File: "
                @if let Some(path) = CONFIG
                    .path
                    .as_ref()
                    .map(|path|
                        path
                            .canonicalize()
                            .unwrap_or(path.clone())
                            .to_string_lossy()
                            .to_string()
                    )
                {
                    code { (path) }
                } @else {
                    b { "None detected, using defaults" }
                }
            }
            p {
                "Default Search Engine: "
                a href=(CONFIG.default_engine.url.replace("{s}", "")) {
                    (CONFIG.default_engine.name)
                }
            }
            h3 { "Configuration Options" }
            p { "Here's an example configuration file:" }
            pre { (include_str!("../local-search-shortcuts.toml")) }
            p { "Configuration files are read in this order:" }
            ul {
                @for path in CONFIG_CHECKS.iter().map(|path| path.canonicalize().unwrap_or(path.clone())) {
                    li { code { (path.to_string_lossy()) } }
                }
            }
        }
    ).into()
});

pub static NOT_FOUND: LazyLock<String> = LazyLock::new(|| {
    let msg = "Error 404: Page Doesn't Exist";
    base_html(msg, html! { (msg) }).into()
});

pub static INFO: LazyLock<String> = LazyLock::new(|| {
    base_html(
        "Local Search Shortcuts Index",
        render_categories(generate_categories()),
    ).into()
});

struct EngineDescription {
    name: CompactString,
    shortcuts: String,
}

type Subcategory = HashMap<String, EngineDescription>;
type Category = HashMap<String, Subcategory>;

const UNCATEGORIZED: &str = "Uncategorized";
const CUSTOM: &str = "Custom";

fn generate_categories() -> Vec<(String, Category)> {
    let mut categories: HashMap<String, Category> = HashMap::new();

    for (shortcuts, engine) in crate::ENGINES.engines() {
        let category_name = engine
            .category
            .map(|s| s.to_string())
            .unwrap_or(UNCATEGORIZED.into());

        let subcategory_name = engine
            .subcategory
            .map(|s| s.to_string())
            .unwrap_or_default();

        let (url, description) = map_engine((shortcuts, engine));

        categories
            .entry(category_name)
            .or_default()
            .entry(subcategory_name)
            .or_default()
            .insert(url, description);
    }

    let custom = crate::CONFIG.engines.engines().map(map_engine).collect();

    categories
        .entry(CUSTOM.to_string())
        .or_default()
        .insert(String::new(), custom);

    let mut categories: Vec<(String, Category)> = categories.into_iter().collect();

    // Sort by Custom -> Alphabetical -> Uncategorized
    categories.sort_by(|(a, _), (b, _)| {
        if a == UNCATEGORIZED {
            Ordering::Greater
        } else if a == CUSTOM {
            Ordering::Less
        } else {
            a.to_lowercase().cmp(&b.to_lowercase())
        }
    });

    categories
}

fn map_engine(
    (shortcuts, engine): (Vec<&CompactString>, SearchEngineRef),
) -> (String, EngineDescription) {
    let shortcuts = shortcuts.into_iter().fold(String::new(), |mut acc, s| {
        if !acc.is_empty() {
            acc.push_str(", ");
        }
        let _ = write!(acc, "!{s}");
        acc
    });

    (
        engine.url.replace("{s}", ""),
        EngineDescription {
            name: engine.name.clone(),
            shortcuts,
        },
    )
}

fn render_categories(categories: Vec<(String, Category)>) -> Markup {
    let category_id = |category: &str| category.replace(" ", "_");
    let subcategory_id = |a: &str, subcategory: &str| format!("{a}_{}", category_id(subcategory));
    let tag = |name: &str| format!("#{name}");

    html! {
        p { i { a href="/" { "Back to Main Page" } } }
        hr;
        div style="text-align: left;" {
            h3 { "Categories" }
            ol {
                @for (category, subcategories) in &categories {
                    @let category_id = category_id(category);
                    li {
                        a href=(tag(&category_id)) { (category) }
                        ul {
                            @for (subcategory, _) in subcategories.iter().filter(|(s, _)| !s.is_empty()) {
                                li {
                                    a href=(tag(&subcategory_id(&category_id, subcategory))) { (subcategory) }
                                }
                            }
                        }
                    }
                }
            }

            @for (category, subcategories) in categories {
                @let category_id = category_id(&category);
                hr;
                h2 id=(category_id) { (category) }
                ul {
                    @for (subcategory, engines) in subcategories {
                        h3 id=(subcategory_id(&category_id, &subcategory)) { (subcategory) }
                        ul {
                            @for (url, engine) in engines {
                                li {
                                    a href=(url) { (engine.name) }
                                    ": "
                                    (engine.shortcuts)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn base_html(title: &str, body: Markup) -> Markup {
    html! {
        (maud::DOCTYPE)
        head {
            meta charset="UTF-8";
            title { (title) }
            style {
                (PreEscaped(r#"
                    body {
                        font-family: Arial, sans-serif; 
                        margin: 0 auto; 
                        max-width: 800px; 
                        padding: 2em; 
                        background-color: #f9f9f9; 
                        color: #333333; 
                        text-align: center; 
                    }
                    a { color: #007acc; text-decoration: none; }
                    a:hover { text-decoration: underline; }
                    pre,code { border: 1px solid #dddddd; background: #f4f4f4; padding: 0.5em; }
                    pre { padding: 1em; text-align: left; overflow-x: auto; }
                    ul, ol { text-align: left; margin: 1em 0 1em 2em; padding: 0; }
                    li { margin: 0.5em 0; }
                "#))
            }
        }

        body {
            h1 { "Local Search Shortcuts v" (env!("CARGO_PKG_VERSION")) }
            p {
                i {
                    (PreEscaped("&copy;2025 Grant Handy &#124; "))
                    a href="https://github.com/grantshandy/local-search-shortcuts" { "View Source" }
                    (PreEscaped(" &#124; "))
                    a href="https://buymeacoffee.com/granthandy" { "Donate" }
                }
            }
            hr;
            (body)
        }

    }
}
