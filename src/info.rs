use std::{cmp::Ordering, collections::HashMap, sync::LazyLock};

use crate::{config::CONFIG_CHECKS, shared::SearchEngine, CONFIG};

pub static INFO: LazyLock<String> = LazyLock::new(|| {
    tracing::debug!("rendering info page");
    base_html(&render_categories(generate_categories()))
});

pub static INDEX: LazyLock<String> = LazyLock::new(|| {
    tracing::debug!("rendering main page");

    let check_paths = CONFIG_CHECKS
        .iter()
        .map(|path| path.canonicalize().unwrap_or(path.clone()))
        .map(|path| format!("<li><code>{path:?}</code></li>"))
        .collect::<String>();

    let active_config = CONFIG
        .path
        .as_ref()
        .map(|path| {
            path.canonicalize()
                .unwrap_or(path.clone())
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or("None detected".to_string());

    let port = CONFIG.port;
    let example = include_str!("../lss.toml");
    let default = &CONFIG.default.name;

    base_html(&format!(
        r#"
        <p>
            <a href="/info">List of Available Shortcuts</a>
            -
            <i>Search <code>!info</code> to view this page at any time.</i>
        </p>
        <hr>
        <h2>Usage Instructions:</h2>
        <p>Just set this as the search engine in your browser:</p>
        <pre>http://localhost:{port}/?q=[TERMS]</pre>
        <p>Then use the many search engine shortcuts like so:</p>
        <pre>!wiki Hello World</pre>
        <p>This redirects to the Wikipedia page or search results.</p>
        <p><i>(the placement of the shortcut is not important, and the first one found is always used)</i></p>
        <hr>
        <h2>Configuration</h2>
        <h4>Current</h4>
        <p>Configuration File: <code>{active_config}</code></p>
        <p>Default Search Engine: <code>{default}</code></p>
        <h4>Example</h4>
        <pre>{example}</pre>
        <p>Configuration files are read in this order:</p>
        <ul>{check_paths}</ul>
    "#
    ))
});

struct EngineDescription {
    name: String,
    shortcuts: Vec<String>,
}

type Subcategory = HashMap<String, EngineDescription>;
type Category = HashMap<String, Subcategory>;

const UNCATEGORIZED: &str = "Uncategorized";
const CUSTOM: &str = "Custom";

fn generate_categories() -> Vec<(String, Category)> {
    let mut categories: HashMap<String, Category> = HashMap::new();

    for (shortcut, engine) in crate::ENGINES.iter() {
        let subcategory = categories
            .entry(engine.category.clone().unwrap_or(UNCATEGORIZED.to_string()))
            .or_default()
            .entry(engine.subcategory.clone().unwrap_or("".to_string()))
            .or_default();

        add_shortcut(subcategory, shortcut, engine.clone());
    }

    let mut custom: Subcategory = Subcategory::new();

    for (shortcut, engine) in crate::CONFIG.engines.iter() {
        add_shortcut(&mut custom, shortcut, engine.clone());
    }

    categories
        .entry(CUSTOM.to_string())
        .or_default()
        .insert("".to_string(), custom);

    let mut categories: Vec<(String, Category)> = categories.into_iter().collect();

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

fn add_shortcut(subcategory: &mut Subcategory, shortcut: &str, engine: SearchEngine) {
    subcategory
        .entry(engine.url.replace("{s}", ""))
        .or_insert(EngineDescription {
            name: engine.name,
            shortcuts: vec![],
        })
        .shortcuts
        .push(format!("!{shortcut}"));
}

fn render_categories(categories: Vec<(String, Category)>) -> String {
    let mut output = String::new();

    output.push_str(r#"
        <p><i><a href="/">Back to Main Page</a></i></p>
        <hr>
        <h3>Categories</h3>
        <ol>
    "#);

    for (category, subcategories) in categories.iter() {
        let category_id = category.replace(' ', "_");
        output.push_str(&format!(
            "<li><a href=\"#{category_id}\">{category}</a><ul>"
        ));

        for (subcategory, _) in subcategories.iter().filter(|(s, _)| !s.is_empty()) {
            let subcategory_id = format!("{category_id}_{}", subcategory.replace(' ', "_"));
            output.push_str(&format!(
                "<li><a href=\"#{subcategory_id}\">{subcategory}</a></li>"
            ));
        }

        output.push_str("</ul></li>");
    }

    output.push_str("</ol>");

    for (category, subcategories) in categories {
        let category_id = category.replace(' ', "_");
        output.push_str(&format!("<hr><h3 id=\"{category_id}\">{category}</h3>"));

        for (subcategory, engines) in subcategories.iter() {
            let subcategory_id = format!("{category_id}_{}", subcategory.replace(' ', "_"));
            output.push_str(&format!("<h4 id=\"{subcategory_id}\">{subcategory}</h4>"));

            output.push_str("<ul>");

            for (url, engine) in engines.iter() {
                output.push_str(&format!(
                    "<li><a href=\"{url}\">{}</a>: {}</li>",
                    engine.name,
                    engine.shortcuts.join(", ")
                ));
            }

            output.push_str("</ul>");
        }
    }

    output
}

pub(crate) fn base_html(content: &str) -> String {
    format!(r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Search Shortcuts</title>
            <style>
                body {{ padding: 2em; background-color: #242424; color: #ffffff; }}
                a {{ color: #3584e4; }}
                pre {{ border: 1px solid #ffffff; padding: 0.5em; background: #1e1e1e; }}
            </style>
        </head>
        <body>
            <h1>Local Search Shortcuts</h1>
            <p>
                <i>
                    &copy;2025 Grant Handy
                    &#124; <a href="https://github.com/grantshandy/lss">View Source</a>
                    &#124; <a href="https://buymeacoffee.com/granthandy">Donate</a>
                </i>
            </p>
            {content}
        </body>
        </html>
    "#)
}
