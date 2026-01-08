//! v-text transform for DOM elements.
//!
//! v-text sets the element's textContent.

use vize_compiler_core::{DirectiveNode, RuntimeHelper};

/// Runtime helper for v-text
pub const V_TEXT: RuntimeHelper = RuntimeHelper::SetBlockTracking;

/// Check if directive is v-text
pub fn is_v_text(dir: &DirectiveNode<'_>) -> bool {
    dir.name.as_str() == "text"
}

/// Generate v-text expression
pub fn generate_text_content(dir: &DirectiveNode<'_>) -> String {
    if let Some(ref exp) = dir.exp {
        if let vize_compiler_core::ExpressionNode::Simple(simple) = exp {
            return format!("_toDisplayString({})", simple.content);
        }
    }
    String::from("''")
}

/// Generate children replacement for v-text
pub fn generate_text_children(dir: &DirectiveNode<'_>) -> Option<String> {
    if let Some(ref exp) = dir.exp {
        if let vize_compiler_core::ExpressionNode::Simple(simple) = exp {
            return Some(format!("_toDisplayString({})", simple.content));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use vize_allocator::{Box, Bump};
    use vize_compiler_core::{DirectiveNode, ExpressionNode, SimpleExpressionNode, SourceLocation};

    use super::*;

    fn create_test_directive<'a>(allocator: &'a Bump, name: &str, exp: &str) -> DirectiveNode<'a> {
        let mut dir = DirectiveNode::new(allocator, name, SourceLocation::STUB);
        let exp_node = SimpleExpressionNode::new(exp, false, SourceLocation::STUB);
        let boxed = Box::new_in(exp_node, allocator);
        dir.exp = Some(ExpressionNode::Simple(boxed));
        dir
    }

    #[test]
    fn test_is_v_text() {
        let allocator = Bump::new();
        let dir = create_test_directive(&allocator, "text", "msg");
        assert!(is_v_text(&dir));
    }

    #[test]
    fn test_generate_text_content() {
        let allocator = Bump::new();
        let dir = create_test_directive(&allocator, "text", "msg");
        let result = generate_text_content(&dir);
        assert!(result.contains("_toDisplayString"));
        assert!(result.contains("msg"));
    }
}
