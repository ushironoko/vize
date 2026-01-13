//! High-performance Vue SFC analyzer.
//!
//! This module provides the `Analyzer` that produces `AnalysisSummary`.
//!
//! ## Performance Considerations
//!
//! - **Lazy analysis**: Only analyze what's requested
//! - **Zero-copy**: Use borrowed strings where possible
//! - **Arena allocation**: Temporary data uses arena allocator
//! - **Efficient structures**: FxHashMap, SmallVec, phf
//! - **Incremental**: Can analyze script and template separately
//!
//! ## Usage
//!
//! ```ignore
//! let mut analyzer = Analyzer::new();
//!
//! // Analyze script (fast path if only script bindings needed)
//! analyzer.analyze_script(script_source);
//!
//! // Analyze template (requires parsed AST)
//! analyzer.analyze_template(&template_ast);
//!
//! // Get results
//! let summary = analyzer.finish();
//! ```

use crate::analysis::{AnalysisSummary, UndefinedRef};
use crate::{ScopeBinding, ScopeKind};
use vize_carton::CompactString;
use vize_relief::ast::{
    ElementNode, ExpressionNode, ForNode, IfNode, PropNode, RootNode, TemplateChildNode,
};
use vize_relief::BindingType;

/// Analysis options for controlling what gets analyzed.
///
/// Use this to skip unnecessary analysis passes for better performance.
#[derive(Debug, Clone, Copy, Default)]
pub struct AnalyzerOptions {
    /// Analyze script bindings (defineProps, defineEmits, etc.)
    pub analyze_script: bool,
    /// Analyze template scopes (v-for, v-slot variables)
    pub analyze_template_scopes: bool,
    /// Track component and directive usage
    pub track_usage: bool,
    /// Detect undefined references (requires script + template)
    pub detect_undefined: bool,
    /// Analyze hoisting opportunities
    pub analyze_hoisting: bool,
}

impl AnalyzerOptions {
    /// Full analysis (all features enabled)
    #[inline]
    pub const fn full() -> Self {
        Self {
            analyze_script: true,
            analyze_template_scopes: true,
            track_usage: true,
            detect_undefined: true,
            analyze_hoisting: true,
        }
    }

    /// Minimal analysis for linting (fast)
    #[inline]
    pub const fn for_lint() -> Self {
        Self {
            analyze_script: true,
            analyze_template_scopes: true,
            track_usage: true,
            detect_undefined: true,
            analyze_hoisting: false,
        }
    }

    /// Analysis for compilation (needs hoisting)
    #[inline]
    pub const fn for_compile() -> Self {
        Self {
            analyze_script: true,
            analyze_template_scopes: true,
            track_usage: true,
            detect_undefined: false,
            analyze_hoisting: true,
        }
    }
}

/// High-performance Vue SFC analyzer.
///
/// Uses lazy evaluation and efficient data structures to minimize overhead.
pub struct Analyzer {
    options: AnalyzerOptions,
    summary: AnalysisSummary,
    /// Track if script was analyzed (for undefined detection)
    script_analyzed: bool,
}

impl Analyzer {
    /// Create a new analyzer with default options
    #[inline]
    pub fn new() -> Self {
        Self::with_options(AnalyzerOptions::default())
    }

    /// Create analyzer with specific options
    #[inline]
    pub fn with_options(options: AnalyzerOptions) -> Self {
        Self {
            options,
            summary: AnalysisSummary::new(),
            script_analyzed: false,
        }
    }

    /// Create analyzer for linting (optimized)
    #[inline]
    pub fn for_lint() -> Self {
        Self::with_options(AnalyzerOptions::for_lint())
    }

    /// Create analyzer for compilation
    #[inline]
    pub fn for_compile() -> Self {
        Self::with_options(AnalyzerOptions::for_compile())
    }

    /// Analyze script setup source code.
    ///
    /// This uses OXC parser to extract:
    /// - defineProps/defineEmits/defineModel calls
    /// - Top-level bindings (const, let, function, class)
    /// - Import statements
    /// - Reactivity wrappers (ref, reactive, computed)
    ///
    /// Performance: OXC provides high-performance AST parsing with accurate span tracking.
    pub fn analyze_script(&mut self, source: &str) -> &mut Self {
        self.analyze_script_setup(source)
    }

    /// Analyze script setup source code.
    pub fn analyze_script_setup(&mut self, source: &str) -> &mut Self {
        if !self.options.analyze_script {
            return self;
        }

        self.script_analyzed = true;

        // Use OXC-based parser for accurate AST analysis
        let result = crate::script_parser::parse_script_setup(source);

        // Merge results into summary
        self.summary.bindings = result.bindings;
        self.summary.macros = result.macros;
        self.summary.reactivity = result.reactivity;
        self.summary.type_exports = result.type_exports;
        self.summary.invalid_exports = result.invalid_exports;
        self.summary.scopes = result.scopes;

        self
    }

    /// Analyze non-script-setup (Options API) source code.
    pub fn analyze_script_plain(&mut self, source: &str) -> &mut Self {
        if !self.options.analyze_script {
            return self;
        }

        self.script_analyzed = true;

        // Use OXC-based parser for non-script-setup
        let result = crate::script_parser::parse_script(source);

        // Merge results into summary
        self.summary.bindings = result.bindings;
        self.summary.macros = result.macros;
        self.summary.reactivity = result.reactivity;
        self.summary.type_exports = result.type_exports;
        self.summary.invalid_exports = result.invalid_exports;
        self.summary.scopes = result.scopes;

        self
    }

    /// Analyze template AST.
    ///
    /// This extracts:
    /// - v-for/v-slot scope variables
    /// - Component usage
    /// - Directive usage
    /// - Undefined references (if script was analyzed)
    ///
    /// Performance: O(n) single traversal
    pub fn analyze_template(&mut self, root: &RootNode<'_>) -> &mut Self {
        if !self.options.analyze_template_scopes && !self.options.track_usage {
            return self;
        }

        // Single-pass template traversal
        for child in root.children.iter() {
            self.visit_template_child(child, &mut Vec::new());
        }

        self
    }

    /// Finish analysis and return the summary.
    ///
    /// Consumes the analyzer.
    #[inline]
    pub fn finish(self) -> AnalysisSummary {
        self.summary
    }

    /// Get a reference to the current summary (without consuming).
    #[inline]
    pub fn summary(&self) -> &AnalysisSummary {
        &self.summary
    }

    // =========================================================================
    // Template Analysis (Single-pass traversal)
    // =========================================================================

    /// Visit template child node
    fn visit_template_child(
        &mut self,
        node: &TemplateChildNode<'_>,
        scope_vars: &mut Vec<CompactString>,
    ) {
        match node {
            TemplateChildNode::Element(el) => self.visit_element(el, scope_vars),
            TemplateChildNode::If(if_node) => self.visit_if(if_node, scope_vars),
            TemplateChildNode::For(for_node) => self.visit_for(for_node, scope_vars),
            TemplateChildNode::Interpolation(interp) => {
                if self.options.detect_undefined && self.script_analyzed {
                    self.check_expression_refs(
                        &interp.content,
                        scope_vars,
                        interp.loc.start.offset,
                    );
                }
            }
            _ => {}
        }
    }

    /// Visit element node
    fn visit_element(&mut self, el: &ElementNode<'_>, scope_vars: &mut Vec<CompactString>) {
        // Track component usage
        if self.options.track_usage {
            let tag = el.tag.as_str();
            if is_component_tag(tag) {
                self.summary.used_components.insert(CompactString::new(tag));
            }
        }

        // Check directives
        for prop in &el.props {
            if let PropNode::Directive(dir) = prop {
                // Track directive usage
                if self.options.track_usage {
                    let name = dir.name.as_str();
                    if !is_builtin_directive(name) {
                        self.summary
                            .used_directives
                            .insert(CompactString::new(name));
                    }
                }

                // Check expressions for undefined refs
                if self.options.detect_undefined && self.script_analyzed {
                    if let Some(ref exp) = dir.exp {
                        // Skip v-for (analyzed separately)
                        if dir.name != "for" {
                            self.check_expression_refs(exp, scope_vars, dir.loc.start.offset);
                        }
                    }
                }
            }
        }

        // Visit children
        for child in el.children.iter() {
            self.visit_template_child(child, scope_vars);
        }
    }

    /// Visit if node
    fn visit_if(&mut self, if_node: &IfNode<'_>, scope_vars: &mut Vec<CompactString>) {
        for branch in if_node.branches.iter() {
            // Check condition
            if self.options.detect_undefined && self.script_analyzed {
                if let Some(ref cond) = branch.condition {
                    self.check_expression_refs(cond, scope_vars, branch.loc.start.offset);
                }
            }

            // Visit children
            for child in branch.children.iter() {
                self.visit_template_child(child, scope_vars);
            }
        }
    }

    /// Visit for node
    fn visit_for(&mut self, for_node: &ForNode<'_>, scope_vars: &mut Vec<CompactString>) {
        // Add v-for variables to scope
        let vars_added = self.extract_for_vars(for_node);
        let vars_count = vars_added.len();

        if self.options.analyze_template_scopes && !vars_added.is_empty() {
            self.summary.scopes.enter_scope(ScopeKind::VFor);
            for var in &vars_added {
                self.summary
                    .scopes
                    .add_binding(var.clone(), ScopeBinding::new(BindingType::SetupConst, 0));
            }
        }

        for var in vars_added {
            scope_vars.push(var);
        }

        // Check source expression
        if self.options.detect_undefined && self.script_analyzed {
            self.check_expression_refs(&for_node.source, scope_vars, for_node.loc.start.offset);
        }

        // Visit children
        for child in for_node.children.iter() {
            self.visit_template_child(child, scope_vars);
        }

        // Remove v-for variables from scope
        for _ in 0..vars_count {
            scope_vars.pop();
        }
        if self.options.analyze_template_scopes && vars_count > 0 {
            self.summary.scopes.exit_scope();
        }
    }

    /// Extract variables from v-for expression
    fn extract_for_vars(&self, for_node: &ForNode<'_>) -> Vec<CompactString> {
        let mut vars = Vec::new();

        // Value alias (e.g., item in "item in items")
        if let Some(ExpressionNode::Simple(exp)) = &for_node.value_alias {
            vars.push(exp.content.clone());
        }

        // Key alias (e.g., key in "(item, key) in items")
        if let Some(ExpressionNode::Simple(exp)) = &for_node.key_alias {
            vars.push(exp.content.clone());
        }

        // Index alias (e.g., index in "(item, key, index) in items")
        if let Some(ExpressionNode::Simple(exp)) = &for_node.object_index_alias {
            vars.push(exp.content.clone());
        }

        vars
    }

    /// Check expression for undefined references
    fn check_expression_refs(
        &mut self,
        expr: &ExpressionNode<'_>,
        scope_vars: &[CompactString],
        offset: u32,
    ) {
        let content = match expr {
            ExpressionNode::Simple(s) => s.content.as_str(),
            ExpressionNode::Compound(c) => c.loc.source.as_str(),
        };

        // Fast identifier extraction
        for ident in extract_identifiers_fast(content) {
            // Check if defined
            let is_defined = scope_vars.iter().any(|v| v.as_str() == ident)
                || self.summary.bindings.contains(ident)
                || crate::builtins::is_js_global(ident)
                || is_keyword(ident);

            if !is_defined {
                self.summary.undefined_refs.push(UndefinedRef {
                    name: CompactString::new(ident),
                    offset,
                    context: CompactString::new("template expression"),
                });
            }
        }
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if a tag is a component (PascalCase or contains hyphen)
#[inline]
fn is_component_tag(tag: &str) -> bool {
    tag.contains('-') || tag.chars().next().is_some_and(|c| c.is_ascii_uppercase())
}

/// Check if a directive is built-in
#[inline]
fn is_builtin_directive(name: &str) -> bool {
    matches!(
        name,
        "if" | "else"
            | "else-if"
            | "for"
            | "show"
            | "bind"
            | "on"
            | "model"
            | "slot"
            | "text"
            | "html"
            | "cloak"
            | "once"
            | "pre"
            | "memo"
    )
}

/// Check if a string is a JS keyword
#[inline]
fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "true"
            | "false"
            | "null"
            | "undefined"
            | "this"
            | "arguments"
            | "if"
            | "else"
            | "for"
            | "while"
            | "do"
            | "switch"
            | "case"
            | "break"
            | "continue"
            | "return"
            | "throw"
            | "try"
            | "catch"
            | "finally"
            | "new"
            | "delete"
            | "typeof"
            | "void"
            | "in"
            | "of"
            | "instanceof"
            | "function"
            | "class"
            | "const"
            | "let"
            | "var"
            | "async"
            | "await"
            | "yield"
            | "import"
            | "export"
            | "default"
            | "from"
            | "as"
    )
}

/// Fast identifier extraction from expression string
#[inline]
fn extract_identifiers_fast(expr: &str) -> Vec<&str> {
    let mut identifiers = Vec::with_capacity(4);
    let bytes = expr.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let c = bytes[i];

        // Start of identifier
        if c.is_ascii_alphabetic() || c == b'_' || c == b'$' {
            let start = i;
            i += 1;

            while i < len
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
            {
                i += 1;
            }

            identifiers.push(&expr[start..i]);
        } else {
            i += 1;
        }
    }

    identifiers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::{InvalidExportKind, TypeExportKind};

    #[test]
    fn test_extract_identifiers_fast() {
        let ids = extract_identifiers_fast("count + 1");
        assert_eq!(ids, vec!["count"]);

        let ids = extract_identifiers_fast("user.name + item.value");
        assert_eq!(ids, vec!["user", "name", "item", "value"]);

        let ids = extract_identifiers_fast("");
        assert!(ids.is_empty());
    }

    #[test]
    fn test_analyzer_script_bindings() {
        let mut analyzer = Analyzer::for_lint();
        analyzer.analyze_script(
            r#"
            const count = ref(0)
            const name = 'hello'
            let flag = true
            function handleClick() {}
        "#,
        );

        let summary = analyzer.finish();
        assert!(summary.bindings.contains("count"));
        assert!(summary.bindings.contains("name"));
        assert!(summary.bindings.contains("flag"));
        assert!(summary.bindings.contains("handleClick"));

        // Check reactivity tracking
        assert!(summary.reactivity.is_reactive("count"));
        assert!(summary.reactivity.needs_value_access("count"));
    }

    #[test]
    fn test_analyzer_define_props() {
        let mut analyzer = Analyzer::for_lint();
        analyzer.analyze_script(
            r#"
            const props = defineProps<{
                msg: string
                count?: number
            }>()
        "#,
        );

        let summary = analyzer.finish();
        assert_eq!(summary.macros.props().len(), 2);

        let prop_names: Vec<_> = summary
            .macros
            .props()
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert!(prop_names.contains(&"msg"));
        assert!(prop_names.contains(&"count"));
    }

    #[test]
    fn test_type_exports() {
        let mut analyzer = Analyzer::for_lint();
        analyzer.analyze_script(
            r#"
export type Props = {
    msg: string
}
export interface Emits {
    (e: 'update', value: string): void
}
const count = ref(0)
        "#,
        );

        let summary = analyzer.finish();
        assert_eq!(summary.type_exports.len(), 2);

        let type_export = &summary.type_exports[0];
        assert_eq!(type_export.name.as_str(), "Props");
        assert_eq!(type_export.kind, TypeExportKind::Type);
        assert!(type_export.hoisted);

        let interface_export = &summary.type_exports[1];
        assert_eq!(interface_export.name.as_str(), "Emits");
        assert_eq!(interface_export.kind, TypeExportKind::Interface);
        assert!(interface_export.hoisted);
    }

    #[test]
    fn test_invalid_exports() {
        let mut analyzer = Analyzer::for_lint();
        analyzer.analyze_script(
            r#"
export const foo = 'bar'
export let count = 0
export function hello() {}
export class MyClass {}
export default { foo: 'bar' }
const valid = ref(0)
        "#,
        );

        let summary = analyzer.finish();
        assert_eq!(summary.invalid_exports.len(), 5);

        let kinds: Vec<_> = summary.invalid_exports.iter().map(|e| e.kind).collect();
        assert!(kinds.contains(&InvalidExportKind::Const));
        assert!(kinds.contains(&InvalidExportKind::Let));
        assert!(kinds.contains(&InvalidExportKind::Function));
        assert!(kinds.contains(&InvalidExportKind::Class));
        assert!(kinds.contains(&InvalidExportKind::Default));

        let names: Vec<_> = summary
            .invalid_exports
            .iter()
            .map(|e| e.name.as_str())
            .collect();
        assert!(names.contains(&"foo"));
        assert!(names.contains(&"count"));
        assert!(names.contains(&"hello"));
        assert!(names.contains(&"MyClass"));
    }

    #[test]
    fn test_mixed_exports() {
        let mut analyzer = Analyzer::for_lint();
        analyzer.analyze_script(
            r#"
export type MyType = string
export const invalid = 123
export interface MyInterface { name: string }
        "#,
        );

        let summary = analyzer.finish();
        // Valid type exports
        assert_eq!(summary.type_exports.len(), 2);
        // Invalid value exports
        assert_eq!(summary.invalid_exports.len(), 1);
        assert_eq!(summary.invalid_exports[0].name.as_str(), "invalid");
    }
}
