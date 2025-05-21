use std::{cmp::Ordering, collections::HashMap, fmt::Write, sync::LazyLock};

use compact_str::CompactString;

use crate::{config::CONFIG_CHECKS, shared::SearchEngineRef, CONFIG};

const EXAMPLE_CONFIG: &str = include_str!("../lss.toml");

pub static INFO: LazyLock<String> =
    LazyLock::new(|| base_html(&render_categories(generate_categories())));

pub static INDEX: LazyLock<String> = LazyLock::new(|| {
    let check_paths = CONFIG_CHECKS
        .iter()
        .map(|path| path.canonicalize().unwrap_or(path.clone()))
        .fold(String::new(), |mut output, path| {
            let _ = write!(output, "<li><code>{path:?}</code></li>");
            output
        });

    let active_config = CONFIG
        .path
        .as_ref()
        .map(|path| {
            path.canonicalize()
                .unwrap_or(path.clone())
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or("None detected, using defaults".to_string());

    let port = CONFIG.port;
    let default = &CONFIG.default_engine.name;

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
        <pre>{EXAMPLE_CONFIG}</pre>
        <p>Configuration files are read in this order:</p>
        <ul>{check_paths}</ul>
    "#
    ))
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
    (
        engine.url.replace("{s}", ""),
        EngineDescription {
            name: engine.name.clone(),
            shortcuts: shortcuts
                .into_iter()
                .map(|s| format!("!{s}"))
                .collect::<Vec<_>>()
                .join(", "),
        },
    )
}

fn render_categories(categories: Vec<(String, Category)>) -> String {
    let mut output = String::new();

    output.push_str(
        r#"
        <p><i><a href="/">Back to Main Page</a></i></p>
        <hr>
        <h3>Categories</h3>
        <ol>
    "#,
    );

    for (category, subcategories) in &categories {
        let category_id = category.replace(' ', "_");
        write!(output, "<li><a href=\"#{category_id}\">{category}</a><ul>").unwrap();

        for (subcategory, _) in subcategories.iter().filter(|(s, _)| !s.is_empty()) {
            write!(
                output,
                "<li><a href=\"#{category_id}_{}\">{subcategory}</a></li>",
                subcategory.replace(' ', "_")
            )
            .unwrap();
        }

        output.push_str("</ul></li>");
    }

    output.push_str("</ol>");

    for (category, subcategories) in categories {
        let category_id = category.replace(' ', "_");
        write!(output, "<hr><h3 id=\"{category_id}\">{category}</h3>").unwrap();

        for (subcategory, engines) in subcategories {
            let subcategory_id = format!("{category_id}_{}", subcategory.replace(' ', "_"));
            write!(output, "<h4 id=\"{subcategory_id}\">{subcategory}</h4>").unwrap();

            output.push_str("<ul>");

            for (url, engine) in engines {
                write!(
                    output,
                    "<li><a href=\"{url}\">{}</a>: {}</li>",
                    engine.name, engine.shortcuts
                )
                .unwrap();
            }

            output.push_str("</ul>");
        }
    }

    output
}

pub(crate) fn base_html(content: &str) -> String {
    format!(
        r#"
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
    "#
    )
}
