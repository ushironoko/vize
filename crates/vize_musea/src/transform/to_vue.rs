//! Transform Art to executable Vue components.
//!
//! This module generates Vue components that can be used to render
//! Art variants in the Musea gallery.

use crate::types::ArtDescriptor;

/// Output of Vue transformation.
#[derive(Debug, Clone)]
pub struct VueOutput {
    /// Generated Vue component code
    pub code: String,
    /// Metadata module code (for sidebar/gallery)
    pub metadata_code: String,
}

/// Transform an Art descriptor to executable Vue component modules.
///
/// Generates:
/// 1. A main component that renders all variants
/// 2. Individual variant components
/// 3. A metadata module for the gallery UI
pub fn transform_to_vue(art: &ArtDescriptor<'_>) -> VueOutput {
    let main_code = generate_main_component(art);
    let metadata_code = generate_metadata_module(art);

    VueOutput {
        code: main_code,
        metadata_code,
    }
}

/// Generate the main component that exposes variants.
fn generate_main_component(art: &ArtDescriptor<'_>) -> String {
    let mut code = String::new();

    // Imports
    code.push_str("import { defineComponent, h, reactive, markRaw } from 'vue';\n");

    // Import target component
    if let Some(ref component_path) = art.metadata.component {
        code.push_str(&format!(
            "import TargetComponent from '{}';\n",
            component_path
        ));
    }

    // Re-export script imports if present
    if let Some(ref script) = art.script_setup {
        for line in script.content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                code.push_str(trimmed);
                code.push('\n');
            }
        }
    }

    code.push('\n');

    // Export metadata
    code.push_str(&format!(
        "export const metadata = {};\n\n",
        generate_metadata_json(art)
    ));

    // Export variants array
    code.push_str("export const variants = [\n");
    for variant in &art.variants {
        let args_json = serde_json::to_string(&variant.args).unwrap_or_else(|_| "{}".to_string());

        code.push_str(&format!(
            "  {{ name: '{}', isDefault: {}, args: {}, skipVrt: {} }},\n",
            escape_js_string(variant.name),
            variant.is_default,
            args_json,
            variant.skip_vrt
        ));
    }
    code.push_str("];\n\n");

    // Generate variant components
    for (i, variant) in art.variants.iter().enumerate() {
        let component_name = to_pascal_case(variant.name);
        let args_json = serde_json::to_string(&variant.args).unwrap_or_else(|_| "{}".to_string());

        code.push_str(&format!(
            r#"export const {} = defineComponent({{
  name: '{}',
  setup(props, {{ attrs }}) {{
    const defaultArgs = {};
    const args = reactive({{ ...defaultArgs, ...attrs }});
    return () => h('div', {{ class: 'musea-variant', 'data-variant': '{}' }}, [
      {}
    ]);
  }}
}});

"#,
            component_name,
            component_name,
            args_json,
            escape_js_string(variant.name),
            generate_render_expression(variant.template, art),
        ));

        // Mark as default if applicable
        if variant.is_default {
            code.push_str(&format!("{}.isDefault = true;\n\n", component_name));
        }

        // Store index for ordering
        code.push_str(&format!("{}.variantIndex = {};\n\n", component_name, i));
    }

    // Default export - the gallery component
    code.push_str(
        r#"export default defineComponent({
  name: 'ArtGallery',
  props: {
    variant: { type: String, default: null },
    interactive: { type: Boolean, default: false },
  },
  setup(props) {
    const variantComponents = {
"#,
    );

    for variant in &art.variants {
        let component_name = to_pascal_case(variant.name);
        code.push_str(&format!(
            "      '{}': {},\n",
            escape_js_string(variant.name),
            component_name
        ));
    }

    code.push_str(
        r#"    };

    return () => {
      if (props.variant && variantComponents[props.variant]) {
        return h(variantComponents[props.variant]);
      }
      // Render all variants
      return h('div', { class: 'musea-gallery' },
        variants.map(v => h(variantComponents[v.name], { key: v.name }))
      );
    };
  }
});
"#,
    );

    code
}

/// Generate a render expression for a variant template.
fn generate_render_expression(template: &str, art: &ArtDescriptor<'_>) -> String {
    // For now, we'll create a simple render using the template as a component
    // In a real implementation, this would compile the template to a render function

    // Check if template uses the target component
    let uses_target = art.metadata.component.is_some();

    if uses_target {
        // Simple case: render the target component with interpolated args
        format!(
            "h(TargetComponent, args, () => `{}`)",
            escape_template_literal(template)
        )
    } else {
        // Render raw template content (for custom components)
        format!(
            "h('div', {{ innerHTML: `{}` }})",
            escape_template_literal(template)
        )
    }
}

/// Generate metadata JSON for the Art.
fn generate_metadata_json(art: &ArtDescriptor<'_>) -> String {
    let mut json = String::new();
    json.push_str("{\n");
    json.push_str(&format!(
        "  title: '{}',\n",
        escape_js_string(art.metadata.title)
    ));

    if let Some(desc) = art.metadata.description {
        json.push_str(&format!("  description: '{}',\n", escape_js_string(desc)));
    }

    if let Some(component) = art.metadata.component {
        json.push_str(&format!(
            "  component: '{}',\n",
            escape_js_string(component)
        ));
    }

    if let Some(category) = art.metadata.category {
        json.push_str(&format!("  category: '{}',\n", escape_js_string(category)));
    }

    if !art.metadata.tags.is_empty() {
        let tags: Vec<String> = art
            .metadata
            .tags
            .iter()
            .map(|t| format!("'{}'", escape_js_string(t)))
            .collect();
        json.push_str(&format!("  tags: [{}],\n", tags.join(", ")));
    }

    json.push_str(&format!(
        "  status: '{}',\n",
        status_to_string(art.metadata.status)
    ));

    if let Some(order) = art.metadata.order {
        json.push_str(&format!("  order: {},\n", order));
    }

    json.push_str(&format!("  variantCount: {},\n", art.variants.len()));

    json.push('}');
    json
}

/// Generate metadata module for gallery sidebar.
fn generate_metadata_module(art: &ArtDescriptor<'_>) -> String {
    let mut code = String::new();

    code.push_str("// Auto-generated metadata module\n");
    code.push_str(&format!(
        "export const metadata = {};\n\n",
        generate_metadata_json(art)
    ));

    code.push_str("export const variants = [\n");
    for variant in &art.variants {
        code.push_str("  {\n");
        code.push_str(&format!(
            "    name: '{}',\n",
            escape_js_string(variant.name)
        ));
        code.push_str(&format!("    isDefault: {},\n", variant.is_default));
        code.push_str(&format!("    skipVrt: {},\n", variant.skip_vrt));

        if let Some(ref viewport) = variant.viewport {
            code.push_str(&format!(
                "    viewport: {{ width: {}, height: {} }},\n",
                viewport.width, viewport.height
            ));
        }

        code.push_str("  },\n");
    }
    code.push_str("];\n");

    code
}

/// Convert status enum to string.
fn status_to_string(status: crate::types::ArtStatus) -> &'static str {
    match status {
        crate::types::ArtStatus::Draft => "draft",
        crate::types::ArtStatus::Ready => "ready",
        crate::types::ArtStatus::Deprecated => "deprecated",
    }
}

/// Convert string to PascalCase.
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
fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Escape content for JavaScript template literal.
fn escape_template_literal(s: &str) -> String {
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
    fn test_transform_to_vue_basic() {
        let allocator = Bump::new();
        let source = r#"
<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button>Click me</Button>
  </variant>
</art>
"#;

        let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
        let output = transform_to_vue(&art);

        assert!(output.code.contains("import { defineComponent"));
        assert!(output.code.contains("import TargetComponent"));
        assert!(output.code.contains("export const Primary"));
        assert!(output.code.contains("export const metadata"));
        assert!(output.metadata_code.contains("title: 'Button'"));
    }

    #[test]
    fn test_transform_multiple_variants() {
        let allocator = Bump::new();
        let source = r#"
<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button variant="primary">Primary</Button>
  </variant>
  <variant name="Secondary">
    <Button variant="secondary">Secondary</Button>
  </variant>
</art>
"#;

        let art = parse_art(&allocator, source, ArtParseOptions::default()).unwrap();
        let output = transform_to_vue(&art);

        assert!(output.code.contains("export const Primary"));
        assert!(output.code.contains("export const Secondary"));
        assert!(output.code.contains("Primary.isDefault = true"));
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("primary"), "Primary");
        assert_eq!(to_pascal_case("with icon"), "WithIcon");
        assert_eq!(to_pascal_case("my-variant"), "MyVariant");
    }

    #[test]
    fn test_escape_template_literal() {
        assert_eq!(escape_template_literal("`code`"), "\\`code\\`");
        assert_eq!(escape_template_literal("${var}"), "\\${var}");
    }
}
