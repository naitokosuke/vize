//! vue/component-name-in-template-casing
//!
//! Enforce specific casing for component names in templates.
//!
//! ## Examples
//!
//! ### Invalid (default: PascalCase)
//! ```vue
//! <my-component />
//! <myComponent />
//! ```
//!
//! ### Valid
//! ```vue
//! <MyComponent />
//! <RouterView />
//! <slot />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/component-name-in-template-casing",
    description: "Enforce specific casing for component names in templates",
    category: RuleCategory::Recommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Known HTML elements that should not be treated as components
const HTML_ELEMENTS: &[&str] = &[
    "a",
    "abbr",
    "address",
    "area",
    "article",
    "aside",
    "audio",
    "b",
    "base",
    "bdi",
    "bdo",
    "blockquote",
    "body",
    "br",
    "button",
    "canvas",
    "caption",
    "cite",
    "code",
    "col",
    "colgroup",
    "data",
    "datalist",
    "dd",
    "del",
    "details",
    "dfn",
    "dialog",
    "div",
    "dl",
    "dt",
    "em",
    "embed",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hgroup",
    "hr",
    "html",
    "i",
    "iframe",
    "img",
    "input",
    "ins",
    "kbd",
    "label",
    "legend",
    "li",
    "link",
    "main",
    "map",
    "mark",
    "menu",
    "meta",
    "meter",
    "nav",
    "noscript",
    "object",
    "ol",
    "optgroup",
    "option",
    "output",
    "p",
    "param",
    "picture",
    "pre",
    "progress",
    "q",
    "rp",
    "rt",
    "ruby",
    "s",
    "samp",
    "script",
    "section",
    "select",
    "slot",
    "small",
    "source",
    "span",
    "strong",
    "style",
    "sub",
    "summary",
    "sup",
    "table",
    "tbody",
    "td",
    "template",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "time",
    "title",
    "tr",
    "track",
    "u",
    "ul",
    "var",
    "video",
    "wbr",
];

/// SVG elements
const SVG_ELEMENTS: &[&str] = &[
    "svg",
    "animate",
    "animateMotion",
    "animateTransform",
    "circle",
    "clipPath",
    "defs",
    "desc",
    "ellipse",
    "feBlend",
    "feColorMatrix",
    "feComponentTransfer",
    "feComposite",
    "feConvolveMatrix",
    "feDiffuseLighting",
    "feDisplacementMap",
    "feDistantLight",
    "feDropShadow",
    "feFlood",
    "feFuncA",
    "feFuncB",
    "feFuncG",
    "feFuncR",
    "feGaussianBlur",
    "feImage",
    "feMerge",
    "feMergeNode",
    "feMorphology",
    "feOffset",
    "fePointLight",
    "feSpecularLighting",
    "feSpotLight",
    "feTile",
    "feTurbulence",
    "filter",
    "foreignObject",
    "g",
    "image",
    "line",
    "linearGradient",
    "marker",
    "mask",
    "metadata",
    "mpath",
    "path",
    "pattern",
    "polygon",
    "polyline",
    "radialGradient",
    "rect",
    "set",
    "stop",
    "switch",
    "symbol",
    "text",
    "textPath",
    "tspan",
    "use",
    "view",
];

/// Vue built-in components
const VUE_BUILT_IN: &[&str] = &[
    "component",
    "transition",
    "transition-group",
    "keep-alive",
    "slot",
    "teleport",
    "suspense",
];

/// Casing style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComponentCasing {
    /// PascalCase: MyComponent
    #[default]
    PascalCase,
    /// kebab-case: my-component
    KebabCase,
}

/// Component name in template casing rule
pub struct ComponentNameInTemplateCasing {
    pub casing: ComponentCasing,
}

impl Default for ComponentNameInTemplateCasing {
    fn default() -> Self {
        Self {
            casing: ComponentCasing::PascalCase,
        }
    }
}

impl ComponentNameInTemplateCasing {
    fn is_html_element(tag: &str) -> bool {
        HTML_ELEMENTS.contains(&tag.to_lowercase().as_str())
    }

    fn is_svg_element(tag: &str) -> bool {
        SVG_ELEMENTS.contains(&tag)
    }

    fn is_vue_built_in(tag: &str) -> bool {
        VUE_BUILT_IN.contains(&tag.to_lowercase().as_str())
    }

    fn is_pascal_case(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        let first_char = name.chars().next().unwrap();
        first_char.is_uppercase() && !name.contains('-')
    }

    fn is_kebab_case(name: &str) -> bool {
        name.chars().all(|c| c.is_lowercase() || c == '-')
    }

    fn to_pascal_case(name: &str) -> String {
        let mut result = String::with_capacity(name.len());
        let mut capitalize_next = true;
        for c in name.chars() {
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

    fn to_kebab_case(name: &str) -> String {
        let mut result = String::with_capacity(name.len() + 4);
        for (i, c) in name.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                result.push('-');
                result.push(c.to_ascii_lowercase());
            } else {
                result.push(c.to_ascii_lowercase());
            }
        }
        result
    }
}

impl Rule for ComponentNameInTemplateCasing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        let tag = element.tag.as_str();

        // Skip HTML elements, SVG elements, and Vue built-ins
        if Self::is_html_element(tag) || Self::is_svg_element(tag) || Self::is_vue_built_in(tag) {
            return;
        }

        match self.casing {
            ComponentCasing::PascalCase => {
                if !Self::is_pascal_case(tag) {
                    let pascal = Self::to_pascal_case(tag);
                    ctx.warn_with_help(
                        format!("Component `<{}>` should use PascalCase", tag),
                        &element.loc,
                        format!("Use `<{}>`", pascal),
                    );
                }
            }
            ComponentCasing::KebabCase => {
                if !Self::is_kebab_case(tag) {
                    let kebab = Self::to_kebab_case(tag);
                    ctx.warn_with_help(
                        format!("Component `<{}>` should use kebab-case", tag),
                        &element.loc,
                        format!("Use `<{}>`", kebab),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ComponentNameInTemplateCasing::default()));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_pascal_case() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_kebab_case() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<my-component />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_html_element() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_vue_built_in() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<slot />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
