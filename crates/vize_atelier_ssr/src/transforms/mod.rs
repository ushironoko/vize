//! SSR-specific transforms.
//!
//! This module contains SSR-specific transform passes that modify the AST
//! for optimal SSR code generation.

use vize_atelier_core::ast::{ElementNode, ExpressionNode, PropNode};

// For now, most SSR-specific transforms are integrated directly into the codegen.
// This module will be expanded as we add more sophisticated transforms.

/// Check if an element has v-model directive
pub fn has_v_model(el: &ElementNode) -> bool {
    el.props
        .iter()
        .any(|p| matches!(p, PropNode::Directive(dir) if dir.name == "model"))
}

/// Check if an element has v-show directive
pub fn has_v_show(el: &ElementNode) -> bool {
    el.props
        .iter()
        .any(|p| matches!(p, PropNode::Directive(dir) if dir.name == "show"))
}

/// Check if an element has v-html directive
pub fn has_v_html(el: &ElementNode) -> bool {
    el.props
        .iter()
        .any(|p| matches!(p, PropNode::Directive(dir) if dir.name == "html"))
}

/// Check if an element has v-text directive
pub fn has_v_text(el: &ElementNode) -> bool {
    el.props
        .iter()
        .any(|p| matches!(p, PropNode::Directive(dir) if dir.name == "text"))
}

/// Get v-model expression if present
pub fn get_v_model_exp<'a>(el: &'a ElementNode<'a>) -> Option<&'a ExpressionNode<'a>> {
    for prop in &el.props {
        if let PropNode::Directive(dir) = prop {
            if dir.name == "model" {
                return dir.exp.as_ref();
            }
        }
    }
    None
}

/// Get v-show expression if present
pub fn get_v_show_exp<'a>(el: &'a ElementNode<'a>) -> Option<&'a ExpressionNode<'a>> {
    for prop in &el.props {
        if let PropNode::Directive(dir) = prop {
            if dir.name == "show" {
                return dir.exp.as_ref();
            }
        }
    }
    None
}

/// Get v-html expression if present
pub fn get_v_html_exp<'a>(el: &'a ElementNode<'a>) -> Option<&'a ExpressionNode<'a>> {
    for prop in &el.props {
        if let PropNode::Directive(dir) = prop {
            if dir.name == "html" {
                return dir.exp.as_ref();
            }
        }
    }
    None
}

/// Get v-text expression if present
pub fn get_v_text_exp<'a>(el: &'a ElementNode<'a>) -> Option<&'a ExpressionNode<'a>> {
    for prop in &el.props {
        if let PropNode::Directive(dir) = prop {
            if dir.name == "text" {
                return dir.exp.as_ref();
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    // Tests will be added along with the full transform implementation
}
