//! Directive code generation for Vapor mode.

use super::block::GenerateContext;
use crate::ir::DirectiveIRNode;
use vize_compiler_core::ExpressionNode;

/// Generate Directive code
pub fn generate_directive(ctx: &mut GenerateContext, directive: &DirectiveIRNode<'_>) {
    let element = format!("_n{}", directive.element);
    let name = &directive.name;

    let arg = if let Some(ref arg) = directive.dir.arg {
        match arg {
            ExpressionNode::Simple(exp) => {
                if exp.is_static {
                    format!("\"{}\"", exp.content)
                } else {
                    exp.content.to_string()
                }
            }
            ExpressionNode::Compound(c) => c.loc.source.to_string(),
        }
    } else {
        String::from("undefined")
    };

    let value = if let Some(ref exp) = directive.dir.exp {
        match exp {
            ExpressionNode::Simple(e) => {
                if e.is_static {
                    format!("\"{}\"", e.content)
                } else {
                    e.content.to_string()
                }
            }
            ExpressionNode::Compound(c) => c.loc.source.to_string(),
        }
    } else {
        String::from("undefined")
    };

    // Generate modifiers object
    let modifiers = if directive.dir.modifiers.is_empty() {
        String::from("{}")
    } else {
        let mod_strs: Vec<String> = directive
            .dir
            .modifiers
            .iter()
            .map(|m| format!("{}: true", m.content))
            .collect();
        format!("{{ {} }}", mod_strs.join(", "))
    };

    if directive.builtin {
        // Built-in directive
        match name.as_str() {
            "show" => {
                ctx.push_line(&format!(
                    "_withDirectives({}, [[_vShow, {}]])",
                    element, value
                ));
            }
            "model" => {
                ctx.push_line(&format!(
                    "_withDirectives({}, [[_vModel, {}, {}, {}]])",
                    element, value, arg, modifiers
                ));
            }
            _ => {
                ctx.push_line(&format!(
                    "_withDirectives({}, [[_{}, {}, {}, {}]])",
                    element, name, value, arg, modifiers
                ));
            }
        }
    } else {
        // Custom directive
        ctx.push_line(&format!(
            "_withDirectives({}, [[_directive_{}, {}, {}, {}]])",
            element, name, value, arg, modifiers
        ));
    }
}

/// Generate directive resolution
pub fn generate_resolve_directive(name: &str) -> String {
    format!("_resolveDirective(\"{}\")", name)
}

/// Generate v-show directive
pub fn generate_v_show(element_var: &str, value: &str) -> String {
    format!("{}.style.display = {} ? '' : 'none'", element_var, value)
}

/// Generate v-cloak removal
pub fn generate_v_cloak_removal(element_var: &str) -> String {
    format!("{}.removeAttribute('v-cloak')", element_var)
}

/// Generate v-pre handling (skip compilation marker)
pub fn is_v_pre_element(_element: &str) -> bool {
    // Elements with v-pre are handled specially during parsing
    false
}

/// Generate withDirectives call
pub fn generate_with_directives(element_var: &str, directives: &[String]) -> String {
    format!(
        "_withDirectives({}, [{}])",
        element_var,
        directives.join(", ")
    )
}

/// Generate single directive array
pub fn generate_directive_array(
    directive: &str,
    value: &str,
    arg: Option<&str>,
    modifiers: Option<&str>,
) -> String {
    let mut parts = vec![directive.to_string(), value.to_string()];

    if let Some(a) = arg {
        parts.push(a.to_string());
        if let Some(m) = modifiers {
            parts.push(m.to_string());
        }
    } else if let Some(m) = modifiers {
        parts.push(String::from("undefined"));
        parts.push(m.to_string());
    }

    format!("[{}]", parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_resolve_directive() {
        let result = generate_resolve_directive("focus");
        assert_eq!(result, "_resolveDirective(\"focus\")");
    }

    #[test]
    fn test_generate_v_show() {
        let result = generate_v_show("_n1", "isVisible");
        assert_eq!(result, "_n1.style.display = isVisible ? '' : 'none'");
    }

    #[test]
    fn test_generate_directive_array_simple() {
        let result = generate_directive_array("_vShow", "isVisible", None, None);
        assert_eq!(result, "[_vShow, isVisible]");
    }

    #[test]
    fn test_generate_directive_array_with_all() {
        let result =
            generate_directive_array("_vModel", "text", Some("\"value\""), Some("{ lazy: true }"));
        assert_eq!(result, "[_vModel, text, \"value\", { lazy: true }]");
    }
}
