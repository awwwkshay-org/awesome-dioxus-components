//! `catalog fetch base-ui`: inventory + per-part explicit props for every
//! component at <https://base-ui.com/react/components>.
//!
//! Base UI has no public API/JSON export, only rendered docs pages, so this
//! is genuine (if brittle) HTML scraping. Every prop's API-reference entry
//! is a `<details class="AccordionItem">` whose `<summary>` has
//! `id="<PartName>-<propName>"` and `aria-label="Prop: <propName>, ..."` --
//! confirmed by hand against the live Dialog page. If that structure ever
//! changes, [`fetch`] fails loudly (a `catalog fetch` error) rather than
//! silently writing an empty snapshot; `primitive-compat` reads only the
//! last-committed snapshot, so a broken scrape never blocks `check`/CI.

use std::time::Duration;

use scraper::{Html, Selector};

use super::schema::{CatalogEntry, CatalogSnapshot, PartEntry, Prop, PropsSource};

const COMPONENTS_INDEX_URL: &str = "https://base-ui.com/react/components";

pub fn fetch(_revision: Option<&str>) -> Result<CatalogSnapshot, String> {
    let slugs = fetch_component_slugs()?;
    if slugs.is_empty() {
        return Err(format!(
            "found no component links at {COMPONENTS_INDEX_URL}; the index page markup may have changed"
        ));
    }

    let mut entries = Vec::with_capacity(slugs.len());
    for slug in &slugs {
        entries.push(fetch_component_entry(slug)?);
    }

    Ok(CatalogSnapshot {
        axis: "base-ui".to_string(),
        source: COMPONENTS_INDEX_URL.to_string(),
        revision: crate::now_utc(),
        refreshed_at: crate::today(),
        entries,
    })
}

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("adico-xtask/1.0")
        .build()
        .map_err(|error| format!("cannot build HTTP client: {error}"))
}

fn get_html(url: &str) -> Result<Html, String> {
    let client = http_client()?;
    let response = client
        .get(url)
        .send()
        .map_err(|error| format!("cannot reach {url}: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("{url} returned {}", response.status()));
    }
    let body = response
        .text()
        .map_err(|error| format!("cannot read response body from {url}: {error}"))?;
    Ok(Html::parse_document(&body))
}

fn fetch_component_slugs() -> Result<Vec<String>, String> {
    let document = get_html(COMPONENTS_INDEX_URL)?;
    let link_selector = Selector::parse(r#"a[href^="/react/components/"]"#)
        .map_err(|error| format!("invalid selector: {error:?}"))?;

    let mut slugs: Vec<String> = document
        .select(&link_selector)
        .filter_map(|element| element.value().attr("href"))
        .filter_map(|href| href.strip_prefix("/react/components/"))
        .filter(|slug| !slug.is_empty() && !slug.contains('/'))
        .map(str::to_string)
        .collect();
    slugs.sort();
    slugs.dedup();
    Ok(slugs)
}

fn fetch_component_entry(slug: &str) -> Result<CatalogEntry, String> {
    let url = format!("{COMPONENTS_INDEX_URL}/{slug}");
    let document = get_html(&url)?;
    parse_component_html(slug, &document).map_err(|error| format!("{url}: {error}"))
}

fn parse_component_html(slug: &str, document: &Html) -> Result<CatalogEntry, String> {
    let heading_selector =
        Selector::parse("h1.MdH1").map_err(|error| format!("invalid selector: {error:?}"))?;
    let item_selector = Selector::parse("details.AccordionItem")
        .map_err(|error| format!("invalid selector: {error:?}"))?;
    let summary_selector =
        Selector::parse("summary").map_err(|error| format!("invalid selector: {error:?}"))?;
    let name_selector = Selector::parse(".ReferenceNameCell code")
        .map_err(|error| format!("invalid selector: {error:?}"))?;
    let type_selector = Selector::parse(".ReferenceTypeCell")
        .map_err(|error| format!("invalid selector: {error:?}"))?;
    let default_selector = Selector::parse(".ReferenceDefaultCell")
        .map_err(|error| format!("invalid selector: {error:?}"))?;
    let description_selector = Selector::parse(".ReferenceDescription")
        .map_err(|error| format!("invalid selector: {error:?}"))?;

    let mut props_by_part: std::collections::BTreeMap<String, Vec<Prop>> =
        std::collections::BTreeMap::new();

    for item in document.select(&item_selector) {
        let Some(summary) = item.select(&summary_selector).next() else {
            continue;
        };
        let is_prop = summary
            .value()
            .attr("aria-label")
            .is_some_and(|label| label.starts_with("Prop: "));
        if !is_prop {
            continue;
        }
        let Some(id) = summary.value().attr("id") else {
            continue;
        };
        let Some((part_prefix, prop_name_from_id)) = id.split_once('-') else {
            continue;
        };

        let name = summary
            .select(&name_selector)
            .next()
            .map(element_text)
            .filter(|text| !text.is_empty())
            .unwrap_or_else(|| prop_name_from_id.to_string());
        let type_name = summary
            .select(&type_selector)
            .next()
            .map(element_text)
            .unwrap_or_default();
        let default = item
            .select(&default_selector)
            .next()
            .map(element_text)
            .filter(|text| !text.is_empty() && text != "\u{2014}");
        let description = item
            .select(&description_selector)
            .next()
            .map(element_text)
            .filter(|text| !text.is_empty());

        props_by_part
            .entry(part_prefix.to_string())
            .or_default()
            .push(Prop {
                name,
                type_name,
                default,
                description,
            });
    }

    if props_by_part.is_empty() {
        return Err("found no props; the API reference markup may have changed".to_string());
    }

    let component_prefix = super::case::pascal_case(slug);
    let mut parts: Vec<PartEntry> = props_by_part
        .into_iter()
        .map(|(part_prefix, props)| {
            let part_id = super::case::part_id_for(&component_prefix, &part_prefix);
            PartEntry {
                id: part_id,
                composition: Vec::new(),
                props_source: PropsSource::Explicit { props },
            }
        })
        .collect();
    parts.sort_by(|left, right| left.id.cmp(&right.id));

    let name = document
        .select(&heading_selector)
        .next()
        .map(element_text)
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| title_case_from_slug(slug));

    Ok(CatalogEntry {
        id: slug.to_string(),
        name,
        parts,
    })
}

fn element_text(element: scraper::ElementRef) -> String {
    element.text().collect::<String>().trim().to_string()
}

fn title_case_from_slug(slug: &str) -> String {
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIALOG_FIXTURE: &str = r#"
        <details class="AccordionItem">
          <summary id="DialogRoot-modal" aria-label="Prop: modal, type:  (default: true)">
            <span class="ReferenceNameCell"><code>modal</code></span>
            <span class="ReferenceTypeCell">boolean</span>
            <span class="ReferenceDefaultCell">true</span>
          </summary>
          <div class="AccordionPanel">
            <dd class="ReferenceDescription">Whether the dialog is modal.</dd>
          </div>
        </details>
        <details class="AccordionItem">
          <summary id="DialogTrigger-nativeButton" aria-label="Prop: nativeButton, type:  (default: true)">
            <span class="ReferenceNameCell"><code>nativeButton</code></span>
            <span class="ReferenceTypeCell">boolean</span>
            <span class="ReferenceDefaultCell">&#x2014;</span>
          </summary>
        </details>
        <details class="AccordionItem">
          <summary id="DialogRoot-close" aria-label="Method: close, returns: value">
            <span class="ReferenceNameCell"><code>close</code></span>
          </summary>
        </details>
    "#;

    #[test]
    fn parses_props_grouped_by_part() {
        let document = Html::parse_fragment(DIALOG_FIXTURE);
        let entry = parse_component_html("dialog", &document).expect("parse succeeds");
        assert_eq!(
            entry.parts.len(),
            2,
            "root and trigger, not the Method entry"
        );

        let root = entry.parts.iter().find(|p| p.id == "root").unwrap();
        let PropsSource::Explicit { props } = &root.props_source else {
            panic!("expected explicit props");
        };
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].name, "modal");
        assert_eq!(props[0].default.as_deref(), Some("true"));
        assert_eq!(
            props[0].description.as_deref(),
            Some("Whether the dialog is modal.")
        );

        let trigger = entry.parts.iter().find(|p| p.id == "trigger").unwrap();
        let PropsSource::Explicit { props } = &trigger.props_source else {
            panic!("expected explicit props");
        };
        assert_eq!(props[0].default, None, "em dash means no default");
    }

    #[test]
    fn errors_loudly_on_malformed_markup() {
        let document = Html::parse_fragment("<div>nothing recognizable here</div>");
        let result = parse_component_html("dialog", &document);
        assert!(
            result.is_err(),
            "must error, not silently write an empty snapshot"
        );
    }
}
