//! vue/valid-v-for
//!
//! Enforce valid `v-for` directives.
//!
//! This rule checks the following:
//! - `v-for` directive has an expression
//! - `v-for` directive's expression is valid (contains "in" or "of")
//! - `v-for` directive doesn't have invalid modifiers
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-for></div>
//! <div v-for=""></div>
//! <div v-for.stop="item in items"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-for="item in items" :key="item.id"></div>
//! <div v-for="(item, index) in items" :key="index"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-for",
    description: "Enforce valid `v-for` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-for directives
pub struct ValidVFor;

impl Rule for ValidVFor {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        // Only check v-for directives
        if directive.name.as_str() != "for" {
            return;
        }

        // Check for modifiers (v-for should not have modifiers)
        if !directive.modifiers.is_empty() {
            ctx.error_with_help(
                "'v-for' directives require no modifier",
                &directive.loc,
                "Remove the modifier from v-for",
            );
            return;
        }

        // Check for argument (v-for should not have an argument like v-for:something)
        if directive.arg.is_some() {
            ctx.error_with_help(
                "'v-for' directives require no argument",
                &directive.loc,
                "Remove the argument from v-for",
            );
            return;
        }

        // Check for expression
        match &directive.exp {
            None => {
                ctx.error_with_help(
                    "'v-for' directives require an expression",
                    &directive.loc,
                    "Add an expression like: v-for=\"item in items\"",
                );
            }
            Some(exp) => {
                // Validate the expression format
                let content = match exp {
                    ExpressionNode::Simple(s) => s.content.as_str(),
                    ExpressionNode::Compound(_) => return, // Complex expressions are harder to validate
                };

                let trimmed = content.trim();

                // Check if empty
                if trimmed.is_empty() {
                    ctx.error_with_help(
                        "'v-for' directives require a non-empty expression",
                        &directive.loc,
                        "Add an expression like: v-for=\"item in items\"",
                    );
                    return;
                }

                // Check for "in" or "of" keyword
                let has_in = trimmed.contains(" in ");
                let has_of = trimmed.contains(" of ");

                if !has_in && !has_of {
                    ctx.error_with_help(
                        "'v-for' expression must contain 'in' or 'of'",
                        &directive.loc,
                        "Use format: v-for=\"item in items\" or v-for=\"item of items\"",
                    );
                    return;
                }

                // Validate alias part (left side of in/of)
                let (alias_part, source_part) = if has_in {
                    let idx = trimmed.find(" in ").unwrap();
                    (&trimmed[..idx], &trimmed[idx + 4..])
                } else {
                    let idx = trimmed.find(" of ").unwrap();
                    (&trimmed[..idx], &trimmed[idx + 4..])
                };

                let alias = alias_part.trim();
                let source = source_part.trim();

                // Check alias is not empty
                if alias.is_empty() {
                    ctx.error_with_help(
                        "'v-for' alias (left side) cannot be empty",
                        &directive.loc,
                        "Add a variable name before 'in'/'of'",
                    );
                    return;
                }

                // Check source is not empty
                if source.is_empty() {
                    ctx.error_with_help(
                        "'v-for' source (right side) cannot be empty",
                        &directive.loc,
                        "Add an iterable expression after 'in'/'of'",
                    );
                }
            }
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
        registry.register(Box::new(ValidVFor));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_for() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item in items" :key="item.id"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_for_with_index() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="(item, index) in items" :key="index"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_for_of() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item of items" :key="item.id"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_for_no_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-for></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
        assert!(result.diagnostics[0].message.contains("expression"));
    }

    #[test]
    fn test_invalid_v_for_empty_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-for=""></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_for_no_in_or_of() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-for="items"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
        assert!(result.diagnostics[0].message.contains("'in' or 'of'"));
    }
}
