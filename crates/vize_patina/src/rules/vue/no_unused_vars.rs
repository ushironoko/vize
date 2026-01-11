//! vue/no-unused-vars
//!
//! Disallow unused variable definitions in `v-for` directives.
//!
//! This rule reports variables that are defined in `v-for` directives
//! but never used in the template.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <!-- 'index' is defined but never used -->
//! <li v-for="(item, index) in items" :key="item.id">{{ item }}</li>
//! ```
//!
//! ### Valid
//! ```vue
//! <li v-for="(item, index) in items" :key="index">{{ item }}</li>
//! <li v-for="item in items" :key="item.id">{{ item }}</li>
//! ```
//!
//! ## Options
//!
//! Variables starting with `_` are ignored by default (e.g., `_unused`).

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::visitor::parse_v_for_variables;
use rustc_hash::{FxHashMap, FxHashSet};
use vize_relief::ast::{
    ElementNode, ExpressionNode, InterpolationNode, PropNode, RootNode, SourceLocation,
    TemplateChildNode,
};

static META: RuleMeta = RuleMeta {
    name: "vue/no-unused-vars",
    description: "Disallow unused variable definitions in `v-for` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow unused v-for variables
pub struct NoUnusedVars {
    /// Pattern for variables to ignore (default: starts with '_')
    ignore_pattern: Option<String>,
}

impl Default for NoUnusedVars {
    fn default() -> Self {
        Self {
            ignore_pattern: Some("^_".to_string()),
        }
    }
}

impl NoUnusedVars {
    /// Check if a variable name should be ignored
    fn should_ignore(&self, name: &str) -> bool {
        if let Some(pattern) = &self.ignore_pattern {
            if pattern == "^_" {
                return name.starts_with('_');
            }
            // For more complex patterns, we'd use regex
            // For now, just support the common ^_ pattern
        }
        false
    }
}

/// Track variable definitions and usages within a v-for scope
#[derive(Debug)]
struct VForScope {
    /// Variables defined by this v-for with their locations
    variables: FxHashMap<String, SourceLocation>,
    /// Variables that have been used
    used: FxHashSet<String>,
}

impl Rule for NoUnusedVars {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        // We need to do a custom traversal to track variable usage
        // Store the current rule name so diagnostics are attributed correctly
        let prev_rule = ctx.current_rule;
        ctx.current_rule = META.name;

        #[cfg(test)]
        eprintln!("no_unused_vars: checking {} children", root.children.len());

        let mut checker = UnusedVarsChecker::new(self);
        checker.check_children(ctx, &root.children);

        ctx.current_rule = prev_rule;
    }
}

struct UnusedVarsChecker<'r> {
    rule: &'r NoUnusedVars,
    /// Stack of v-for scopes
    scopes: Vec<VForScope>,
}

impl<'r> UnusedVarsChecker<'r> {
    fn new(rule: &'r NoUnusedVars) -> Self {
        Self {
            rule,
            scopes: Vec::new(),
        }
    }

    fn check_children<'a>(
        &mut self,
        ctx: &mut LintContext<'a>,
        children: &[TemplateChildNode<'a>],
    ) {
        for child in children {
            self.check_node(ctx, child);
        }
    }

    fn check_node<'a>(&mut self, ctx: &mut LintContext<'a>, node: &TemplateChildNode<'a>) {
        match node {
            TemplateChildNode::Element(el) => self.check_element(ctx, el),
            TemplateChildNode::Interpolation(interp) => self.check_interpolation(interp),
            TemplateChildNode::If(if_node) => {
                for branch in if_node.branches.iter() {
                    // Check condition for variable usage
                    if let Some(condition) = &branch.condition {
                        self.mark_expression_vars_used(condition);
                    }
                    self.check_children(ctx, &branch.children);
                }
            }
            TemplateChildNode::For(for_node) => {
                // v-for nodes already have variables extracted
                self.check_children(ctx, &for_node.children);
            }
            _ => {}
        }
    }

    fn check_element<'a>(&mut self, ctx: &mut LintContext<'a>, el: &ElementNode<'a>) {
        // Check if this element has v-for
        let v_for_info = self.extract_v_for_info(el);
        let has_v_for = v_for_info.is_some();

        #[cfg(test)]
        eprintln!(
            "check_element: <{}> has_v_for={} v_for_info={:?}",
            el.tag,
            has_v_for,
            v_for_info.as_ref().map(|(vars, _)| vars)
        );

        if let Some((vars, loc)) = v_for_info {
            // Push new scope
            let mut var_map = FxHashMap::default();
            for var in &vars {
                var_map.insert(var.clone(), loc.clone());
            }
            self.scopes.push(VForScope {
                variables: var_map,
                used: FxHashSet::default(),
            });
        }

        // Check all props for variable usage
        for prop in el.props.iter() {
            match prop {
                PropNode::Attribute(attr) => {
                    // Check static attribute value for mustache-like usage (rare but possible)
                    if let Some(value) = &attr.value {
                        self.mark_vars_in_text(&value.content);
                    }
                }
                PropNode::Directive(dir) => {
                    // Skip v-for directive's exp - it defines variables, doesn't use them
                    if dir.name.as_str() == "for" {
                        continue;
                    }
                    // Check directive expression
                    if let Some(exp) = &dir.exp {
                        self.mark_expression_vars_used(exp);
                    }
                    // Check directive argument
                    if let Some(arg) = &dir.arg {
                        self.mark_expression_vars_used(arg);
                    }
                }
            }
        }

        // Check children
        self.check_children(ctx, &el.children);

        // If we have a v-for scope, report unused variables
        if has_v_for {
            if let Some(scope) = self.scopes.pop() {
                #[cfg(test)]
                eprintln!(
                    "Scope popped - variables: {:?}, used: {:?}",
                    scope.variables.keys().collect::<Vec<_>>(),
                    scope.used
                );

                for (var_name, var_loc) in scope.variables {
                    if !scope.used.contains(&var_name) && !self.rule.should_ignore(&var_name) {
                        #[cfg(test)]
                        eprintln!("Reporting unused var: {}", var_name);

                        ctx.warn_with_help(
                            format!("'{}' is defined but never used", var_name),
                            &var_loc,
                            format!(
                                "Remove the unused variable or prefix with underscore: _{}",
                                var_name
                            ),
                        );
                    }
                }
            }
        }
    }

    fn check_interpolation(&mut self, interp: &InterpolationNode) {
        self.mark_expression_vars_used(&interp.content);
    }

    fn extract_v_for_info(&self, el: &ElementNode) -> Option<(Vec<String>, SourceLocation)> {
        for prop in el.props.iter() {
            if let PropNode::Directive(dir) = prop {
                if dir.name.as_str() == "for" {
                    if let Some(exp) = &dir.exp {
                        let vars: Vec<String> = parse_v_for_variables(exp)
                            .into_iter()
                            .map(|s| s.to_string())
                            .collect();
                        if !vars.is_empty() {
                            return Some((vars, dir.loc.clone()));
                        }
                    }
                }
            }
        }
        None
    }

    fn mark_expression_vars_used(&mut self, exp: &ExpressionNode) {
        let content = match exp {
            ExpressionNode::Simple(s) => s.content.as_str(),
            ExpressionNode::Compound(c) => {
                // For compound expressions, check each child
                for child in c.children.iter() {
                    match child {
                        vize_relief::ast::CompoundExpressionChild::Simple(s) => {
                            self.mark_vars_in_text(s.content.as_str());
                        }
                        vize_relief::ast::CompoundExpressionChild::Interpolation(i) => {
                            self.mark_expression_vars_used(&i.content);
                        }
                        _ => {}
                    }
                }
                return;
            }
        };

        self.mark_vars_in_text(content);
    }

    fn mark_vars_in_text(&mut self, text: &str) {
        // Mark any variables in the current scopes as used if they appear in the text
        for scope in self.scopes.iter_mut() {
            for var_name in scope.variables.keys() {
                if text_contains_identifier(text, var_name) {
                    scope.used.insert(var_name.clone());
                }
            }
        }
    }
}

/// Check if text contains an identifier (not as part of another word)
fn text_contains_identifier(text: &str, identifier: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = text[start..].find(identifier) {
        let abs_pos = start + pos;
        let before_ok = abs_pos == 0
            || !text[..abs_pos]
                .chars()
                .last()
                .map(|c| c.is_alphanumeric() || c == '_')
                .unwrap_or(false);

        let after_pos = abs_pos + identifier.len();
        let after_ok = after_pos >= text.len()
            || !text[after_pos..]
                .chars()
                .next()
                .map(|c| c.is_alphanumeric() || c == '_')
                .unwrap_or(false);

        if before_ok && after_ok {
            return true;
        }

        start = abs_pos + 1;
        if start >= text.len() {
            break;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoUnusedVars::default()));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_all_vars_used() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="(item, index) in items" :key="index">{{ item }}</li>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_single_var_used() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="item in items" :key="item.id">{{ item.name }}</li>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_index_unused() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="(item, index) in items" :key="item.id">{{ item }}</li>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
        assert!(result.diagnostics[0].message.contains("index"));
    }

    #[test]
    fn test_invalid_item_unused() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="(item, index) in items" :key="index">Static content</li>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
        assert!(result.diagnostics[0].message.contains("item"));
    }

    #[test]
    fn test_ignored_underscore_prefix() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="(item, _index) in items" :key="item.id">{{ item }}</li>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_var_used_in_directive() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="item in items" :key="item.id" :class="{ active: item.active }">Click</li>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_var_used_in_event_handler() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="item in items" :key="item.id" @click="select(item)">Click</li>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_nested_v_for() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="row in rows" :key="row.id"><span v-for="cell in row.cells" :key="cell.id">{{ cell }}</span></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_text_contains_identifier() {
        assert!(text_contains_identifier("item.name", "item"));
        assert!(text_contains_identifier("foo + item", "item"));
        assert!(text_contains_identifier("item", "item"));
        assert!(!text_contains_identifier("items", "item"));
        assert!(!text_contains_identifier("myitem", "item"));
        assert!(text_contains_identifier("arr[index]", "index"));
    }
}
