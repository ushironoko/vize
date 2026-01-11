//! Transform Art to Storybook CSF 3.0 format.
//!
//! This module generates Storybook-compatible Component Story Format (CSF) files
//! from Art descriptors.

use crate::types::{ArtDescriptor, ArtVariant, CsfOutput};

/// Transform an Art descriptor to Storybook CSF 3.0 format.
///
/// # Example
///
/// ```ignore
/// use vize_musea::transform::transform_to_csf;
/// use vize_musea::parse::parse_art;
///
/// let source = r#"
/// <art title="Button" component="./Button.vue">
///   <variant name="Primary" default>
///     <Button>Click</Button>
///   </variant>
/// </art>
/// "#;
///
/// let art = parse_art(source, Default::default()).unwrap();
/// let csf = transform_to_csf(&art);
/// ```
pub fn transform_to_csf(art: &ArtDescriptor<'_>) -> CsfOutput {
    let mut output = String::new();

    // Generate imports
    output.push_str(&generate_imports(art));
    output.push('\n');

    // Generate meta (default export)
    output.push_str(&generate_meta(art));
    output.push('\n');

    // Generate stories (named exports)
    for variant in &art.variants {
        output.push_str(&generate_story(variant, art));
        output.push('\n');
    }

    // Determine filename
    let base_name = art
        .filename
        .trim_end_matches(".art.vue")
        .rsplit('/')
        .next()
        .unwrap_or("Component");

    CsfOutput {
        code: output,
        filename: format!("{}.stories.ts", base_name),
    }
}

/// Generate import statements.
fn generate_imports(art: &ArtDescriptor<'_>) -> String {
    let mut imports = String::new();

    // Import from Storybook
    imports.push_str("import type { Meta, StoryObj } from '@storybook/vue3';\n");

    // Import the component
    let component_path = art.metadata.component.unwrap_or("./Component.vue");

    imports.push_str(&format!("import Component from '{}';\n", component_path));

    // Add script imports if present
    if let Some(script) = &art.script_setup {
        // Extract imports from script setup
        for line in script.content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") && !trimmed.contains("Component") {
                imports.push_str(trimmed);
                imports.push('\n');
            }
        }
    }

    imports
}

/// Generate meta (default export).
fn generate_meta(art: &ArtDescriptor<'_>) -> String {
    let mut meta = String::new();

    // Build the title path
    let title = if let Some(ref category) = art.metadata.category {
        format!("{}/{}", category, art.metadata.title)
    } else {
        art.metadata.title.to_string()
    };

    meta.push_str("const meta: Meta<typeof Component> = {\n");
    meta.push_str(&format!("  title: '{}',\n", escape_string(&title)));
    meta.push_str("  component: Component,\n");

    // Add tags
    let mut tags = vec!["autodocs".to_string()];
    for tag in &art.metadata.tags {
        tags.push(tag.to_string());
    }
    meta.push_str(&format!(
        "  tags: [{}],\n",
        tags.iter()
            .map(|t| format!("'{}'", t))
            .collect::<Vec<_>>()
            .join(", ")
    ));

    // Add parameters for description
    if let Some(desc) = art.metadata.description {
        meta.push_str("  parameters: {\n");
        meta.push_str("    docs: {\n");
        meta.push_str("      description: {\n");
        meta.push_str(&format!("        component: '{}',\n", escape_string(desc)));
        meta.push_str("      },\n");
        meta.push_str("    },\n");
        meta.push_str("  },\n");
    }

    meta.push_str("};\n\n");
    meta.push_str("export default meta;\n");
    meta.push_str("type Story = StoryObj<typeof meta>;\n");

    meta
}

/// Generate a story (named export) from a variant.
fn generate_story(variant: &ArtVariant<'_>, _art: &ArtDescriptor<'_>) -> String {
    let mut story = String::new();

    // Convert variant name to PascalCase for export name
    let export_name = to_pascal_case(variant.name);

    story.push_str(&format!("export const {}: Story = {{\n", export_name));

    // Add name if different from export name
    if export_name != variant.name {
        story.push_str(&format!("  name: '{}',\n", escape_string(variant.name)));
    }

    // Add args if present
    if !variant.args.is_empty() {
        story.push_str("  args: {\n");
        for (key, value) in &variant.args {
            let value_str =
                serde_json::to_string(value).unwrap_or_else(|_| "undefined".to_string());
            story.push_str(&format!("    {}: {},\n", key, value_str));
        }
        story.push_str("  },\n");
    }

    // Add render function with template
    story.push_str("  render: (args) => ({\n");
    story.push_str("    components: { Component },\n");
    story.push_str("    setup() {\n");
    story.push_str("      return { args };\n");
    story.push_str("    },\n");

    // Use the variant's template
    let template = variant.template.trim();
    story.push_str(&format!("    template: `{}`,\n", escape_template(template)));

    story.push_str("  }),\n");

    // Add parameters for default story
    if variant.is_default {
        story.push_str("  parameters: {\n");
        story.push_str("    docs: {\n");
        story.push_str("      canvas: { sourceState: 'shown' },\n");
        story.push_str("    },\n");
        story.push_str("  },\n");
    }

    story.push_str("};\n");

    story
}

/// Convert a string to PascalCase.
fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c.is_whitespace() || c == '-' || c == '_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Escape a string for JavaScript string literal.
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Escape a template string for JavaScript template literal.
fn escape_template(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse_art;
    use crate::types::ArtParseOptions;
    use vize_carton::Bump;

    #[test]
    fn test_transform_simple() {
        let allocator = Bump::new();
        let source = r#"
<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button variant="primary">Click me</Button>
  </variant>
</art>
"#;

        let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
        let csf = transform_to_csf(&art);

        assert!(csf.code.contains("import type { Meta, StoryObj }"));
        assert!(csf.code.contains("import Component from './Button.vue'"));
        assert!(csf.code.contains("title: 'Button'"));
        assert!(csf.code.contains("export const Primary: Story"));
        assert!(csf.filename.ends_with(".stories.ts"));
    }

    #[test]
    fn test_transform_with_category() {
        let allocator = Bump::new();
        let source = r#"
<art title="Button" category="atoms" component="./Button.vue">
  <variant name="Default">
    <Button>Click</Button>
  </variant>
</art>
"#;

        let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
        let csf = transform_to_csf(&art);

        assert!(csf.code.contains("title: 'atoms/Button'"));
    }

    #[test]
    fn test_transform_multiple_variants() {
        let allocator = Bump::new();
        let source = r#"
<art title="Button" component="./Button.vue">
  <variant name="Primary">
    <Button variant="primary">Primary</Button>
  </variant>
  <variant name="Secondary">
    <Button variant="secondary">Secondary</Button>
  </variant>
</art>
"#;

        let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
        let csf = transform_to_csf(&art);

        assert!(csf.code.contains("export const Primary: Story"));
        assert!(csf.code.contains("export const Secondary: Story"));
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("primary"), "Primary");
        assert_eq!(to_pascal_case("with icon"), "WithIcon");
        assert_eq!(to_pascal_case("my-button"), "MyButton");
        assert_eq!(to_pascal_case("my_button"), "MyButton");
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("hello"), "hello");
        assert_eq!(escape_string("it's"), "it\\'s");
        assert_eq!(escape_string("line\nbreak"), "line\\nbreak");
    }

    #[test]
    fn test_escape_template() {
        assert_eq!(escape_template("hello"), "hello");
        assert_eq!(escape_template("`code`"), "\\`code\\`");
        assert_eq!(escape_template("${var}"), "\\${var}");
    }
}
