//! v-if generation functions.

use crate::ast::*;

use super::children::generate_children;
use super::context::CodegenContext;
use super::expression::generate_expression;
use super::helpers::escape_js_string;
use super::node::generate_node;

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
                            generate_if_branch_element(ctx, inner, branch, branch_index);
                            return;
                        }
                    }
                    // Template with multiple children -> fragment
                    generate_if_branch_template_fragment(ctx, &el.children, branch, branch_index);
                } else if el.tag_type == ElementType::Component {
                    // Component
                    ctx.use_helper(RuntimeHelper::CreateBlock);
                    ctx.push("(");
                    ctx.push(ctx.helper(RuntimeHelper::OpenBlock));
                    ctx.push("(), ");
                    ctx.push(ctx.helper(RuntimeHelper::CreateBlock));
                    ctx.push("(");
                    // Generate component name
                    let component_name = format!("_component_{}", el.tag.as_str());
                    ctx.push(&component_name);
                    ctx.push(", { key: ");
                    generate_if_branch_key(ctx, branch, branch_index);
                    ctx.push(" }))");
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
    ctx.push("\", { key: ");
    generate_if_branch_key(ctx, branch, branch_index);
    ctx.push(" }");

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
