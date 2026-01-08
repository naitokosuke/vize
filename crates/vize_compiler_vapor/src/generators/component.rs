//! Component code generation for Vapor mode.

use super::block::GenerateContext;
use crate::ir::CreateComponentIRNode;

/// Generate CreateComponent code
pub fn generate_create_component(ctx: &mut GenerateContext, component: &CreateComponentIRNode<'_>) {
    let temp = ctx.next_temp();
    let tag = &component.tag;

    // Generate props object
    let props = if component.props.is_empty() {
        String::from("{}")
    } else {
        let prop_strs: Vec<String> = component
            .props
            .iter()
            .map(|p| {
                let key = &p.key.content;
                let value = if let Some(first) = p.values.first() {
                    if first.is_static {
                        format!("\"{}\"", first.content)
                    } else {
                        first.content.to_string()
                    }
                } else {
                    String::from("undefined")
                };
                format!("{}: {}", key, value)
            })
            .collect();
        format!("{{ {} }}", prop_strs.join(", "))
    };

    // Generate slots if present
    let slots_code = if component.slots.is_empty() {
        None
    } else {
        Some(generate_slots_object(component))
    };

    if let Some(slots) = slots_code {
        ctx.push_line(&format!(
            "const {} = _createComponent({}, {}, {})",
            temp, tag, props, slots
        ));
    } else {
        ctx.push_line(&format!(
            "const {} = _createComponent({}, {})",
            temp, tag, props
        ));
    }
}

/// Generate slots object for component
fn generate_slots_object(component: &CreateComponentIRNode<'_>) -> String {
    let slot_strs: Vec<String> = component
        .slots
        .iter()
        .map(|slot| {
            let name = if slot.name.is_static {
                slot.name.content.to_string()
            } else {
                format!("[{}]", slot.name.content)
            };

            let params = slot
                .fn_exp
                .as_ref()
                .map(|p| p.content.to_string())
                .unwrap_or_default();

            format!("{}: ({}) => {{ /* slot content */ }}", name, params)
        })
        .collect();

    format!("{{ {} }}", slot_strs.join(", "))
}

/// Generate component resolution
pub fn generate_resolve_component(name: &str) -> String {
    format!("_resolveComponent(\"{}\")", name)
}

/// Generate dynamic component
pub fn generate_dynamic_component(
    component_expr: &str,
    props: &str,
    slots: Option<&str>,
) -> String {
    if let Some(slots_code) = slots {
        format!(
            "_createComponent({}, {}, {})",
            component_expr, props, slots_code
        )
    } else {
        format!("_createComponent({}, {})", component_expr, props)
    }
}

/// Generate async component wrapper
pub fn generate_async_component(component_expr: &str) -> String {
    format!("_defineAsyncComponent(() => {})", component_expr)
}

/// Generate suspense boundary
pub fn generate_suspense(fallback: Option<&str>) -> (String, String) {
    if let Some(fb) = fallback {
        (
            format!("_createSuspense({{ fallback: () => {} }})", fb),
            String::from("})"),
        )
    } else {
        (String::from("_createSuspense({"), String::from("})"))
    }
}

/// Generate keep-alive wrapper
pub fn generate_keep_alive(
    include: Option<&str>,
    exclude: Option<&str>,
    max: Option<usize>,
) -> String {
    let mut options = Vec::new();

    if let Some(inc) = include {
        options.push(format!("include: {}", inc));
    }
    if let Some(exc) = exclude {
        options.push(format!("exclude: {}", exc));
    }
    if let Some(m) = max {
        options.push(format!("max: {}", m));
    }

    if options.is_empty() {
        String::from("_createKeepAlive({})")
    } else {
        format!("_createKeepAlive({{ {} }})", options.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_resolve_component() {
        let result = generate_resolve_component("MyComponent");
        assert_eq!(result, "_resolveComponent(\"MyComponent\")");
    }

    #[test]
    fn test_generate_keep_alive_empty() {
        let result = generate_keep_alive(None, None, None);
        assert_eq!(result, "_createKeepAlive({})");
    }

    #[test]
    fn test_generate_keep_alive_with_options() {
        let result = generate_keep_alive(Some("\"Component1\""), None, Some(10));
        assert!(result.contains("include: \"Component1\""));
        assert!(result.contains("max: 10"));
    }
}
