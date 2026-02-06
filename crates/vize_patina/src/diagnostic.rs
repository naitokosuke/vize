//! Diagnostic types for vize_patina linter.
//!
//! Uses `CompactString` for efficient small string storage.

use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;
use serde::Serialize;
use vize_carton::CompactString;

/// Lint diagnostic severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

/// Help display level for diagnostics
///
/// Controls how much help text is included in diagnostics.
/// Useful for environments where markdown rendering is unavailable
/// or CLI output where verbose help is distracting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HelpLevel {
    /// No help text
    None,
    /// Short help text (first line only, markdown stripped)
    Short,
    /// Full help text with markdown formatting
    #[default]
    Full,
}

impl HelpLevel {
    /// Process help text according to this level
    pub fn process(&self, help: &str) -> Option<String> {
        match self {
            HelpLevel::None => None,
            HelpLevel::Short => Some(strip_markdown_first_line(help)),
            HelpLevel::Full => Some(help.to_string()),
        }
    }
}

/// Strip markdown formatting and return the first meaningful line.
fn strip_markdown_first_line(text: &str) -> String {
    let mut in_code_block = false;
    for line in text.lines() {
        let trimmed = line.trim();
        // Track code fence blocks
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        // Skip lines inside code blocks
        if in_code_block {
            continue;
        }
        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }
        // Strip markdown bold/italic
        let stripped = trimmed.replace("**", "").replace("__", "").replace('`', "");
        // Skip lines that are just markdown headers
        let stripped = stripped.trim_start_matches('#').trim();
        if stripped.is_empty() {
            continue;
        }
        return stripped.to_string();
    }
    text.lines().next().unwrap_or(text).to_string()
}

/// A text edit for auto-fixing a diagnostic.
///
/// Represents a single text replacement in the source code.
#[derive(Debug, Clone, Serialize)]
pub struct TextEdit {
    /// Start byte offset
    pub start: u32,
    /// End byte offset
    pub end: u32,
    /// Replacement text
    pub new_text: String,
}

impl TextEdit {
    /// Create a new text edit
    #[inline]
    pub fn new(start: u32, end: u32, new_text: impl Into<String>) -> Self {
        Self {
            start,
            end,
            new_text: new_text.into(),
        }
    }

    /// Create an insertion edit
    #[inline]
    pub fn insert(offset: u32, text: impl Into<String>) -> Self {
        Self::new(offset, offset, text)
    }

    /// Create a deletion edit
    #[inline]
    pub fn delete(start: u32, end: u32) -> Self {
        Self::new(start, end, "")
    }

    /// Create a replacement edit
    #[inline]
    pub fn replace(start: u32, end: u32, text: impl Into<String>) -> Self {
        Self::new(start, end, text)
    }
}

/// A fix for a diagnostic, containing one or more text edits.
#[derive(Debug, Clone, Serialize)]
pub struct Fix {
    /// Description of the fix
    pub message: String,
    /// Text edits to apply
    pub edits: Vec<TextEdit>,
}

impl Fix {
    /// Create a new fix with a single edit
    #[inline]
    pub fn new(message: impl Into<String>, edit: TextEdit) -> Self {
        Self {
            message: message.into(),
            edits: vec![edit],
        }
    }

    /// Create a new fix with multiple edits
    #[inline]
    pub fn with_edits(message: impl Into<String>, edits: Vec<TextEdit>) -> Self {
        Self {
            message: message.into(),
            edits,
        }
    }

    /// Apply the fix to a source string
    #[inline]
    pub fn apply(&self, source: &str) -> String {
        let mut result = source.to_string();
        // Apply edits in reverse order to preserve offsets
        let mut edits = self.edits.clone();
        edits.sort_by(|a, b| b.start.cmp(&a.start));

        for edit in edits {
            let start = edit.start as usize;
            let end = edit.end as usize;
            if start <= result.len() && end <= result.len() {
                result.replace_range(start..end, &edit.new_text);
            }
        }
        result
    }
}

/// A lint diagnostic with rich information for display.
///
/// Uses `CompactString` for message storage - strings up to 24 bytes
/// are stored inline without heap allocation.
#[derive(Debug, Clone)]
pub struct LintDiagnostic {
    /// Rule that triggered this diagnostic
    pub rule_name: &'static str,
    /// Severity level
    pub severity: Severity,
    /// Primary message (CompactString for efficiency)
    pub message: CompactString,
    /// Start byte offset in source
    pub start: u32,
    /// End byte offset in source
    pub end: u32,
    /// Help message for fixing (optional, CompactString)
    pub help: Option<CompactString>,
    /// Related diagnostic information
    pub labels: Vec<Label>,
    /// Auto-fix for this diagnostic (optional)
    pub fix: Option<Fix>,
}

/// Additional label for a diagnostic
#[derive(Debug, Clone)]
pub struct Label {
    /// Message for this label (CompactString for efficiency)
    pub message: CompactString,
    /// Start byte offset
    pub start: u32,
    /// End byte offset
    pub end: u32,
}

impl LintDiagnostic {
    /// Create a new error diagnostic
    #[inline]
    pub fn error(
        rule_name: &'static str,
        message: impl Into<CompactString>,
        start: u32,
        end: u32,
    ) -> Self {
        Self {
            rule_name,
            severity: Severity::Error,
            message: message.into(),
            start,
            end,
            help: None,
            labels: Vec::new(),
            fix: None,
        }
    }

    /// Create a new warning diagnostic
    #[inline]
    pub fn warn(
        rule_name: &'static str,
        message: impl Into<CompactString>,
        start: u32,
        end: u32,
    ) -> Self {
        Self {
            rule_name,
            severity: Severity::Warning,
            message: message.into(),
            start,
            end,
            help: None,
            labels: Vec::new(),
            fix: None,
        }
    }

    /// Add a help message
    #[inline]
    pub fn with_help(mut self, help: impl Into<CompactString>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Add a related label
    #[inline]
    pub fn with_label(mut self, message: impl Into<CompactString>, start: u32, end: u32) -> Self {
        self.labels.push(Label {
            message: message.into(),
            start,
            end,
        });
        self
    }

    /// Add a fix for this diagnostic
    #[inline]
    pub fn with_fix(mut self, fix: Fix) -> Self {
        self.fix = Some(fix);
        self
    }

    /// Check if this diagnostic has a fix
    #[inline]
    pub fn has_fix(&self) -> bool {
        self.fix.is_some()
    }

    /// Get the formatted message with `[vize:RULE]` prefix.
    #[inline]
    pub fn formatted_message(&self) -> String {
        format!("[vize:{}] {}", self.rule_name, self.message)
    }

    /// Convert to OxcDiagnostic for rich rendering
    #[inline]
    pub fn into_oxc_diagnostic(self) -> OxcDiagnostic {
        // Format message with [vize:RULE] prefix
        let formatted_msg = format!("[vize:{}] {}", self.rule_name, self.message);

        let mut diag = match self.severity {
            Severity::Error => OxcDiagnostic::error(formatted_msg),
            Severity::Warning => OxcDiagnostic::warn(formatted_msg),
        };

        // Add primary label
        diag = diag.with_label(Span::new(self.start, self.end));

        // Add help if present
        if let Some(help) = self.help {
            diag = diag.with_help(help.to_string());
        }

        // Add additional labels
        for label in self.labels {
            diag =
                diag.and_label(Span::new(label.start, label.end).label(label.message.to_string()));
        }

        diag
    }
}

/// Summary of lint results
#[derive(Debug, Clone, Default, Serialize)]
pub struct LintSummary {
    pub error_count: usize,
    pub warning_count: usize,
    pub file_count: usize,
}

impl LintSummary {
    #[inline]
    pub fn add(&mut self, diagnostic: &LintDiagnostic) {
        match diagnostic.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
        }
    }

    #[inline]
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_level_full() {
        let level = HelpLevel::Full;
        let help = "**Why:** Use `:key` for tracking.\n\n```vue\n<li :key=\"id\">\n```";
        let result = level.process(help);
        assert_eq!(result, Some(help.to_string()));
    }

    #[test]
    fn test_help_level_none() {
        let level = HelpLevel::None;
        let result = level.process("Any help text");
        assert_eq!(result, None);
    }

    #[test]
    fn test_help_level_short_strips_markdown() {
        let level = HelpLevel::Short;
        let help = "**Why:** The `:key` attribute helps Vue track items.\n\n**Fix:**\n```vue\n<li :key=\"id\">\n```";
        let result = level.process(help);
        assert_eq!(
            result,
            Some("Why: The :key attribute helps Vue track items.".to_string())
        );
    }

    #[test]
    fn test_help_level_short_skips_code_blocks() {
        let level = HelpLevel::Short;
        let help = "```vue\n<li :key=\"id\">\n```\nUse unique keys";
        let result = level.process(help);
        assert_eq!(result, Some("Use unique keys".to_string()));
    }

    #[test]
    fn test_help_level_short_simple_text() {
        let level = HelpLevel::Short;
        let help = "Add a key attribute to the element";
        let result = level.process(help);
        assert_eq!(
            result,
            Some("Add a key attribute to the element".to_string())
        );
    }

    #[test]
    fn test_strip_markdown_first_line_with_backticks() {
        let result = strip_markdown_first_line("Use `v-model` instead of `{{ }}`");
        assert_eq!(result, "Use v-model instead of {{ }}");
    }
}
