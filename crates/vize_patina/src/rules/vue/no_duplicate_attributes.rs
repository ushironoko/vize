//! vue/no-duplicate-attributes
//!
//! Disallow duplicate attributes on the same element.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div id="foo" id="bar"></div>
//! <div :class="foo" class="bar"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div id="foo"></div>
//! <div :class="foo"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use rustc_hash::FxHashSet;
use vize_relief::ast::{ElementNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-duplicate-attributes",
    description: "Disallow duplicate attributes on the same element",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow duplicate attributes
pub struct NoDuplicateAttributes {
    /// Allow :class and class to coexist
    pub allow_coexist_class: bool,
    /// Allow :style and style to coexist
    pub allow_coexist_style: bool,
}

impl Default for NoDuplicateAttributes {
    fn default() -> Self {
        Self {
            allow_coexist_class: true,
            allow_coexist_style: true,
        }
    }
}

impl Rule for NoDuplicateAttributes {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        let mut seen_attrs: FxHashSet<std::string::String> = FxHashSet::default();
        let mut seen_directives: FxHashSet<std::string::String> = FxHashSet::default();

        for prop in element.props.iter() {
            match prop {
                PropNode::Attribute(attr) => {
                    let name = attr.name.as_str().to_lowercase();

                    // Check for duplicate static attributes
                    if seen_attrs.contains(&name) {
                        ctx.error(format!("Duplicate attribute '{}'", attr.name), &attr.loc);
                    } else {
                        seen_attrs.insert(name.clone());
                    }

                    // Check for coexistence with directives (unless allowed)
                    if !self.allow_coexist_class
                        && name == "class"
                        && seen_directives.contains("class")
                    {
                        ctx.error(
                            "Duplicate attribute 'class' (already defined as v-bind:class)",
                            &attr.loc,
                        );
                    }
                    if !self.allow_coexist_style
                        && name == "style"
                        && seen_directives.contains("style")
                    {
                        ctx.error(
                            "Duplicate attribute 'style' (already defined as v-bind:style)",
                            &attr.loc,
                        );
                    }
                }
                PropNode::Directive(dir) => {
                    // Handle v-bind directives
                    if dir.name.as_str() == "bind" {
                        if let Some(ref arg) = dir.arg {
                            let arg_name = get_expression_content(arg).to_lowercase();

                            // Check for duplicate directives
                            if seen_directives.contains(&arg_name) {
                                ctx.error(
                                    format!("Duplicate directive 'v-bind:{}'", arg_name),
                                    &dir.loc,
                                );
                            } else {
                                seen_directives.insert(arg_name.clone());
                            }

                            // Check for coexistence with static attributes (unless allowed)
                            if !self.allow_coexist_class
                                && arg_name == "class"
                                && seen_attrs.contains("class")
                            {
                                ctx.error(
                                    "Duplicate directive 'v-bind:class' (already defined as class attribute)",
                                    &dir.loc,
                                );
                            }
                            if !self.allow_coexist_style
                                && arg_name == "style"
                                && seen_attrs.contains("style")
                            {
                                ctx.error(
                                    "Duplicate directive 'v-bind:style' (already defined as style attribute)",
                                    &dir.loc,
                                );
                            }
                        }
                    }
                    // Handle v-on directives
                    else if dir.name.as_str() == "on" {
                        if let Some(ref arg) = dir.arg {
                            let event_key = format!("on:{}", get_expression_content(arg));
                            if seen_directives.contains(&event_key) {
                                ctx.error(
                                    format!(
                                        "Duplicate event handler 'v-on:{}'",
                                        get_expression_content(arg)
                                    ),
                                    &dir.loc,
                                );
                            } else {
                                seen_directives.insert(event_key);
                            }
                        }
                    }
                    // Handle v-model
                    else if dir.name.as_str() == "model" {
                        let model_key = if let Some(ref arg) = dir.arg {
                            format!("model:{}", get_expression_content(arg))
                        } else {
                            "model:modelValue".to_string()
                        };
                        if seen_directives.contains(&model_key) {
                            ctx.error("Duplicate v-model directive", &dir.loc);
                        } else {
                            seen_directives.insert(model_key);
                        }
                    }
                }
            }
        }
    }
}

/// Get content from ExpressionNode
fn get_expression_content(expr: &vize_relief::ast::ExpressionNode) -> String {
    match expr {
        vize_relief::ast::ExpressionNode::Simple(s) => s.content.to_string(),
        vize_relief::ast::ExpressionNode::Compound(_) => "<dynamic>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDuplicateAttributes::default()));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_unique_attributes() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div id="foo" class="bar"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_duplicate_id() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div id="foo" id="bar"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
        assert!(result.diagnostics[0].message.contains("Duplicate"));
    }

    #[test]
    fn test_valid_class_coexist() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="foo" class="bar"></div>"#, "test.vue");
        // Default allows coexistence
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_duplicate_v_bind() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :id="foo" :id="bar"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
