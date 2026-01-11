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

    /// Convert to OxcDiagnostic for rich rendering
    #[inline]
    pub fn into_oxc_diagnostic(self) -> OxcDiagnostic {
        let mut diag = match self.severity {
            Severity::Error => OxcDiagnostic::error(self.message.to_string()),
            Severity::Warning => OxcDiagnostic::warn(self.message.to_string()),
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
