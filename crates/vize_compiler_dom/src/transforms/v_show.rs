//! v-show transform for DOM elements.
//!
//! v-show toggles the element's display CSS property.

use vize_compiler_core::{DirectiveNode, RuntimeHelper};

/// Runtime helper for v-show
pub const V_SHOW: RuntimeHelper = RuntimeHelper::WithDirectives;

/// Check if directive is v-show
pub fn is_v_show(dir: &DirectiveNode<'_>) -> bool {
    dir.name.as_str() == "show"
}

/// Generate v-show style expression
pub fn generate_show_style(dir: &DirectiveNode<'_>) -> String {
    if let Some(vize_compiler_core::ExpressionNode::Simple(simple)) = &dir.exp {
        return format!("display: ({}) ? '' : 'none'", simple.content);
    }
    String::from("display: ''")
}

/// Generate v-show directive registration for withDirectives
pub fn generate_show_directive(dir: &DirectiveNode<'_>) -> String {
    if let Some(vize_compiler_core::ExpressionNode::Simple(simple)) = &dir.exp {
        return format!("[vShow, {}]", simple.content);
    }
    String::from("[vShow, true]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v_show_helper() {
        assert_eq!(V_SHOW, RuntimeHelper::WithDirectives);
    }
}
