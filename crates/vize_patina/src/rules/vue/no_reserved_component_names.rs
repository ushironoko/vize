//! vue/no-reserved-component-names
//!
//! Disallow the use of reserved names as component names.
//!
//! HTML element names, SVG element names, and Vue built-in component names
//! should not be used as component names.
//!
//! ## Examples
//!
//! ### Invalid (in script)
//! ```vue
//! export default { name: 'div' }
//! export default { name: 'slot' }
//! export default { name: 'component' }
//! ```
//!
//! ### Invalid (in template - component usage)
//! ```vue
//! <div></div> <!-- This is fine as HTML -->
//! <Div></Div> <!-- PascalCase component named 'Div' conflicts with div -->
//! ```
//!
//! ### Valid
//! ```vue
//! export default { name: 'MyComponent' }
//! export default { name: 'AppHeader' }
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::is_html_tag;
use vize_croquis::builtins::is_builtin_component;
use vize_relief::ast::{ElementNode, ElementType};

static META: RuleMeta = RuleMeta {
    name: "vue/no-reserved-component-names",
    description: "Disallow the use of reserved names as component names",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Reserved names that cannot be used (specific edge cases)
const RESERVED_NAMES: &[&str] = &[
    "annotation-xml",
    "color-profile",
    "font-face",
    "font-face-src",
    "font-face-uri",
    "font-face-format",
    "font-face-name",
    "missing-glyph",
];

/// Disallow reserved component names
pub struct NoReservedComponentNames {
    /// Also disallow HTML element names
    pub disallow_html: bool,
    /// Also disallow Vue built-ins
    pub disallow_vue_builtins: bool,
}

impl Default for NoReservedComponentNames {
    fn default() -> Self {
        Self {
            disallow_html: true,
            disallow_vue_builtins: true,
        }
    }
}

impl Rule for NoReservedComponentNames {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Only check components (PascalCase or kebab-case custom elements)
        if element.tag_type != ElementType::Component {
            return;
        }

        let tag = element.tag.as_str();
        let tag_lower = tag.to_lowercase();

        // Check against reserved names
        if RESERVED_NAMES.contains(&tag_lower.as_str()) {
            ctx.error_with_help(
                ctx.t_fmt("vue/no-reserved-component-names.message", &[("name", tag)]),
                &element.loc,
                ctx.t("vue/no-reserved-component-names.help"),
            );
            return;
        }

        // Check against HTML elements
        if self.disallow_html && is_html_tag(&tag_lower) {
            ctx.error_with_help(
                ctx.t_fmt("vue/no-reserved-component-names.message", &[("name", tag)]),
                &element.loc,
                ctx.t("vue/no-reserved-component-names.help"),
            );
            return;
        }

        // Check against Vue built-ins
        if self.disallow_vue_builtins
            && (is_builtin_component(&tag_lower) || is_builtin_component(tag))
        {
            ctx.error_with_help(
                ctx.t_fmt("vue/no-reserved-component-names.message", &[("name", tag)]),
                &element.loc,
                ctx.t("vue/no-reserved-component-names.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoReservedComponentNames::default()));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_custom_component() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent></MyComponent>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_html_element() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div></div>"#, "test.vue");
        // HTML elements in lowercase are fine
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_pascalcase_html_name() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<Div></Div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_vue_builtin() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<Component></Component>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
