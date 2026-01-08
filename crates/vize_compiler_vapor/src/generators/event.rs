//! Event code generation for Vapor mode.

use super::block::GenerateContext;
use crate::ir::{EventModifiers, SetEventIRNode};

/// Generate SetEvent code
pub fn generate_set_event(ctx: &mut GenerateContext, set_event: &SetEventIRNode<'_>) {
    let element = format!("_n{}", set_event.element);
    let event_name = &set_event.key.content;

    let handler = if let Some(ref value) = set_event.value {
        if value.is_static {
            format!("\"{}\"", value.content)
        } else {
            value.content.to_string()
        }
    } else {
        String::from("() => {}")
    };

    // Apply modifiers if present
    let final_handler = apply_modifiers(&handler, &set_event.modifiers);

    ctx.push_line(&format!(
        "_on({}, \"{}\", {})",
        element, event_name, final_handler
    ));
}

/// Apply event modifiers to handler
fn apply_modifiers(handler: &str, modifiers: &EventModifiers) -> String {
    let mut result = handler.to_string();

    // Apply key modifiers
    if !modifiers.keys.is_empty() {
        let keys: Vec<String> = modifiers
            .keys
            .iter()
            .map(|k| format!("\"{}\"", k))
            .collect();
        result = format!("_withKeys({}, [{}])", result, keys.join(", "));
    }

    // Apply non-key modifiers
    if !modifiers.non_keys.is_empty() {
        let mods: Vec<String> = modifiers
            .non_keys
            .iter()
            .map(|m| format!("\"{}\"", m))
            .collect();
        result = format!("_withModifiers({}, [{}])", result, mods.join(", "));
    }

    result
}

/// Generate event options
pub fn generate_event_options(modifiers: &EventModifiers) -> Option<String> {
    let options = &modifiers.options;

    if !options.capture && !options.once && !options.passive {
        return None;
    }

    let mut parts = Vec::new();

    if options.capture {
        parts.push("capture: true");
    }
    if options.once {
        parts.push("once: true");
    }
    if options.passive {
        parts.push("passive: true");
    }

    Some(format!("{{ {} }}", parts.join(", ")))
}

/// Generate delegate event handler
pub fn generate_delegate_event(
    element_var: &str,
    event_name: &str,
    handler: &str,
    options: Option<&str>,
) -> String {
    if let Some(opts) = options {
        format!(
            "_delegate({}, \"{}\", {}, {})",
            element_var, event_name, handler, opts
        )
    } else {
        format!(
            "_delegate({}, \"{}\", {})",
            element_var, event_name, handler
        )
    }
}

/// Generate inline event handler
pub fn generate_inline_handler(element_var: &str, event_name: &str, handler: &str) -> String {
    format!(
        "{}.addEventListener(\"{}\", {})",
        element_var, event_name, handler
    )
}

/// Capitalize event name for onEvent format
pub fn capitalize_event_name(event: &str) -> String {
    let mut chars = event.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => format!("on{}{}", first.to_uppercase(), chars.as_str()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_modifiers_none() {
        let modifiers = EventModifiers::default();
        let result = apply_modifiers("handleClick", &modifiers);
        assert_eq!(result, "handleClick");
    }

    #[test]
    fn test_capitalize_event_name() {
        assert_eq!(capitalize_event_name("click"), "onClick");
        assert_eq!(capitalize_event_name("keydown"), "onKeydown");
    }

    #[test]
    fn test_generate_event_options_none() {
        let modifiers = EventModifiers::default();
        assert_eq!(generate_event_options(&modifiers), None);
    }
}
