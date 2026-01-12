//! Catalog and index generation for Art collections.

use super::{CatalogOutput, DocOptions};
use crate::types::{ArtDescriptor, ArtStatus};
use serde::{Deserialize, Serialize};
use vize_carton::FxHashMap;

/// Entry in a component catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    /// Component title.
    pub title: String,

    /// Component description.
    pub description: Option<String>,

    /// Category (e.g., "atoms", "molecules").
    pub category: Option<String>,

    /// Tags for filtering.
    pub tags: Vec<String>,

    /// Component status.
    pub status: ArtStatus,

    /// Number of variants.
    pub variant_count: usize,

    /// Path to the component documentation.
    pub doc_path: String,

    /// Path to the source file.
    pub source_path: String,

    /// Display order.
    pub order: Option<u32>,
}

impl CatalogEntry {
    /// Create a catalog entry from an Art descriptor.
    pub fn from_descriptor(art: &ArtDescriptor<'_>, base_path: &str) -> Self {
        let slug = slugify(art.metadata.title);
        let doc_path = if base_path.is_empty() {
            format!("{}.md", slug)
        } else {
            format!("{}/{}.md", base_path.trim_end_matches('/'), slug)
        };

        Self {
            title: art.metadata.title.to_string(),
            description: art.metadata.description.map(|s| s.to_string()),
            category: art.metadata.category.map(|s| s.to_string()),
            tags: art.metadata.tags.iter().map(|s| s.to_string()).collect(),
            status: art.metadata.status,
            variant_count: art.variants.len(),
            doc_path,
            source_path: art.filename.to_string(),
            order: art.metadata.order,
        }
    }
}

/// Generate a complete component catalog.
///
/// Creates a Markdown page with:
/// - Overview statistics
/// - Components grouped by category
/// - Alphabetical listing
pub fn generate_catalog(entries: &[CatalogEntry], options: &DocOptions) -> CatalogOutput {
    let mut md = String::with_capacity(8192);

    // Title
    let title = options.title.as_deref().unwrap_or("Component Catalog");
    md.push_str("# ");
    md.push_str(title);
    md.push_str("\n\n");

    // Statistics
    let categories = collect_categories(entries);
    let tags = collect_tags(entries);

    md.push_str(&format!(
        "> **{}** components across **{}** categories\n\n",
        entries.len(),
        categories.len()
    ));

    // Quick links to categories
    if !categories.is_empty() {
        md.push_str("## Categories\n\n");
        for category in &categories {
            let anchor = slugify(category);
            md.push_str(&format!("- [{}](#{})\n", category, anchor));
        }
        md.push('\n');
    }

    // Components by category
    let by_category = group_by_category(entries);

    for category in &categories {
        md.push_str(&format!("## {}\n\n", category));

        if let Some(category_entries) = by_category.get(category.as_str()) {
            md.push_str(&generate_component_table(category_entries));
        }
    }

    // Uncategorized
    let uncategorized: Vec<_> = entries.iter().filter(|e| e.category.is_none()).collect();
    if !uncategorized.is_empty() {
        md.push_str("## Uncategorized\n\n");
        md.push_str(&generate_component_table(&uncategorized));
    }

    CatalogOutput {
        markdown: md,
        filename: "README.md".to_string(),
        component_count: entries.len(),
        categories,
        tags,
    }
}

/// Generate an index page for a specific category.
pub fn generate_category_index(
    entries: &[CatalogEntry],
    category: &str,
    options: &DocOptions,
) -> CatalogOutput {
    let mut md = String::with_capacity(4096);

    let filtered: Vec<_> = entries
        .iter()
        .filter(|e| e.category.as_deref() == Some(category))
        .collect();

    // Title
    md.push_str("# ");
    md.push_str(category);
    md.push_str("\n\n");

    md.push_str(&format!("> **{}** components\n\n", filtered.len()));

    // Component table
    md.push_str(&generate_component_table(&filtered));

    // Tags in this category
    let tags = collect_tags_from_entries(&filtered);
    if !tags.is_empty() && options.include_metadata {
        md.push_str("## Tags\n\n");
        for tag in &tags {
            md.push_str(&format!("- `{}`\n", tag));
        }
        md.push('\n');
    }

    CatalogOutput {
        markdown: md,
        filename: format!("{}.md", slugify(category)),
        component_count: filtered.len(),
        categories: vec![category.to_string()],
        tags,
    }
}

/// Generate a tags index page.
pub fn generate_tags_index(entries: &[CatalogEntry], _options: &DocOptions) -> CatalogOutput {
    let mut md = String::with_capacity(4096);

    md.push_str("# Tags Index\n\n");

    // Group by tag
    let by_tag = group_by_tag(entries);
    let mut tags: Vec<_> = by_tag.keys().collect();
    tags.sort();

    md.push_str(&format!("> **{}** tags\n\n", tags.len()));

    // Tag cloud / list
    md.push_str("## All Tags\n\n");
    for tag in &tags {
        let count = by_tag.get(*tag).map(|v| v.len()).unwrap_or(0);
        let anchor = slugify(tag);
        md.push_str(&format!("- [`{}`](#{}) ({})\n", tag, anchor, count));
    }
    md.push('\n');

    // Components by tag
    for tag in &tags {
        md.push_str(&format!("## {}\n\n", tag));

        if let Some(tag_entries) = by_tag.get(*tag) {
            md.push_str("| Component | Category | Variants |\n");
            md.push_str("|-----------|----------|----------|\n");
            for entry in tag_entries {
                md.push_str(&format!(
                    "| [{}]({}) | {} | {} |\n",
                    entry.title,
                    entry.doc_path,
                    entry.category.as_deref().unwrap_or("-"),
                    entry.variant_count
                ));
            }
            md.push('\n');
        }
    }

    let all_tags: Vec<String> = tags.iter().map(|s| s.to_string()).collect();

    CatalogOutput {
        markdown: md,
        filename: "tags.md".to_string(),
        component_count: entries.len(),
        categories: vec![],
        tags: all_tags,
    }
}

/// Generate a component table in Markdown.
fn generate_component_table(entries: &[&CatalogEntry]) -> String {
    let mut md = String::new();

    md.push_str("| Component | Description | Variants | Status |\n");
    md.push_str("|-----------|-------------|----------|--------|\n");

    // Sort by order, then by title
    let mut sorted: Vec<_> = entries.iter().collect();
    sorted.sort_by(|a, b| match (a.order, b.order) {
        (Some(a_order), Some(b_order)) => a_order.cmp(&b_order),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.title.cmp(&b.title),
    });

    for entry in sorted {
        let desc = entry
            .description
            .as_deref()
            .unwrap_or("-")
            .chars()
            .take(50)
            .collect::<String>();
        let desc = if entry.description.as_ref().map(|d| d.len()).unwrap_or(0) > 50 {
            format!("{}...", desc)
        } else {
            desc
        };

        let status = match entry.status {
            ArtStatus::Ready => "✅",
            ArtStatus::Draft => "🚧",
            ArtStatus::Deprecated => "⚠️",
        };

        md.push_str(&format!(
            "| [{}]({}) | {} | {} | {} |\n",
            entry.title, entry.doc_path, desc, entry.variant_count, status
        ));
    }

    md.push('\n');
    md
}

/// Collect all unique categories.
fn collect_categories(entries: &[CatalogEntry]) -> Vec<String> {
    let mut categories: Vec<_> = entries.iter().filter_map(|e| e.category.clone()).collect();
    categories.sort();
    categories.dedup();
    categories
}

/// Collect all unique tags.
fn collect_tags(entries: &[CatalogEntry]) -> Vec<String> {
    let mut tags: Vec<_> = entries.iter().flat_map(|e| e.tags.clone()).collect();
    tags.sort();
    tags.dedup();
    tags
}

/// Collect tags from a slice of entry references.
fn collect_tags_from_entries(entries: &[&CatalogEntry]) -> Vec<String> {
    let mut tags: Vec<_> = entries.iter().flat_map(|e| e.tags.clone()).collect();
    tags.sort();
    tags.dedup();
    tags
}

/// Group entries by category.
fn group_by_category(entries: &[CatalogEntry]) -> FxHashMap<&str, Vec<&CatalogEntry>> {
    let mut map: FxHashMap<&str, Vec<&CatalogEntry>> = FxHashMap::default();
    for entry in entries {
        if let Some(ref category) = entry.category {
            map.entry(category.as_str()).or_default().push(entry);
        }
    }
    map
}

/// Group entries by tag.
fn group_by_tag(entries: &[CatalogEntry]) -> FxHashMap<&str, Vec<&CatalogEntry>> {
    let mut map: FxHashMap<&str, Vec<&CatalogEntry>> = FxHashMap::default();
    for entry in entries {
        for tag in &entry.tags {
            map.entry(tag.as_str()).or_default().push(entry);
        }
    }
    map
}

/// Convert a string to a URL-safe slug.
#[inline]
fn slugify(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(title: &str, category: Option<&str>, tags: &[&str]) -> CatalogEntry {
        CatalogEntry {
            title: title.to_string(),
            description: Some(format!("{} description", title)),
            category: category.map(|s| s.to_string()),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            status: ArtStatus::Ready,
            variant_count: 2,
            doc_path: format!("{}.md", slugify(title)),
            source_path: format!("{}.art.vue", slugify(title)),
            order: None,
        }
    }

    #[test]
    fn test_generate_catalog() {
        let entries = vec![
            make_entry("Button", Some("atoms"), &["ui", "input"]),
            make_entry("Card", Some("molecules"), &["layout"]),
            make_entry("Icon", Some("atoms"), &["ui"]),
        ];

        let output = generate_catalog(&entries, &DocOptions::default());

        assert!(output.markdown.contains("# Component Catalog"));
        assert!(output.markdown.contains("## atoms"));
        assert!(output.markdown.contains("## molecules"));
        assert_eq!(output.component_count, 3);
        assert_eq!(output.categories.len(), 2);
    }

    #[test]
    fn test_generate_tags_index() {
        let entries = vec![
            make_entry("Button", Some("atoms"), &["ui", "input"]),
            make_entry("Input", Some("atoms"), &["ui", "input", "form"]),
        ];

        let output = generate_tags_index(&entries, &DocOptions::default());

        assert!(output.markdown.contains("# Tags Index"));
        assert!(output.markdown.contains("`ui`"));
        assert!(output.markdown.contains("`input`"));
        assert!(output.markdown.contains("`form`"));
    }

    #[test]
    fn test_collect_categories() {
        let entries = vec![
            make_entry("A", Some("atoms"), &[]),
            make_entry("B", Some("molecules"), &[]),
            make_entry("C", Some("atoms"), &[]),
        ];

        let categories = collect_categories(&entries);
        assert_eq!(categories, vec!["atoms", "molecules"]);
    }
}
