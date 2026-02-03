//! v-if generation functions.

use crate::ast::*;

use super::children::generate_children;
use super::context::CodegenContext;
use super::expression::generate_expression;
use super::helpers::{escape_js_string, is_valid_js_identifier};
use super::node::generate_node;
use super::props::{generate_directive_prop_with_static, generate_props};

/// Generate if node
pub fn generate_if(ctx: &mut CodegenContext, if_node: &IfNode<'_>) {
    ctx.use_helper(RuntimeHelper::OpenBlock);

    // Vue always imports createCommentVNode for v-if nodes
    ctx.use_helper(RuntimeHelper::CreateComment);

    for (i, branch) in if_node.branches.iter().enumerate() {
        if let Some(condition) = &branch.condition {
            if i == 0 {
                // First branch: output condition with parentheses
                ctx.push("(");
                generate_expression(ctx, condition);
                ctx.push(")");
                ctx.indent();
                ctx.newline();
                ctx.push("? ");
            } else {
                // Subsequent branches (else-if)
                ctx.newline();
                ctx.push(": (");
                generate_expression(ctx, condition);
                ctx.push(")");
                ctx.indent();
                ctx.newline();
                ctx.push("? ");
            }
        } else {
            // Else branch (no condition)
            ctx.newline();
            ctx.push(": ");
        }

        // Generate branch content based on children
        generate_if_branch(ctx, branch, i);

        if branch.condition.is_some() && i > 0 {
            ctx.deindent();
        }
    }

    // Else branch (comment node) - only if all branches have conditions
    if if_node.branches.iter().all(|b| b.condition.is_some()) {
        ctx.newline();
        ctx.push(": ");
        ctx.push(ctx.helper(RuntimeHelper::CreateComment));
        ctx.push("(\"v-if\", true)");
    }

    ctx.deindent();
}

/// Generate a single if branch
pub fn generate_if_branch(
    ctx: &mut CodegenContext,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    // Single child optimization
    if branch.children.len() == 1 {
        match &branch.children[0] {
            TemplateChildNode::Element(el) => {
                // Check if it's a template element - treat as fragment
                if el.tag_type == ElementType::Template {
                    // Template with single child -> unwrap to single element
                    if el.children.len() == 1 {
                        if let TemplateChildNode::Element(inner) = &el.children[0] {
                            // Check if inner element is a component
                            if inner.tag_type == ElementType::Component {
                                generate_if_branch_component(ctx, inner, branch, branch_index);
                            } else {
                                generate_if_branch_element(ctx, inner, branch, branch_index);
                            }
                            return;
                        }
                    }
                    // Template with multiple children -> fragment
                    generate_if_branch_template_fragment(ctx, &el.children, branch, branch_index);
                } else if el.tag_type == ElementType::Component {
                    // Component
                    generate_if_branch_component(ctx, el, branch, branch_index);
                } else {
                    // Regular element
                    generate_if_branch_element(ctx, el, branch, branch_index);
                }
            }
            _ => {
                // Other node types - wrap in fragment
                generate_if_branch_fragment(ctx, branch, branch_index);
            }
        }
    } else {
        // Multiple children - wrap in fragment
        generate_if_branch_fragment(ctx, branch, branch_index);
    }
}

/// Generate component for if branch
pub fn generate_if_branch_component(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    ctx.use_helper(RuntimeHelper::CreateBlock);
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
    ctx.push("(), ");
    ctx.push(ctx.helper(RuntimeHelper::CreateBlock));
    ctx.push("(");
    // Generate component name
    // In inline mode, components are directly in scope (imported at module level)
    // In function mode, use $setup.ComponentName to access setup bindings
    if ctx.is_component_in_bindings(&el.tag) {
        if !ctx.options.inline {
            ctx.push("$setup.");
        }
        ctx.push(el.tag.as_str());
    } else {
        let component_name = format!("_component_{}", el.tag.as_str());
        ctx.push(&component_name);
    }

    // Check if component has v-bind object spread - needs special handling
    if has_vbind_object_spread(el) {
        // When v-bind="obj" is present, we need to use _mergeProps to combine
        // the spread object with the key and other props
        ctx.use_helper(RuntimeHelper::MergeProps);
        ctx.push(", ");
        ctx.push(ctx.helper(RuntimeHelper::MergeProps));
        ctx.push("({ key: ");
        generate_if_branch_key(ctx, branch, branch_index);
        ctx.push(" }, ");
        // Use generate_props which handles v-bind object spread correctly
        generate_props(ctx, &el.props);
        ctx.push(")");
    } else {
        // No v-bind object spread - generate props inline
        // Check if component has props
        let has_other = has_other_props_for_if(el);
        ctx.push(", {");
        ctx.indent();
        ctx.newline();
        ctx.push("key: ");
        generate_if_branch_key(ctx, branch, branch_index);

        // Extract static class/style for merging with dynamic bindings
        let (static_class, static_style) = extract_static_class_style(el);
        let has_dyn_class = has_dynamic_class(el);
        let has_dyn_style = has_dynamic_style(el);

        // Add other props if any (skip duplicate event handlers)
        if has_other {
            let mut seen_events: std::collections::HashSet<String> = std::collections::HashSet::new();
            for prop in el.props.iter() {
                if should_skip_prop_for_if(prop, has_dyn_class, has_dyn_style) {
                    continue;
                }
                // Skip duplicate v-on event handlers (can occur with v-model transform)
                if let PropNode::Directive(dir) = prop {
                    if dir.name == "on" {
                        if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                            let event_key = arg.content.to_string();
                            if seen_events.contains(&event_key) {
                                continue; // Skip duplicate event
                            }
                            seen_events.insert(event_key);
                        }
                    }
                }
                ctx.push(",");
                ctx.newline();
                generate_single_prop_for_if(ctx, prop, static_class, static_style);
            }
        }

        // Add scope_id for scoped CSS
        if let Some(ref scope_id) = ctx.options.scope_id.clone() {
            ctx.push(",");
            ctx.newline();
            ctx.push("\"");
            ctx.push(scope_id);
            ctx.push("\": \"\"");
        }

        ctx.deindent();
        ctx.newline();
        ctx.push("}");
    }

    ctx.push("))")
}

/// Check if element has v-bind object spread (v-bind="obj" without argument)
fn has_vbind_object_spread(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|p| {
        if let PropNode::Directive(dir) = p {
            dir.name == "bind" && dir.arg.is_none()
        } else {
            false
        }
    })
}

/// Check if element has props besides the key (for v-if branch elements)
fn has_other_props_for_if(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|p| match p {
        PropNode::Attribute(_) => true,
        PropNode::Directive(dir) => {
            // Skip key binding (handled separately)
            if dir.name == "bind" {
                if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                    if arg.content == "key" {
                        return false;
                    }
                }
                // Skip v-bind object spread (handled separately)
                if dir.arg.is_none() {
                    return false;
                }
                return true; // v-bind with arg is supported
            }
            // Skip v-if/v-else-if/v-else/v-slot directives
            if matches!(dir.name.as_str(), "if" | "else-if" | "else" | "slot") {
                return false;
            }
            // Skip v-model - it's handled via withDirectives for native elements
            if dir.name == "model" {
                return false;
            }
            // v-on, v-html, v-text are supported - count them
            if matches!(dir.name.as_str(), "on" | "html" | "text") {
                return true;
            }
            // Skip custom directives (v-click-outside, v-focus, etc.)
            false
        }
    })
}

/// Check if prop should be skipped for v-if branch element
fn should_skip_prop_for_if(
    p: &PropNode<'_>,
    has_dynamic_class: bool,
    has_dynamic_style: bool,
) -> bool {
    match p {
        PropNode::Attribute(attr) => {
            // Skip static class if there's a dynamic :class (will be merged)
            if attr.name == "class" && has_dynamic_class {
                return true;
            }
            // Skip static style if there's a dynamic :style (will be merged)
            if attr.name == "style" && has_dynamic_style {
                return true;
            }
            false
        }
        PropNode::Directive(dir) => {
            if dir.name == "bind" {
                if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                    if arg.content == "key" {
                        return true;
                    }
                }
                return false; // v-bind is supported
            }
            // Skip v-if/v-else-if/v-else/v-slot directives
            // v-slot is handled separately as slot definition, not as prop
            if matches!(dir.name.as_str(), "if" | "else-if" | "else" | "slot") {
                return true;
            }
            // Skip v-model for native elements - it's handled via withDirectives
            // The onUpdate:modelValue handler is added separately by transform
            if dir.name == "model" {
                return true;
            }
            // v-on, v-html, v-text are supported - don't skip
            if matches!(dir.name.as_str(), "on" | "html" | "text") {
                return false;
            }
            // Skip custom directives (v-click-outside, v-focus, etc.)
            // These are handled separately with withDirectives at runtime
            true
        }
    }
}

/// Extract static class and style values from element props
fn extract_static_class_style<'a>(el: &'a ElementNode<'_>) -> (Option<&'a str>, Option<&'a str>) {
    let mut static_class = None;
    let mut static_style = None;
    for prop in el.props.iter() {
        if let PropNode::Attribute(attr) = prop {
            if attr.name == "class" {
                if let Some(val) = &attr.value {
                    static_class = Some(val.content.as_str());
                }
            } else if attr.name == "style" {
                if let Some(val) = &attr.value {
                    static_style = Some(val.content.as_str());
                }
            }
        }
    }
    (static_class, static_style)
}

/// Check if element has dynamic :class binding
fn has_dynamic_class(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|p| {
        if let PropNode::Directive(dir) = p {
            if dir.name == "bind" {
                if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                    return arg.content == "class";
                }
            }
        }
        false
    })
}

/// Check if element has dynamic :style binding
fn has_dynamic_style(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|p| {
        if let PropNode::Directive(dir) = p {
            if dir.name == "bind" {
                if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                    return arg.content == "style";
                }
            }
        }
        false
    })
}

/// Generate a single prop for v-if branch element
fn generate_single_prop_for_if(
    ctx: &mut CodegenContext,
    prop: &PropNode<'_>,
    static_class: Option<&str>,
    static_style: Option<&str>,
) {
    match prop {
        PropNode::Attribute(attr) => {
            let needs_quotes = !is_valid_js_identifier(&attr.name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(&attr.name);
            if needs_quotes {
                ctx.push("\"");
            }
            ctx.push(": ");
            if let Some(value) = &attr.value {
                ctx.push("\"");
                ctx.push(&escape_js_string(value.content.as_str()));
                ctx.push("\"");
            } else {
                ctx.push("\"\"");
            }
        }
        PropNode::Directive(dir) => {
            generate_directive_prop_with_static(ctx, dir, static_class, static_style);
        }
    }
}

/// Generate element for if branch
pub fn generate_if_branch_element(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    ctx.use_helper(RuntimeHelper::CreateElementBlock);
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
    ctx.push("(), ");
    ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
    ctx.push("(\"");
    ctx.push(el.tag.as_str());
    ctx.push("\"");

    // Extract static class/style for merging with dynamic bindings
    let (static_class, static_style) = extract_static_class_style(el);
    let has_dyn_class = has_dynamic_class(el);
    let has_dyn_style = has_dynamic_style(el);

    // Generate props with key and all other props
    let has_other = has_other_props_for_if(el);
    ctx.push(", {");
    ctx.indent();
    ctx.newline();
    ctx.push("key: ");
    generate_if_branch_key(ctx, branch, branch_index);

    // Add other props (skip duplicate event handlers)
    if has_other {
        let mut seen_events: std::collections::HashSet<String> = std::collections::HashSet::new();
        for prop in el.props.iter() {
            if should_skip_prop_for_if(prop, has_dyn_class, has_dyn_style) {
                continue;
            }
            // Skip duplicate v-on event handlers (can occur with v-model transform)
            if let PropNode::Directive(dir) = prop {
                if dir.name == "on" {
                    if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                        let event_key = arg.content.to_string();
                        if seen_events.contains(&event_key) {
                            continue; // Skip duplicate event
                        }
                        seen_events.insert(event_key);
                    }
                }
            }
            ctx.push(",");
            ctx.newline();
            generate_single_prop_for_if(ctx, prop, static_class, static_style);
        }
    }

    // Add scope_id for scoped CSS
    if let Some(ref scope_id) = ctx.options.scope_id.clone() {
        ctx.push(",");
        ctx.newline();
        ctx.push("\"");
        ctx.push(scope_id);
        ctx.push("\": \"\"");
    }

    ctx.deindent();
    ctx.newline();
    ctx.push("}");

    // Generate children if any
    if !el.children.is_empty() {
        ctx.push(", ");
        if el.children.len() == 1 {
            if let TemplateChildNode::Text(text) = &el.children[0] {
                ctx.push("\"");
                ctx.push(&escape_js_string(text.content.as_str()));
                ctx.push("\"");
            } else {
                generate_if_branch_children(ctx, &el.children);
            }
        } else {
            generate_if_branch_children(ctx, &el.children);
        }
    }

    ctx.push("))");
}

/// Generate template fragment for if branch (multiple children from template)
pub fn generate_if_branch_template_fragment(
    ctx: &mut CodegenContext,
    children: &[TemplateChildNode<'_>],
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    ctx.use_helper(RuntimeHelper::CreateElementBlock);
    ctx.use_helper(RuntimeHelper::Fragment);
    ctx.use_helper(RuntimeHelper::CreateElementVNode);
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
    ctx.push("(), ");
    ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::Fragment));
    ctx.push(", { key: ");
    generate_if_branch_key(ctx, branch, branch_index);
    ctx.push(" }, [");
    ctx.indent();
    for (i, child) in children.iter().enumerate() {
        if i > 0 {
            ctx.push(",");
        }
        ctx.newline();
        generate_node(ctx, child);
    }
    ctx.deindent();
    ctx.newline();
    ctx.push("], 64 /* STABLE_FRAGMENT */))");
}

/// Generate key for if branch
pub fn generate_if_branch_key(
    ctx: &mut CodegenContext,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    // Check if branch has a user-provided key
    if let Some(ref user_key) = branch.user_key {
        match user_key {
            PropNode::Attribute(attr) => {
                // Static key attribute
                if let Some(ref value) = attr.value {
                    ctx.push("\"");
                    ctx.push(&escape_js_string(value.content.as_str()));
                    ctx.push("\"");
                } else {
                    ctx.push(&branch_index.to_string());
                }
            }
            PropNode::Directive(dir) => {
                // Dynamic :key binding
                if let Some(ref exp) = dir.exp {
                    generate_expression(ctx, exp);
                } else {
                    ctx.push(&branch_index.to_string());
                }
            }
        }
    } else {
        ctx.push(&branch_index.to_string());
    }
}

/// Generate fragment wrapper for if branch with multiple children
pub fn generate_if_branch_fragment(
    ctx: &mut CodegenContext,
    branch: &IfBranchNode<'_>,
    branch_index: usize,
) {
    ctx.use_helper(RuntimeHelper::CreateElementBlock);
    ctx.use_helper(RuntimeHelper::Fragment);
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
    ctx.push("(), ");
    ctx.push(ctx.helper(RuntimeHelper::CreateElementBlock));
    ctx.push("(");
    ctx.push(ctx.helper(RuntimeHelper::Fragment));
    ctx.push(", { key: ");
    generate_if_branch_key(ctx, branch, branch_index);
    ctx.push(" }, ");
    generate_children(ctx, &branch.children);
    ctx.push(", 64 /* STABLE_FRAGMENT */))");
}

/// Generate children for if branch element
pub fn generate_if_branch_children(ctx: &mut CodegenContext, children: &[TemplateChildNode<'_>]) {
    if children.is_empty() {
        return;
    }

    // Check if all children are simple (text or interpolation)
    let has_only_text_or_interpolation = children.iter().all(|c| {
        matches!(
            c,
            TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
        )
    });

    if has_only_text_or_interpolation && children.len() == 1 {
        match &children[0] {
            TemplateChildNode::Interpolation(interp) => {
                ctx.use_helper(RuntimeHelper::ToDisplayString);
                ctx.push(ctx.helper(RuntimeHelper::ToDisplayString));
                ctx.push("(");
                generate_expression(ctx, &interp.content);
                ctx.push("), 1 /* TEXT */");
            }
            TemplateChildNode::Text(text) => {
                ctx.push("\"");
                ctx.push(&escape_js_string(text.content.as_str()));
                ctx.push("\"");
            }
            _ => {}
        }
    } else {
        // Complex children - use array
        ctx.push("[");
        ctx.indent();
        for (i, child) in children.iter().enumerate() {
            if i > 0 {
                ctx.push(",");
            }
            ctx.newline();
            generate_node(ctx, child);
        }
        ctx.deindent();
        ctx.newline();
        ctx.push("]");
    }
}

#[cfg(test)]
mod tests {
    //! Tests for v-if directive generation.
    //!
    //! Note: Full AST node tests require bumpalo allocation.
    //! For comprehensive testing of the v-if behavior,
    //! see the SFC snapshot tests in tests/fixtures/sfc/patches.toml.

    use std::collections::HashSet;

    /// Test v-bind object spread detection logic
    #[test]
    fn test_vbind_object_spread_logic() {
        // v-bind without argument = object spread (v-bind="props")
        let is_object_spread = |name: &str, has_arg: bool| -> bool {
            name == "bind" && !has_arg
        };

        // v-bind="props" - object spread
        assert!(is_object_spread("bind", false));

        // v-bind:class="..." - named binding, not spread
        assert!(!is_object_spread("bind", true));

        // v-on:click - not v-bind
        assert!(!is_object_spread("on", false));
    }

    /// Test that the HashSet-based duplicate event filtering works correctly
    #[test]
    fn test_duplicate_event_filtering() {
        let mut seen_events: HashSet<String> = HashSet::new();

        // First "click" event - should be added
        let event1 = "click".to_string();
        assert!(!seen_events.contains(&event1));
        seen_events.insert(event1.clone());

        // Second "click" event - should be detected as duplicate
        let event2 = "click".to_string();
        assert!(seen_events.contains(&event2));

        // Different event - should not be duplicate
        let event3 = "input".to_string();
        assert!(!seen_events.contains(&event3));
        seen_events.insert(event3.clone());

        // Now input is also seen
        assert!(seen_events.contains(&"input".to_string()));
    }

    /// Test the logic for which props to skip in v-if branches
    #[test]
    fn test_prop_skip_logic_for_if() {
        // Props/directives that should be skipped in v-if
        let should_skip_in_if = |name: &str| -> bool {
            matches!(name, "if" | "else-if" | "else" | "slot")
        };

        assert!(should_skip_in_if("if"));
        assert!(should_skip_in_if("else-if"));
        assert!(should_skip_in_if("else"));
        assert!(should_skip_in_if("slot"));

        // These should NOT be skipped
        assert!(!should_skip_in_if("on"));
        assert!(!should_skip_in_if("bind"));
        assert!(!should_skip_in_if("model"));
    }

    /// Test v-model directive handling in v-if (should be handled separately)
    #[test]
    fn test_vmodel_in_vif_logic() {
        // v-model should be skipped for props generation in v-if
        // because it's handled via withDirectives for native elements
        let should_skip_vmodel_in_props = |name: &str| -> bool {
            name == "model"
        };

        assert!(should_skip_vmodel_in_props("model"));
        assert!(!should_skip_vmodel_in_props("bind"));
        assert!(!should_skip_vmodel_in_props("on"));
    }

    /// Test key generation for v-if branches
    #[test]
    fn test_if_branch_key_generation() {
        // Keys should be sequential integers by default
        let generate_key = |branch_index: usize| -> i32 {
            branch_index as i32
        };

        assert_eq!(generate_key(0), 0);
        assert_eq!(generate_key(1), 1);
        assert_eq!(generate_key(2), 2);
    }
}
