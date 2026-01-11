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
use vize_relief::ast::{ElementNode, ElementType};

static META: RuleMeta = RuleMeta {
    name: "vue/no-reserved-component-names",
    description: "Disallow the use of reserved names as component names",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Reserved HTML element names
const HTML_ELEMENTS: &[&str] = &[
    "html",
    "body",
    "base",
    "head",
    "link",
    "meta",
    "style",
    "title",
    "address",
    "article",
    "aside",
    "footer",
    "header",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "main",
    "nav",
    "section",
    "blockquote",
    "dd",
    "div",
    "dl",
    "dt",
    "figcaption",
    "figure",
    "hr",
    "li",
    "ol",
    "p",
    "pre",
    "ul",
    "a",
    "abbr",
    "b",
    "bdi",
    "bdo",
    "br",
    "cite",
    "code",
    "data",
    "dfn",
    "em",
    "i",
    "kbd",
    "mark",
    "q",
    "rp",
    "rt",
    "ruby",
    "s",
    "samp",
    "small",
    "span",
    "strong",
    "sub",
    "sup",
    "time",
    "u",
    "var",
    "wbr",
    "area",
    "audio",
    "img",
    "map",
    "track",
    "video",
    "embed",
    "iframe",
    "object",
    "param",
    "picture",
    "portal",
    "source",
    "svg",
    "math",
    "canvas",
    "noscript",
    "script",
    "del",
    "ins",
    "caption",
    "col",
    "colgroup",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "button",
    "datalist",
    "fieldset",
    "form",
    "input",
    "label",
    "legend",
    "meter",
    "optgroup",
    "option",
    "output",
    "progress",
    "select",
    "textarea",
    "details",
    "dialog",
    "menu",
    "summary",
    "slot",
    "template",
];

/// Vue built-in component names
const VUE_BUILTINS: &[&str] = &[
    "component",
    "transition",
    "transition-group",
    "keep-alive",
    "slot",
    "teleport",
    "suspense",
];

/// Reserved names that cannot be used
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
                format!(
                    "'{}' is a reserved name and cannot be used as a component name",
                    tag
                ),
                &element.loc,
                "Choose a different component name",
            );
            return;
        }

        // Check against HTML elements
        if self.disallow_html && HTML_ELEMENTS.contains(&tag_lower.as_str()) {
            ctx.error_with_help(
                format!("'{}' conflicts with an HTML element name", tag),
                &element.loc,
                "Choose a component name that doesn't conflict with HTML elements",
            );
            return;
        }

        // Check against Vue built-ins
        if self.disallow_vue_builtins && VUE_BUILTINS.contains(&tag_lower.as_str()) {
            ctx.error_with_help(
                format!("'{}' is a Vue built-in component name", tag),
                &element.loc,
                "Choose a different component name that doesn't conflict with Vue built-ins",
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
        assert!(result.diagnostics[0].message.contains("conflicts"));
    }

    #[test]
    fn test_invalid_vue_builtin() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<Component></Component>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
