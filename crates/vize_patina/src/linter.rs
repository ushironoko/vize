//! Main linter entry point.
//!
//! High-performance Vue template linter with arena allocation.

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, LintSummary};
use crate::rule::RuleRegistry;
use crate::visitor::LintVisitor;
use vize_armature::Parser;
use vize_carton::i18n::Locale;
use vize_carton::{Allocator, FxHashSet};

/// Lint result for a single file
#[derive(Debug, Clone)]
pub struct LintResult {
    /// Filename that was linted
    pub filename: String,
    /// Collected diagnostics
    pub diagnostics: Vec<LintDiagnostic>,
    /// Number of errors
    pub error_count: usize,
    /// Number of warnings
    pub warning_count: usize,
}

impl LintResult {
    /// Check if there are any errors
    #[inline]
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Check if there are any diagnostics
    #[inline]
    pub fn has_diagnostics(&self) -> bool {
        !self.diagnostics.is_empty()
    }
}

/// Main linter struct.
///
/// The linter is designed for high performance:
/// - Uses arena allocation for AST and context
/// - Pre-allocates vectors with expected capacity
/// - Minimizes allocations during traversal
pub struct Linter {
    registry: RuleRegistry,
    /// Estimated initial allocator capacity (in bytes)
    initial_capacity: usize,
    /// Locale for i18n messages
    locale: Locale,
    /// Optional set of enabled rule names (if None, all rules are enabled)
    enabled_rules: Option<FxHashSet<String>>,
}

impl Linter {
    /// Default initial capacity for the arena (64KB)
    const DEFAULT_INITIAL_CAPACITY: usize = 64 * 1024;

    /// Create a new linter with recommended rules
    #[inline]
    pub fn new() -> Self {
        Self {
            registry: RuleRegistry::with_recommended(),
            initial_capacity: Self::DEFAULT_INITIAL_CAPACITY,
            locale: Locale::default(),
            enabled_rules: None,
        }
    }

    /// Create a linter with a custom rule registry
    #[inline]
    pub fn with_registry(registry: RuleRegistry) -> Self {
        Self {
            registry,
            initial_capacity: Self::DEFAULT_INITIAL_CAPACITY,
            locale: Locale::default(),
            enabled_rules: None,
        }
    }

    /// Set the initial allocator capacity
    #[inline]
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.initial_capacity = capacity;
        self
    }

    /// Set the locale for i18n messages
    #[inline]
    pub fn with_locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// Set enabled rules (if None, all rules are enabled)
    ///
    /// Pass a list of rule names to enable only those rules.
    /// Rules not in the list will be skipped during linting.
    #[inline]
    pub fn with_enabled_rules(mut self, rules: Option<Vec<String>>) -> Self {
        self.enabled_rules = rules.map(|r| r.into_iter().collect());
        self
    }

    /// Get the current locale
    #[inline]
    pub fn locale(&self) -> Locale {
        self.locale
    }

    /// Check if a rule is enabled
    #[inline]
    pub fn is_rule_enabled(&self, rule_name: &str) -> bool {
        match &self.enabled_rules {
            Some(set) => set.contains(rule_name),
            None => true,
        }
    }

    /// Lint a Vue template source
    #[inline]
    pub fn lint_template(&self, source: &str, filename: &str) -> LintResult {
        // Create allocator sized for source (rough heuristic: 4x source size)
        let capacity = (source.len() * 4).max(self.initial_capacity);
        let allocator = Allocator::with_capacity(capacity);

        self.lint_template_with_allocator(&allocator, source, filename)
    }

    /// Lint a Vue template with a provided allocator (for reuse)
    pub fn lint_template_with_allocator(
        &self,
        allocator: &Allocator,
        source: &str,
        filename: &str,
    ) -> LintResult {
        // Parse the template
        let parser = Parser::new(allocator.as_bump(), source);
        let (root, _parse_errors) = parser.parse();

        // Create lint context with locale and enabled rules filter
        let mut ctx = LintContext::with_locale(allocator, source, filename, self.locale);
        ctx.set_enabled_rules(self.enabled_rules.clone());

        // Run visitor with all rules (filtering happens in context)
        let mut visitor = LintVisitor::new(&mut ctx, self.registry.rules());
        visitor.visit_root(&root);

        // Collect results (error/warning counts are cached)
        let error_count = ctx.error_count();
        let warning_count = ctx.warning_count();
        let diagnostics = ctx.into_diagnostics();

        LintResult {
            filename: filename.to_string(),
            diagnostics,
            error_count,
            warning_count,
        }
    }

    /// Lint multiple files and aggregate results
    pub fn lint_files(&self, files: &[(String, String)]) -> (Vec<LintResult>, LintSummary) {
        let mut results = Vec::with_capacity(files.len());
        let mut summary = LintSummary::default();

        // Reuse allocator across files for better memory efficiency
        let mut allocator = Allocator::with_capacity(self.initial_capacity);

        for (filename, source) in files {
            let result = self.lint_template_with_allocator(&allocator, source, filename);
            summary.error_count += result.error_count;
            summary.warning_count += result.warning_count;
            results.push(result);

            // Reset allocator for next file
            allocator.reset();
        }

        summary.file_count = files.len();
        (results, summary)
    }

    /// Get the rule registry
    #[inline]
    pub fn registry(&self) -> &RuleRegistry {
        &self.registry
    }

    /// Get all registered rules
    #[inline]
    pub fn rules(&self) -> &[Box<dyn crate::rule::Rule>] {
        self.registry.rules()
    }

    /// Lint a full Vue SFC file
    ///
    /// This extracts the template from the SFC and lints it.
    #[inline]
    pub fn lint_sfc(&self, source: &str, filename: &str) -> LintResult {
        // Extract template content from SFC with byte offset
        let template_info = extract_template_content(source);

        if let Some((content, byte_offset)) = template_info {
            let mut result = self.lint_template(&content, filename);

            // Adjust byte offsets in diagnostics to match original file positions
            if byte_offset > 0 {
                for diag in &mut result.diagnostics {
                    diag.start += byte_offset;
                    diag.end += byte_offset;
                    // Also adjust label positions
                    for label in &mut diag.labels {
                        label.start += byte_offset;
                        label.end += byte_offset;
                    }
                }
            }

            result
        } else {
            // No template found, return empty result
            LintResult {
                filename: filename.to_string(),
                diagnostics: Vec::new(),
                error_count: 0,
                warning_count: 0,
            }
        }
    }
}

/// Extract template content from SFC source (optimized)
/// Returns the content and the byte offset where the template content starts
#[inline]
fn extract_template_content(source: &str) -> Option<(String, u32)> {
    // Fast path: check if template tag exists
    let start_tag = "<template";
    let start_idx = source.find(start_tag)?;

    // Find the end of the opening tag (handle attributes)
    let tag_end = source[start_idx..].find('>')?;
    let content_start = start_idx + tag_end + 1;

    // Find </template> closing tag (search from end for speed)
    let end_tag = "</template>";
    let content_end = source.rfind(end_tag)?;

    if content_start >= content_end {
        return None;
    }

    Some((
        source[content_start..content_end].to_string(),
        content_start as u32,
    ))
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_empty_template() {
        let linter = Linter::new();
        let result = linter.lint_template("", "test.vue");
        assert!(!result.has_errors());
        assert!(!result.has_diagnostics());
    }

    #[test]
    fn test_lint_simple_template() {
        let linter = Linter::new();
        let result = linter.lint_template("<div>Hello</div>", "test.vue");
        assert!(!result.has_errors());
    }

    #[test]
    fn test_lint_with_allocator_reuse() {
        let linter = Linter::new();
        let allocator = Allocator::with_capacity(1024);

        let result1 =
            linter.lint_template_with_allocator(&allocator, "<div>Hello</div>", "test1.vue");
        assert!(!result1.has_errors());

        // Allocator is borrowed, can't reset here, but demonstrates the API
    }

    #[test]
    fn test_lint_files_batch() {
        let linter = Linter::new();
        let files = vec![
            ("test1.vue".to_string(), "<div>Hello</div>".to_string()),
            ("test2.vue".to_string(), "<span>World</span>".to_string()),
        ];

        let (results, summary) = linter.lint_files(&files);
        assert_eq!(results.len(), 2);
        assert_eq!(summary.file_count, 2);
    }

    #[test]
    fn test_disable_next_line() {
        let linter = Linter::new();
        // Without disable comment - should have error
        let result = linter.lint_template(
            r#"<ul><li v-for="item in items">{{ item }}</li></ul>"#,
            "test.vue",
        );
        assert!(result.error_count > 0, "Should have error without key");

        // With disable comment - should suppress error
        let result = linter.lint_template(
            r#"<ul><!-- vize-disable-next-line -->
<li v-for="item in items">{{ item }}</li></ul>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0, "Error should be suppressed");
    }

    #[test]
    fn test_disable_specific_rule() {
        let linter = Linter::new();
        // With specific rule disable
        let result = linter.lint_template(
            r#"<ul><!-- vize-disable-next-line vue/require-v-for-key -->
<li v-for="item in items">{{ item }}</li></ul>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0, "Specific rule should be suppressed");
    }

    #[test]
    fn test_disable_all() {
        let linter = Linter::new();
        // With disable all
        let result = linter.lint_template(
            r#"<!-- vize-disable -->
<ul><li v-for="item in items">{{ item }}</li></ul>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0, "All rules should be disabled");
    }

    #[test]
    fn test_lint_sfc_extracts_template() {
        let linter = Linter::new();
        // SFC with script and template - should only lint template content
        let sfc = r#"<script setup lang="ts">
interface Props {
  schema?: BaseSchema<FormShape, FormShape, any>;
}
</script>

<template>
  <div>Hello World</div>
</template>
"#;
        let result = linter.lint_sfc(sfc, "test.vue");
        // Should not report errors for TypeScript code in <script>
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_lint_sfc_no_template() {
        let linter = Linter::new();
        // SFC without template - should return empty result
        let sfc = r#"<script setup lang="ts">
const foo = 'bar';
</script>
"#;
        let result = linter.lint_sfc(sfc, "test.vue");
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_lint_sfc_byte_offset() {
        let linter = Linter::new();
        // SFC where template has an error - byte offset should be adjusted
        let sfc = r#"<script setup lang="ts">
const foo = 'bar';
</script>

<template>
  <ul><li v-for="item in items">{{ item }}</li></ul>
</template>
"#;
        let result = linter.lint_sfc(sfc, "test.vue");
        // Should have error for missing :key
        assert!(result.error_count > 0, "Should detect v-for without key");

        // The byte offset should point to the correct location in the original file
        if let Some(diag) = result.diagnostics.first() {
            // The diagnostic should point somewhere in the template section
            // Template starts after "<script>...</script>\n\n<template>\n"
            assert!(
                diag.start > 50,
                "Byte offset should be adjusted for template position"
            );
        }
    }
}
