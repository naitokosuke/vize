//! Prop code generation for Vapor mode.

use super::block::GenerateContext;
use crate::ir::{SetDynamicPropsIRNode, SetPropIRNode};

/// Generate SetProp code
pub fn generate_set_prop(ctx: &mut GenerateContext, set_prop: &SetPropIRNode<'_>) {
    let element = format!("_n{}", set_prop.element);
    let key = &set_prop.prop.key.content;

    let value = if let Some(first) = set_prop.prop.values.first() {
        if first.is_static {
            format!("\"{}\"", first.content)
        } else {
            first.content.to_string()
        }
    } else {
        String::from("undefined")
    };

    // Determine how to set the prop
    if is_dom_prop(key) {
        // DOM property
        ctx.push_line(&format!("{}.{} = {}", element, key, value));
    } else if key.starts_with("on") {
        // Event handler as prop (component)
        ctx.push_line(&format!(
            "_setEventProp({}, \"{}\", {})",
            element, key, value
        ));
    } else {
        // Attribute
        ctx.push_line(&format!(
            "_setAttribute({}, \"{}\", {})",
            element, key, value
        ));
    }
}

/// Generate SetDynamicProps code
pub fn generate_set_dynamic_props(
    ctx: &mut GenerateContext,
    set_props: &SetDynamicPropsIRNode<'_>,
) {
    let element = format!("_n{}", set_props.element);

    for prop in set_props.props.iter() {
        let expr = if prop.is_static {
            format!("\"{}\"", prop.content)
        } else {
            prop.content.to_string()
        };
        ctx.push_line(&format!("_setDynamicProps({}, {})", element, expr));
    }
}

/// Check if key is a DOM property (vs attribute)
fn is_dom_prop(key: &str) -> bool {
    matches!(
        key,
        "innerHTML"
            | "textContent"
            | "value"
            | "checked"
            | "selected"
            | "disabled"
            | "readOnly"
            | "multiple"
            | "indeterminate"
    )
}

/// Generate class binding
pub fn generate_class_binding(element_var: &str, value: &str, is_static: bool) -> String {
    if is_static {
        format!("{}.className = \"{}\"", element_var, value)
    } else {
        format!("_setClass({}, {})", element_var, value)
    }
}

/// Generate style binding
pub fn generate_style_binding(element_var: &str, value: &str, is_static: bool) -> String {
    if is_static {
        format!("{}.style.cssText = \"{}\"", element_var, value)
    } else {
        format!("_setStyle({}, {})", element_var, value)
    }
}

/// Generate attribute binding
pub fn generate_attribute(element_var: &str, name: &str, value: &str) -> String {
    format!("{}.setAttribute(\"{}\", {})", element_var, name, value)
}

/// Generate prop binding for component
pub fn generate_component_prop(component_var: &str, key: &str, value: &str) -> String {
    format!("{}.$props.{} = {}", component_var, key, value)
}

/// Normalize prop key for components
pub fn normalize_prop_key(key: &str) -> String {
    // Convert kebab-case to camelCase
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in key.chars() {
        if c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dom_prop() {
        assert!(is_dom_prop("value"));
        assert!(is_dom_prop("innerHTML"));
        assert!(!is_dom_prop("class"));
        assert!(!is_dom_prop("id"));
    }

    #[test]
    fn test_normalize_prop_key() {
        assert_eq!(normalize_prop_key("foo-bar"), "fooBar");
        assert_eq!(normalize_prop_key("foo-bar-baz"), "fooBarBaz");
        assert_eq!(normalize_prop_key("foo"), "foo");
    }

    #[test]
    fn test_generate_class_binding_static() {
        let result = generate_class_binding("_n1", "active", true);
        assert_eq!(result, "_n1.className = \"active\"");
    }
}
