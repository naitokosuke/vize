//! Compiler options.

use vize_allocator::String;

/// Parse mode for the tokenizer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParseMode {
    /// Platform-agnostic mode
    #[default]
    Base,
    /// HTML mode with special handling for certain tags
    Html,
    /// SFC mode for parsing .vue files
    Sfc,
}

/// Text mode for different contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextMode {
    /// Normal text parsing (default)
    #[default]
    Data,
    /// RCDATA (e.g., textarea, title)
    RcData,
    /// Raw text (e.g., script, style)
    RawText,
    /// CDATA section
    CData,
    /// Attribute value
    AttributeValue,
}

/// Parser options
#[derive(Debug, Clone)]
pub struct ParserOptions {
    /// Parse mode
    pub mode: ParseMode,
    /// Whether to trim whitespace
    pub whitespace: WhitespaceStrategy,
    /// Custom delimiters for interpolation (default: ["{{", "}}"])
    pub delimiters: (String, String),
    /// Whether in pre tag
    pub is_pre_tag: fn(&str) -> bool,
    /// Whether is a native tag
    pub is_native_tag: Option<fn(&str) -> bool>,
    /// Whether is a custom element
    pub is_custom_element: Option<fn(&str) -> bool>,
    /// Whether is a void tag
    pub is_void_tag: fn(&str) -> bool,
    /// Get the namespace for a tag
    pub get_namespace: fn(&str, Option<&str>) -> crate::Namespace,
    /// Error handler
    pub on_error: Option<fn(crate::CompilerError)>,
    /// Warning handler
    pub on_warn: Option<fn(crate::CompilerError)>,
    /// Enable comment preservation
    pub comments: bool,
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            mode: ParseMode::Base,
            whitespace: WhitespaceStrategy::Condense,
            delimiters: (String::from("{{"), String::from("}}")),
            is_pre_tag: |_| false,
            is_native_tag: None,
            is_custom_element: None,
            is_void_tag: vize_shared::is_void_tag,
            get_namespace: |_, _| crate::Namespace::Html,
            on_error: None,
            on_warn: None,
            comments: true,
        }
    }
}

/// Whitespace handling strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhitespaceStrategy {
    /// Condense whitespace (default)
    #[default]
    Condense,
    /// Preserve all whitespace
    Preserve,
}

/// Transform options
#[derive(Debug, Clone)]
pub struct TransformOptions {
    /// Filename for error messages
    pub filename: String,
    /// Whether to prefix identifiers
    pub prefix_identifiers: bool,
    /// Whether to hoist static nodes
    pub hoist_static: bool,
    /// Whether to cache handlers
    pub cache_handlers: bool,
    /// Scope ID for scoped CSS
    pub scope_id: Option<String>,
    /// Whether in SSR mode
    pub ssr: bool,
    /// Whether SSR optimize is enabled
    pub ssr_css_vars: Option<String>,
    /// Binding metadata from script setup
    pub binding_metadata: Option<BindingMetadata>,
    /// Inline mode
    pub inline: bool,
    /// Whether is TypeScript
    pub is_ts: bool,
}

impl Default for TransformOptions {
    fn default() -> Self {
        Self {
            filename: String::from("template.vue"),
            prefix_identifiers: false,
            hoist_static: false,
            cache_handlers: false,
            scope_id: None,
            ssr: false,
            ssr_css_vars: None,
            binding_metadata: None,
            inline: false,
            is_ts: false,
        }
    }
}

/// Binding metadata from script setup
#[derive(Debug, Clone, Default)]
pub struct BindingMetadata {
    /// Setup bindings
    pub bindings: rustc_hash::FxHashMap<String, BindingType>,
}

/// Binding type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingType {
    /// Variable declared with let/const in setup
    SetupLet,
    /// Const binding that may be a ref
    SetupMaybeRef,
    /// Const binding that is definitely a ref
    SetupRef,
    /// Reactive binding
    SetupReactiveConst,
    /// Const literal
    SetupConst,
    /// Binding from props
    Props,
    /// Binding from props with default
    PropsAliased,
    /// Data binding
    Data,
    /// Options binding
    Options,
    /// Literal constant
    LiteralConst,
}

/// Codegen options
#[derive(Debug, Clone)]
pub struct CodegenOptions {
    /// Output mode
    pub mode: CodegenMode,
    /// Whether to prefix identifiers
    pub prefix_identifiers: bool,
    /// Whether to generate source map
    pub source_map: bool,
    /// Filename for source map
    pub filename: String,
    /// Scope ID for scoped CSS
    pub scope_id: Option<String>,
    /// Whether in SSR mode
    pub ssr: bool,
    /// Whether SSR optimize is enabled
    pub optimize_imports: bool,
    /// Runtime module name
    pub runtime_module_name: String,
    /// Runtime global name
    pub runtime_global_name: String,
    /// Whether is TypeScript
    pub is_ts: bool,
    /// Inline mode
    pub inline: bool,
    /// Binding metadata from script setup
    pub binding_metadata: Option<BindingMetadata>,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            mode: CodegenMode::Function,
            prefix_identifiers: false,
            source_map: false,
            filename: String::from("template.vue"),
            scope_id: None,
            ssr: false,
            optimize_imports: false,
            runtime_module_name: String::from("vue"),
            runtime_global_name: String::from("Vue"),
            is_ts: false,
            inline: false,
            binding_metadata: None,
        }
    }
}

/// Codegen output mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CodegenMode {
    /// Generate a function (default)
    #[default]
    Function,
    /// Generate an ES module
    Module,
}

/// Combined compiler options
#[derive(Debug, Clone, Default)]
pub struct CompilerOptions {
    pub parser: ParserOptions,
    pub transform: TransformOptions,
    pub codegen: CodegenOptions,
}
