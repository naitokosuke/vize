//! Slot code generation for Vapor mode.

use super::block::GenerateContext;
use crate::ir::SlotOutletIRNode;

/// Generate SlotOutlet code
pub fn generate_slot_outlet(ctx: &mut GenerateContext, slot: &SlotOutletIRNode<'_>) {
    let temp = ctx.next_temp();
    let slot_name = if slot.name.is_static {
        format!("\"{}\"", slot.name.content)
    } else {
        slot.name.content.to_string()
    };

    // Generate props for slot
    let props = if slot.props.is_empty() {
        None
    } else {
        let prop_strs: Vec<String> = slot
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
        Some(format!("{{ {} }}", prop_strs.join(", ")))
    };

    // Generate fallback if present
    let has_fallback = slot.fallback.is_some();

    if let Some(props_str) = props {
        if has_fallback {
            ctx.push_line(&format!(
                "const {} = _renderSlot($slots, {}, {}, () => {{",
                temp, slot_name, props_str
            ));
            ctx.indent();
            // Fallback content would be generated here
            ctx.push_line("/* fallback content */");
            ctx.deindent();
            ctx.push_line("})");
        } else {
            ctx.push_line(&format!(
                "const {} = _renderSlot($slots, {}, {})",
                temp, slot_name, props_str
            ));
        }
    } else if has_fallback {
        ctx.push_line(&format!(
            "const {} = _renderSlot($slots, {}, {{}}, () => {{",
            temp, slot_name
        ));
        ctx.indent();
        ctx.push_line("/* fallback content */");
        ctx.deindent();
        ctx.push_line("})");
    } else {
        ctx.push_line(&format!(
            "const {} = _renderSlot($slots, {})",
            temp, slot_name
        ));
    }
}

/// Generate slot function for component
pub fn generate_slot_function(name: &str, params: Option<&str>, body: &str) -> String {
    if let Some(p) = params {
        format!("{}: ({}) => {}", name, p, body)
    } else {
        format!("{}: () => {}", name, body)
    }
}

/// Generate scoped slots object
pub fn generate_scoped_slots(slots: &[(String, Option<String>, String)]) -> String {
    let slot_strs: Vec<String> = slots
        .iter()
        .map(|(name, params, body)| generate_slot_function(name, params.as_deref(), body))
        .collect();

    format!("{{ {} }}", slot_strs.join(", "))
}

/// Generate slot props normalization
pub fn generate_normalize_slots(slots_expr: &str) -> String {
    format!("_normalizeSlots({})", slots_expr)
}

/// Generate dynamic slot name
pub fn generate_dynamic_slot_name(expr: &str) -> String {
    format!("[{}]", expr)
}

/// Check if slot is dynamic
pub fn is_dynamic_slot_name(name: &str) -> bool {
    name.starts_with('[') && name.ends_with(']')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_slot_function_no_params() {
        let result = generate_slot_function("default", None, "_n1");
        assert_eq!(result, "default: () => _n1");
    }

    #[test]
    fn test_generate_slot_function_with_params() {
        let result = generate_slot_function("item", Some("{ data }"), "_n1");
        assert_eq!(result, "item: ({ data }) => _n1");
    }

    #[test]
    fn test_is_dynamic_slot_name() {
        assert!(is_dynamic_slot_name("[slotName]"));
        assert!(!is_dynamic_slot_name("default"));
    }

    #[test]
    fn test_generate_dynamic_slot_name() {
        let result = generate_dynamic_slot_name("dynamicName");
        assert_eq!(result, "[dynamicName]");
    }
}
