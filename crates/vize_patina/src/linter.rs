//! Main linter entry point.
//!
//! High-performance Vue template linter with arena allocation.

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, LintSummary};
use crate::rule::RuleRegistry;
use crate::visitor::LintVisitor;
use vize_armature::Parser;
use vize_carton::Allocator;

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
        }
    }

    /// Create a linter with a custom rule registry
    #[inline]
    pub fn with_registry(registry: RuleRegistry) -> Self {
        Self {
            registry,
            initial_capacity: Self::DEFAULT_INITIAL_CAPACITY,
        }
    }

    /// Set the initial allocator capacity
    #[inline]
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.initial_capacity = capacity;
        self
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

        // Create lint context
        let mut ctx = LintContext::new(allocator, source, filename);

        // Run visitor with all rules
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
        // Extract template content from SFC
        let template_content = extract_template_content(source);

        if let Some(content) = template_content {
            self.lint_template(&content, filename)
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
#[inline]
fn extract_template_content(source: &str) -> Option<String> {
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

    Some(source[content_start..content_end].to_string())
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
}
