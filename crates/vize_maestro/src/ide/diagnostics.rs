//! Diagnostics aggregation from multiple sources.
//!
//! Aggregates diagnostics from:
//! - SFC parser errors
//! - Template parser errors
//! - vize_patina (linter)
//! - Future: vize_canon (type checker)

use tower_lsp::lsp_types::{
    CodeDescription, Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url,
};

use crate::server::ServerState;

/// Diagnostic source identifiers.
pub mod sources {
    pub const SFC_PARSER: &str = "vize/sfc";
    pub const TEMPLATE_PARSER: &str = "vize/template";
    pub const SCRIPT_PARSER: &str = "vize/script";
    pub const LINTER: &str = "vize/lint";
    pub const TYPE_CHECKER: &str = "vize/types";
    pub const MUSEA: &str = "vize/musea";
}

/// Diagnostic severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

impl From<Severity> for DiagnosticSeverity {
    fn from(s: Severity) -> Self {
        match s {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Information => DiagnosticSeverity::INFORMATION,
            Severity::Hint => DiagnosticSeverity::HINT,
        }
    }
}

/// Diagnostic service for collecting and aggregating diagnostics.
pub struct DiagnosticService;

impl DiagnosticService {
    /// Collect all diagnostics for a document.
    pub fn collect(state: &ServerState, uri: &Url) -> Vec<Diagnostic> {
        let Some(doc) = state.documents.get(uri) else {
            return vec![];
        };

        let content = doc.text();
        let mut diagnostics = Vec::new();

        // Check if this is an Art file (*.art.vue)
        let path = uri.path();
        if path.ends_with(".art.vue") {
            // Use Musea-specific diagnostics for Art files
            diagnostics.extend(Self::collect_musea_diagnostics(uri, &content));
            return diagnostics;
        }

        // Standard SFC processing
        // Collect SFC parser diagnostics
        diagnostics.extend(Self::collect_sfc_diagnostics(uri, &content));

        // Collect template parser diagnostics
        diagnostics.extend(Self::collect_template_diagnostics(uri, &content));

        // Collect linter diagnostics (vize_patina)
        diagnostics.extend(Self::collect_lint_diagnostics(uri, &content));

        // Collect type checker diagnostics (vize_canon)
        diagnostics.extend(super::TypeService::collect_diagnostics(state, uri));

        diagnostics
    }

    /// Collect diagnostics asynchronously (includes tsgo diagnostics when available).
    #[cfg(feature = "native")]
    pub async fn collect_async(state: &ServerState, uri: &Url) -> Vec<Diagnostic> {
        // Start with sync diagnostics (patina, etc.)
        let mut diagnostics = Self::collect(state, uri);

        // Try to get tsgo diagnostics (with timeout, skip on failure)
        let tsgo_future = Self::collect_tsgo_diagnostics(state, uri);
        if let Ok(tsgo_diags) =
            tokio::time::timeout(std::time::Duration::from_secs(5), tsgo_future).await
        {
            diagnostics.extend(tsgo_diags);
        }

        diagnostics
    }

    /// Collect diagnostics from tsgo LSP.
    #[cfg(feature = "native")]
    async fn collect_tsgo_diagnostics(state: &ServerState, uri: &Url) -> Vec<Diagnostic> {
        // Only process .vue files
        if !uri.path().ends_with(".vue") {
            return vec![];
        }

        // Get document content
        let Some(doc) = state.documents.get(uri) else {
            return vec![];
        };
        let content = doc.text();

        // Get tsgo bridge
        let Some(bridge) = state.get_tsgo_bridge().await else {
            return vec![];
        };

        // Generate virtual TypeScript
        let Some(virtual_ts) = Self::generate_virtual_ts(uri, &content) else {
            return vec![];
        };

        // Create virtual URI
        let virtual_uri = format!("file://{}.ts", uri.path());

        // Open document in tsgo
        if bridge
            .open_virtual_document(&virtual_uri, &virtual_ts)
            .await
            .is_err()
        {
            return vec![];
        }

        // Get diagnostics
        let Ok(tsgo_diags) = bridge.get_diagnostics(&virtual_uri).await else {
            return vec![];
        };

        // Convert to LSP diagnostics
        tsgo_diags
            .into_iter()
            .filter_map(|diag| {
                // Skip diagnostics in preamble (before script content)
                if diag.range.start.line < 20 {
                    return None;
                }

                Some(Diagnostic {
                    range: Range {
                        start: Position {
                            line: diag.range.start.line.saturating_sub(20),
                            character: diag.range.start.character,
                        },
                        end: Position {
                            line: diag.range.end.line.saturating_sub(20),
                            character: diag.range.end.character,
                        },
                    },
                    severity: diag.severity.map(|s| match s {
                        1 => DiagnosticSeverity::ERROR,
                        2 => DiagnosticSeverity::WARNING,
                        3 => DiagnosticSeverity::INFORMATION,
                        _ => DiagnosticSeverity::HINT,
                    }),
                    source: Some("vize/tsgo".to_string()),
                    message: diag.message,
                    ..Default::default()
                })
            })
            .collect()
    }

    /// Generate virtual TypeScript for a Vue SFC.
    #[cfg(feature = "native")]
    fn generate_virtual_ts(uri: &Url, content: &str) -> Option<String> {
        use vize_atelier_sfc::{parse_sfc, SfcParseOptions};
        use vize_canon::virtual_ts::generate_virtual_ts;
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let options = SfcParseOptions {
            filename: uri.path().to_string(),
            ..Default::default()
        };

        let descriptor = parse_sfc(content, options).ok()?;

        let script_content = descriptor
            .script_setup
            .as_ref()
            .map(|s| s.content.as_ref())
            .or_else(|| descriptor.script.as_ref().map(|s| s.content.as_ref()));

        let template_block = descriptor.template.as_ref()?;
        let template_offset = template_block.loc.start as u32;

        let allocator = vize_carton::Bump::new();
        let (template_ast, _) = vize_armature::parse(&allocator, &template_block.content);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        if let Some(script) = script_content {
            analyzer.analyze_script(script);
        }
        analyzer.analyze_template(&template_ast);

        let summary = analyzer.finish();
        Some(generate_virtual_ts(
            &summary,
            script_content,
            Some(&template_ast),
            template_offset,
        ))
    }

    /// Collect diagnostics for Art files (*.art.vue) using vize_patina's MuseaLinter.
    fn collect_musea_diagnostics(_uri: &Url, content: &str) -> Vec<Diagnostic> {
        use vize_patina::rules::musea::MuseaLinter;

        let linter = MuseaLinter::new();
        let result = linter.lint(content);

        result
            .diagnostics
            .into_iter()
            .map(|lint_diag| {
                // Convert byte offset to line/column
                let (start_line, start_col) = offset_to_line_col(content, lint_diag.start as usize);
                let (end_line, end_col) = offset_to_line_col(content, lint_diag.end as usize);

                // Build the diagnostic message with help text
                let message = if let Some(ref help) = lint_diag.help {
                    format!("{}\n\nHelp: {}", lint_diag.message, help)
                } else {
                    lint_diag.message.to_string()
                };

                Diagnostic {
                    range: Range {
                        start: Position {
                            line: start_line,
                            character: start_col,
                        },
                        end: Position {
                            line: end_line,
                            character: end_col,
                        },
                    },
                    severity: Some(match lint_diag.severity {
                        vize_patina::Severity::Error => DiagnosticSeverity::ERROR,
                        vize_patina::Severity::Warning => DiagnosticSeverity::WARNING,
                    }),
                    code: Some(NumberOrString::String(lint_diag.rule_name.to_string())),
                    code_description: Some(CodeDescription {
                        href: Url::parse("https://github.com/ubugeeei/vize/wiki/musea-rules")
                            .unwrap_or_else(|_| {
                                Url::parse("https://github.com/ubugeeei/vize").unwrap()
                            }),
                    }),
                    source: Some(sources::MUSEA.to_string()),
                    message,
                    ..Default::default()
                }
            })
            .collect()
    }

    /// Collect SFC parser diagnostics.
    fn collect_sfc_diagnostics(uri: &Url, content: &str) -> Vec<Diagnostic> {
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string(),
            ..Default::default()
        };

        match vize_atelier_sfc::parse_sfc(content, options) {
            Ok(_) => vec![],
            Err(err) => {
                let range = if let Some(ref loc) = err.loc {
                    Range {
                        start: Position {
                            line: loc.start_line.saturating_sub(1) as u32,
                            character: loc.start_column.saturating_sub(1) as u32,
                        },
                        end: Position {
                            line: loc.end_line.saturating_sub(1) as u32,
                            character: loc.end_column.saturating_sub(1) as u32,
                        },
                    }
                } else {
                    Range::default()
                };

                vec![Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some(sources::SFC_PARSER.to_string()),
                    message: err.message,
                    ..Default::default()
                }]
            }
        }
    }

    /// Collect template parser diagnostics.
    fn collect_template_diagnostics(uri: &Url, content: &str) -> Vec<Diagnostic> {
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string(),
            ..Default::default()
        };

        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
            return vec![];
        };

        let Some(ref template) = descriptor.template else {
            return vec![];
        };

        let allocator = vize_carton::Bump::new();
        let (_, errors) = vize_armature::parse(&allocator, &template.content);

        errors
            .iter()
            .filter_map(|error| {
                let loc = error.loc.as_ref()?;

                // Adjust line numbers based on template block position
                let start_line =
                    (template.loc.start_line as u32) + loc.start.line.saturating_sub(1);
                let end_line = (template.loc.start_line as u32) + loc.end.line.saturating_sub(1);

                Some(Diagnostic {
                    range: Range {
                        start: Position {
                            line: start_line.saturating_sub(1),
                            character: loc.start.column.saturating_sub(1),
                        },
                        end: Position {
                            line: end_line.saturating_sub(1),
                            character: loc.end.column.saturating_sub(1),
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::Number(error.code as i32)),
                    source: Some(sources::TEMPLATE_PARSER.to_string()),
                    message: error.message.clone(),
                    ..Default::default()
                })
            })
            .collect()
    }

    /// Collect linter diagnostics from vize_patina.
    fn collect_lint_diagnostics(uri: &Url, content: &str) -> Vec<Diagnostic> {
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string(),
            ..Default::default()
        };

        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
            return vec![];
        };

        let Some(ref template) = descriptor.template else {
            return vec![];
        };

        // Create linter and lint the template content
        let linter = vize_patina::Linter::new();
        let result = linter.lint_template(&template.content, uri.path());

        // Convert lint diagnostics to LSP diagnostics
        result
            .diagnostics
            .into_iter()
            .map(|lint_diag| {
                // Convert byte offset to line/column within template
                let (start_line, start_col) =
                    offset_to_line_col(&template.content, lint_diag.start as usize);
                let (end_line, end_col) =
                    offset_to_line_col(&template.content, lint_diag.end as usize);

                // Adjust line numbers based on template block position in SFC
                let sfc_start_line = template.loc.start_line as u32 + start_line;
                let sfc_end_line = template.loc.start_line as u32 + end_line;

                // Build the diagnostic message with help text
                let message = if let Some(ref help) = lint_diag.help {
                    format!("{}\n\nHelp: {}", lint_diag.message, help)
                } else {
                    lint_diag.message.to_string()
                };

                Diagnostic {
                    range: Range {
                        start: Position {
                            line: sfc_start_line.saturating_sub(1),
                            character: start_col,
                        },
                        end: Position {
                            line: sfc_end_line.saturating_sub(1),
                            character: end_col,
                        },
                    },
                    severity: Some(match lint_diag.severity {
                        vize_patina::Severity::Error => DiagnosticSeverity::ERROR,
                        vize_patina::Severity::Warning => DiagnosticSeverity::WARNING,
                    }),
                    code: Some(NumberOrString::String(lint_diag.rule_name.to_string())),
                    code_description: Some(CodeDescription {
                        href: Url::parse(&format!(
                            "https://eslint.vuejs.org/rules/{}.html",
                            lint_diag
                                .rule_name
                                .strip_prefix("vue/")
                                .unwrap_or(lint_diag.rule_name)
                        ))
                        .unwrap_or_else(|_| Url::parse("https://eslint.vuejs.org/rules/").unwrap()),
                    }),
                    source: Some(sources::LINTER.to_string()),
                    message,
                    ..Default::default()
                }
            })
            .collect()
    }

    /// Create a diagnostic from a custom error.
    pub fn create_diagnostic(
        range: Range,
        severity: Severity,
        source: &str,
        code: Option<i32>,
        message: String,
    ) -> Diagnostic {
        Diagnostic {
            range,
            severity: Some(severity.into()),
            code: code.map(NumberOrString::Number),
            source: Some(source.to_string()),
            message,
            ..Default::default()
        }
    }
}

/// Builder for creating diagnostics.
pub struct DiagnosticBuilder {
    range: Range,
    severity: Severity,
    source: String,
    code: Option<i32>,
    message: String,
    related_information: Vec<tower_lsp::lsp_types::DiagnosticRelatedInformation>,
}

impl DiagnosticBuilder {
    /// Create a new diagnostic builder.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            range: Range::default(),
            severity: Severity::Error,
            source: "vize".to_string(),
            code: None,
            message: message.into(),
            related_information: Vec::new(),
        }
    }

    /// Set the range.
    pub fn range(mut self, range: Range) -> Self {
        self.range = range;
        self
    }

    /// Set the severity.
    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Set the source.
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Set the error code.
    pub fn code(mut self, code: i32) -> Self {
        self.code = Some(code);
        self
    }

    /// Add related information.
    pub fn related(
        mut self,
        location: tower_lsp::lsp_types::Location,
        message: impl Into<String>,
    ) -> Self {
        self.related_information
            .push(tower_lsp::lsp_types::DiagnosticRelatedInformation {
                location,
                message: message.into(),
            });
        self
    }

    /// Build the diagnostic.
    pub fn build(self) -> Diagnostic {
        Diagnostic {
            range: self.range,
            severity: Some(self.severity.into()),
            code: self.code.map(NumberOrString::Number),
            source: Some(self.source),
            message: self.message,
            related_information: if self.related_information.is_empty() {
                None
            } else {
                Some(self.related_information)
            },
            ..Default::default()
        }
    }
}

/// Convert byte offset to (line, column) - both 0-indexed for LSP.
fn offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut col = 0u32;
    let mut current_offset = 0;

    for ch in source.chars() {
        if current_offset >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        current_offset += ch.len_utf8();
    }

    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_builder() {
        let diagnostic = DiagnosticBuilder::new("Test error")
            .severity(Severity::Warning)
            .source("test")
            .code(42)
            .build();

        assert_eq!(diagnostic.message, "Test error");
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
        assert_eq!(diagnostic.source, Some("test".to_string()));
        assert_eq!(diagnostic.code, Some(NumberOrString::Number(42)));
    }

    #[test]
    fn test_severity_conversion() {
        assert_eq!(
            DiagnosticSeverity::from(Severity::Error),
            DiagnosticSeverity::ERROR
        );
        assert_eq!(
            DiagnosticSeverity::from(Severity::Warning),
            DiagnosticSeverity::WARNING
        );
        assert_eq!(
            DiagnosticSeverity::from(Severity::Information),
            DiagnosticSeverity::INFORMATION
        );
        assert_eq!(
            DiagnosticSeverity::from(Severity::Hint),
            DiagnosticSeverity::HINT
        );
    }
}
