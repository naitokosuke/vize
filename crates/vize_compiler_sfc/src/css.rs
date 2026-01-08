//! CSS compilation using LightningCSS.
//!
//! Provides high-performance CSS parsing, transformation, and minification.
//! When the `native` feature is disabled (e.g., for wasm builds), a simple
//! passthrough implementation is used.

#[cfg(feature = "native")]
use lightningcss::printer::PrinterOptions;
#[cfg(feature = "native")]
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
#[cfg(feature = "native")]
use lightningcss::targets::{Browsers, Targets};
use serde::{Deserialize, Serialize};

use crate::types::SfcStyleBlock;

/// CSS compilation options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssCompileOptions {
    /// Scope ID for scoped CSS (e.g., "data-v-abc123")
    #[serde(default)]
    pub scope_id: Option<String>,

    /// Whether to apply scoped CSS transformation
    #[serde(default)]
    pub scoped: bool,

    /// Whether to minify the output
    #[serde(default)]
    pub minify: bool,

    /// Whether to generate source maps
    #[serde(default)]
    pub source_map: bool,

    /// Browser targets for autoprefixing
    #[serde(default)]
    pub targets: Option<CssTargets>,

    /// Filename for error reporting
    #[serde(default)]
    pub filename: Option<String>,
}

/// Browser targets for CSS autoprefixing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssTargets {
    #[serde(default)]
    pub chrome: Option<u32>,
    #[serde(default)]
    pub firefox: Option<u32>,
    #[serde(default)]
    pub safari: Option<u32>,
    #[serde(default)]
    pub edge: Option<u32>,
    #[serde(default)]
    pub ios: Option<u32>,
    #[serde(default)]
    pub android: Option<u32>,
}

#[cfg(feature = "native")]
impl CssTargets {
    fn to_lightningcss_targets(&self) -> Targets {
        let mut browsers = Browsers::default();

        if let Some(v) = self.chrome {
            browsers.chrome = Some(version_to_u32(v));
        }
        if let Some(v) = self.firefox {
            browsers.firefox = Some(version_to_u32(v));
        }
        if let Some(v) = self.safari {
            browsers.safari = Some(version_to_u32(v));
        }
        if let Some(v) = self.edge {
            browsers.edge = Some(version_to_u32(v));
        }
        if let Some(v) = self.ios {
            browsers.ios_saf = Some(version_to_u32(v));
        }
        if let Some(v) = self.android {
            browsers.android = Some(version_to_u32(v));
        }

        Targets::from(browsers)
    }
}

/// Convert major version to LightningCSS format (major << 16)
#[cfg(feature = "native")]
fn version_to_u32(major: u32) -> u32 {
    major << 16
}

/// CSS compilation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssCompileResult {
    /// Compiled CSS code
    pub code: String,

    /// Source map (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map: Option<String>,

    /// CSS variables found (from v-bind())
    #[serde(default)]
    pub css_vars: Vec<String>,

    /// Errors during compilation
    #[serde(default)]
    pub errors: Vec<String>,

    /// Warnings during compilation
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Compile CSS using LightningCSS (native feature enabled)
#[cfg(feature = "native")]
pub fn compile_css(css: &str, options: &CssCompileOptions) -> CssCompileResult {
    let filename = options
        .filename
        .clone()
        .unwrap_or_else(|| "style.css".to_string());

    // Extract v-bind() expressions before parsing
    let (processed_css, css_vars) = extract_and_transform_v_bind(css);

    // Apply scoped transformation if needed
    let scoped_css = if options.scoped {
        if let Some(ref scope_id) = options.scope_id {
            apply_scoped_css_lightningcss(&processed_css, scope_id)
        } else {
            processed_css
        }
    } else {
        processed_css
    };

    // Apply targets for autoprefixing
    let targets = options
        .targets
        .as_ref()
        .map(|t| t.to_lightningcss_targets())
        .unwrap_or_default();

    // Parse and process CSS
    let result = compile_css_internal(&scoped_css, &filename, options.minify, targets);

    CssCompileResult {
        code: result.0,
        map: None,
        css_vars,
        errors: result.1,
        warnings: vec![],
    }
}

/// Compile CSS (wasm fallback - no LightningCSS)
#[cfg(not(feature = "native"))]
pub fn compile_css(css: &str, options: &CssCompileOptions) -> CssCompileResult {
    // Extract v-bind() expressions before parsing
    let (processed_css, css_vars) = extract_and_transform_v_bind(css);

    // Apply scoped transformation if needed
    let scoped_css = if options.scoped {
        if let Some(ref scope_id) = options.scope_id {
            apply_scoped_css_lightningcss(&processed_css, scope_id)
        } else {
            processed_css
        }
    } else {
        processed_css
    };

    CssCompileResult {
        code: scoped_css,
        map: None,
        css_vars,
        errors: vec![],
        warnings: vec![],
    }
}

/// Internal CSS compilation with owned strings to avoid borrow issues
#[cfg(feature = "native")]
fn compile_css_internal(
    css: &str,
    filename: &str,
    minify: bool,
    targets: Targets,
) -> (String, Vec<String>) {
    let parser_options = ParserOptions {
        filename: filename.to_string(),
        ..Default::default()
    };

    let mut stylesheet = match StyleSheet::parse(css, parser_options) {
        Ok(ss) => ss,
        Err(e) => {
            return (css.to_string(), vec![format!("CSS parse error: {}", e)]);
        }
    };

    // Minify if requested
    if minify {
        if let Err(e) = stylesheet.minify(lightningcss::stylesheet::MinifyOptions {
            targets: targets,
            ..Default::default()
        }) {
            return (css.to_string(), vec![format!("CSS minify error: {:?}", e)]);
        }
    }

    // Print the CSS
    let printer_options = PrinterOptions {
        minify,
        targets,
        ..Default::default()
    };

    match stylesheet.to_css(printer_options) {
        Ok(result) => (result.code, vec![]),
        Err(e) => (css.to_string(), vec![format!("CSS print error: {:?}", e)]),
    }
}

/// Compile a style block
pub fn compile_style_block(style: &SfcStyleBlock, options: &CssCompileOptions) -> CssCompileResult {
    let mut opts = options.clone();
    opts.scoped = style.scoped || opts.scoped;
    compile_css(&style.content, &opts)
}

/// Extract v-bind() expressions and transform them to CSS variables
fn extract_and_transform_v_bind(css: &str) -> (String, Vec<String>) {
    let mut vars = Vec::new();
    let mut result = css.to_string();
    let mut search_from = 0;

    while let Some(pos) = result[search_from..].find("v-bind(") {
        let actual_pos = search_from + pos;
        let start = actual_pos + 7;

        if let Some(end) = result[start..].find(')') {
            let expr = result[start..start + end].trim();
            // Remove quotes if present
            let expr = expr.trim_matches(|c| c == '"' || c == '\'');
            vars.push(expr.to_string());

            // Transform v-bind(expr) to var(--hash-expr)
            let var_name = format!("--{}", hash_v_bind_var(expr));
            let replacement = format!("var({})", var_name);
            result = format!(
                "{}{}{}",
                &result[..actual_pos],
                replacement,
                &result[start + end + 1..]
            );

            search_from = actual_pos + replacement.len();
        } else {
            break;
        }
    }

    (result, vars)
}

/// Hash a v-bind variable name for CSS variable
fn hash_v_bind_var(expr: &str) -> String {
    // Simple hash - in production, this should match Vue's hashing
    let hash: u32 = expr
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    format!(
        "{:08x}-{}",
        hash,
        expr.replace(['.', '[', ']', '(', ')'], "_")
    )
}

/// Apply scoped CSS transformation using string manipulation
/// (LightningCSS doesn't have built-in scoping, so we do it manually)
fn apply_scoped_css_lightningcss(css: &str, scope_id: &str) -> String {
    let attr_selector = format!("[{}]", scope_id);
    let mut output = String::with_capacity(css.len() * 2);
    let mut chars = css.chars().peekable();
    let mut in_selector = true;
    let mut in_string = false;
    let mut string_char = '"';
    let mut in_comment = false;
    let mut brace_depth = 0;
    let mut last_selector_end = 0;
    let mut current = String::new();
    let mut in_at_rule = false;
    let mut at_rule_depth = 0;

    while let Some(c) = chars.next() {
        current.push(c);

        if in_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                current.push(chars.next().unwrap());
                in_comment = false;
            }
            continue;
        }

        if in_string {
            if c == string_char && !current.ends_with("\\\"") && !current.ends_with("\\'") {
                in_string = false;
            }
            if !in_selector {
                output.push(c);
            }
            continue;
        }

        match c {
            '"' | '\'' => {
                in_string = true;
                string_char = c;
                if !in_selector {
                    output.push(c);
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                current.push(chars.next().unwrap());
                in_comment = true;
            }
            '@' => {
                in_at_rule = true;
                output.push(c);
            }
            '{' => {
                brace_depth += 1;
                if in_at_rule {
                    at_rule_depth = brace_depth;
                    in_at_rule = false;
                    output.push(c);
                } else if in_selector && brace_depth == 1 {
                    // End of selector, apply scope
                    let selector_part = &current[last_selector_end..current.len() - 1];
                    output.push_str(&scope_selector(selector_part.trim(), &attr_selector));
                    output.push('{');
                    in_selector = false;
                    last_selector_end = current.len();
                } else if in_selector && brace_depth > at_rule_depth {
                    // Nested rule selector
                    let selector_part = &current[last_selector_end..current.len() - 1];
                    output.push_str(&scope_selector(selector_part.trim(), &attr_selector));
                    output.push('{');
                    in_selector = false;
                    last_selector_end = current.len();
                } else {
                    output.push(c);
                }
            }
            '}' => {
                brace_depth -= 1;
                output.push(c);
                if brace_depth == 0 || (at_rule_depth > 0 && brace_depth == at_rule_depth - 1) {
                    in_selector = true;
                    last_selector_end = current.len();
                    if brace_depth < at_rule_depth {
                        at_rule_depth = 0;
                    }
                }
            }
            _ if in_selector => {
                // Still building selector
            }
            _ => {
                output.push(c);
            }
        }
    }

    // Handle any remaining content
    if !current[last_selector_end..].is_empty() && in_selector {
        output.push_str(&current[last_selector_end..]);
    }

    output
}

/// Add scope attribute to a selector
fn scope_selector(selector: &str, attr_selector: &str) -> String {
    if selector.is_empty() {
        return selector.to_string();
    }

    // Handle at-rules that don't have selectors
    if selector.starts_with('@') {
        return selector.to_string();
    }

    // Handle multiple selectors separated by comma
    selector
        .split(',')
        .map(|s| scope_single_selector(s.trim(), attr_selector))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Add scope attribute to a single selector
fn scope_single_selector(selector: &str, attr_selector: &str) -> String {
    if selector.is_empty() {
        return selector.to_string();
    }

    // Handle :deep(), :slotted(), :global()
    if selector.contains(":deep(") {
        return transform_deep(selector, attr_selector);
    }

    if selector.contains(":slotted(") {
        return transform_slotted(selector, attr_selector);
    }

    if selector.contains(":global(") {
        return transform_global(selector);
    }

    // Find the last simple selector to append the attribute
    let parts: Vec<&str> = selector.split_whitespace().collect();
    if parts.is_empty() {
        return selector.to_string();
    }

    // Add scope to the last part
    let mut result = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            result.push(' ');
        }

        if i == parts.len() - 1 {
            // Last part - add scope
            result.push_str(&add_scope_to_element(part, attr_selector));
        } else {
            result.push_str(part);
        }
    }

    result
}

/// Add scope attribute to an element selector
fn add_scope_to_element(selector: &str, attr_selector: &str) -> String {
    // Handle pseudo-elements (::before, ::after, etc.)
    if let Some(pseudo_pos) = selector.find("::") {
        let (before, after) = selector.split_at(pseudo_pos);
        return format!("{}{}{}", before, attr_selector, after);
    }

    // Handle pseudo-classes (:hover, :focus, etc.)
    if let Some(pseudo_pos) = selector.rfind(':') {
        let before = &selector[..pseudo_pos];
        if !before.is_empty() && !before.ends_with('\\') {
            let after = &selector[pseudo_pos..];
            return format!("{}{}{}", before, attr_selector, after);
        }
    }

    format!("{}{}", selector, attr_selector)
}

/// Transform :deep() to descendant selector
fn transform_deep(selector: &str, attr_selector: &str) -> String {
    if let Some(start) = selector.find(":deep(") {
        let before = &selector[..start];
        let after = &selector[start + 6..];

        if let Some(end) = find_matching_paren(after) {
            let inner = &after[..end];
            let rest = &after[end + 1..];

            let scoped_before = if before.is_empty() {
                attr_selector.to_string()
            } else {
                format!("{}{}", before.trim(), attr_selector)
            };

            return format!("{} {}{}", scoped_before, inner, rest);
        }
    }

    selector.to_string()
}

/// Transform :slotted() for slot content
fn transform_slotted(selector: &str, attr_selector: &str) -> String {
    if let Some(start) = selector.find(":slotted(") {
        let after = &selector[start + 9..];

        if let Some(end) = find_matching_paren(after) {
            let inner = &after[..end];
            let rest = &after[end + 1..];

            return format!("{}{}-s{}", inner, attr_selector, rest);
        }
    }

    selector.to_string()
}

/// Transform :global() to unscoped
fn transform_global(selector: &str) -> String {
    if let Some(start) = selector.find(":global(") {
        let before = &selector[..start];
        let after = &selector[start + 8..];

        if let Some(end) = find_matching_paren(after) {
            let inner = &after[..end];
            let rest = &after[end + 1..];

            return format!("{}{}{}", before, inner, rest);
        }
    }

    selector.to_string()
}

/// Find the matching closing parenthesis
fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 1;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_css() {
        let css = ".foo { color: red; }";
        let result = compile_css(css, &CssCompileOptions::default());
        assert!(result.errors.is_empty());
        assert!(result.code.contains(".foo"));
        assert!(result.code.contains("color"));
    }

    #[test]
    fn test_compile_scoped_css() {
        let css = ".foo { color: red; }";
        let result = compile_css(
            css,
            &CssCompileOptions {
                scoped: true,
                scope_id: Some("data-v-123".to_string()),
                ..Default::default()
            },
        );
        assert!(result.errors.is_empty());
        assert!(result.code.contains("[data-v-123]"));
    }

    #[test]
    #[cfg(feature = "native")]
    fn test_compile_minified_css() {
        let css = ".foo {\n  color: red;\n  background: blue;\n}";
        let result = compile_css(
            css,
            &CssCompileOptions {
                minify: true,
                ..Default::default()
            },
        );
        assert!(result.errors.is_empty());
        // Minified should have no newlines in simple case
        assert!(!result.code.contains('\n') || result.code.lines().count() == 1);
    }

    #[test]
    fn test_v_bind_extraction() {
        let css = ".foo { color: v-bind(color); background: v-bind('bgColor'); }";
        let (transformed, vars) = extract_and_transform_v_bind(css);
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"color".to_string()));
        assert!(vars.contains(&"bgColor".to_string()));
        assert!(transformed.contains("var(--"));
    }

    #[test]
    fn test_scope_deep() {
        let result = transform_deep(":deep(.child)", "[data-v-123]");
        assert_eq!(result, "[data-v-123] .child");
    }

    #[test]
    fn test_scope_global() {
        let result = transform_global(":global(.foo)");
        assert_eq!(result, ".foo");
    }

    #[test]
    fn test_scope_slotted() {
        let result = transform_slotted(":slotted(.child)", "[data-v-123]");
        assert_eq!(result, ".child[data-v-123]-s");
    }

    #[test]
    fn test_scope_with_pseudo_element() {
        let result = add_scope_to_element(".foo::before", "[data-v-123]");
        assert_eq!(result, ".foo[data-v-123]::before");
    }

    #[test]
    fn test_scope_with_pseudo_class() {
        let result = add_scope_to_element(".foo:hover", "[data-v-123]");
        assert_eq!(result, ".foo[data-v-123]:hover");
    }

    #[test]
    #[cfg(feature = "native")]
    fn test_compile_with_targets() {
        let css = ".foo { display: flex; }";
        let result = compile_css(
            css,
            &CssCompileOptions {
                targets: Some(CssTargets {
                    chrome: Some(80),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        assert!(result.errors.is_empty());
        assert!(result.code.contains("flex"));
    }

    #[test]
    fn test_scoped_css_with_quoted_font_family() {
        let css = ".foo { font-family: 'JetBrains Mono', monospace; }";
        let result = compile_css(
            css,
            &CssCompileOptions {
                scoped: true,
                scope_id: Some("data-v-123".to_string()),
                ..Default::default()
            },
        );
        println!("Result: {}", result.code);
        assert!(result.errors.is_empty());
        // Note: LightningCSS may remove quotes from font names
        assert!(
            result.code.contains("JetBrains Mono"),
            "Expected font name in: {}",
            result.code
        );
        assert!(result.code.contains("monospace"));
    }

    #[test]
    fn test_apply_scoped_css_with_quoted_string() {
        // Test the raw scoping function without LightningCSS
        let css = ".foo { font-family: 'JetBrains Mono', monospace; }";
        let result = apply_scoped_css_lightningcss(css, "data-v-123");
        println!("Scoped result: {}", result);
        assert!(
            result.contains("'JetBrains Mono'"),
            "Expected quoted font name in: {}",
            result
        );
        assert!(result.contains("monospace"));
    }
}
