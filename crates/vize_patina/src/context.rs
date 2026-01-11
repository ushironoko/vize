//! Lint context for rule execution.
//!
//! Uses arena allocation for high-performance memory management.

use crate::diagnostic::{LintDiagnostic, Severity};
use rustc_hash::FxHashSet;
use vize_carton::{Allocator, CompactString};
use vize_relief::ast::SourceLocation;

/// Context for tracking element state during traversal
///
/// Uses `CompactString` for tag to avoid lifetime complications while
/// maintaining efficiency for small strings (inline storage up to 24 bytes).
#[derive(Debug, Clone)]
pub struct ElementContext {
    /// Tag name (CompactString for efficiency)
    pub tag: CompactString,
    /// Whether element has v-for directive
    pub has_v_for: bool,
    /// Whether element has v-if directive
    pub has_v_if: bool,
    /// Variables defined by v-for on this element
    pub v_for_vars: Vec<CompactString>,
}

impl ElementContext {
    /// Create a new element context
    #[inline]
    pub fn new(tag: impl Into<CompactString>) -> Self {
        Self {
            tag: tag.into(),
            has_v_for: false,
            has_v_if: false,
            v_for_vars: Vec::new(),
        }
    }

    /// Create with v-for info
    #[inline]
    pub fn with_v_for(tag: impl Into<CompactString>, vars: Vec<CompactString>) -> Self {
        Self {
            tag: tag.into(),
            has_v_for: true,
            has_v_if: false,
            v_for_vars: vars,
        }
    }
}

/// Lint context provides utilities for rules during execution.
///
/// Uses arena allocation for efficient memory management during lint traversal.
pub struct LintContext<'a> {
    /// Arena allocator for this lint session
    allocator: &'a Allocator,
    /// Source code being linted
    pub source: &'a str,
    /// Filename for diagnostics
    pub filename: &'a str,
    /// Collected diagnostics (pre-allocated capacity)
    diagnostics: Vec<LintDiagnostic>,
    /// Current rule name (set by visitor before calling rule methods)
    pub current_rule: &'static str,
    /// Parent element stack for context (pre-allocated capacity)
    element_stack: Vec<ElementContext>,
    /// Variables in current scope (from v-for)
    scope_variables: FxHashSet<CompactString>,
    /// Cached error count for fast access
    error_count: usize,
    /// Cached warning count for fast access
    warning_count: usize,
}

impl<'a> LintContext<'a> {
    /// Initial capacity for diagnostics vector
    const INITIAL_DIAGNOSTICS_CAPACITY: usize = 16;
    /// Initial capacity for element stack
    const INITIAL_STACK_CAPACITY: usize = 32;

    /// Create a new lint context with arena allocator
    #[inline]
    pub fn new(allocator: &'a Allocator, source: &'a str, filename: &'a str) -> Self {
        Self {
            allocator,
            source,
            filename,
            diagnostics: Vec::with_capacity(Self::INITIAL_DIAGNOSTICS_CAPACITY),
            current_rule: "",
            element_stack: Vec::with_capacity(Self::INITIAL_STACK_CAPACITY),
            scope_variables: FxHashSet::default(),
            error_count: 0,
            warning_count: 0,
        }
    }

    /// Get the allocator
    #[inline]
    pub fn allocator(&self) -> &'a Allocator {
        self.allocator
    }

    /// Allocate a string in the arena
    #[inline]
    pub fn alloc_str(&self, s: &str) -> &'a str {
        self.allocator.alloc_str(s)
    }

    /// Report a lint diagnostic
    #[inline]
    pub fn report(&mut self, diagnostic: LintDiagnostic) {
        match diagnostic.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
        }
        self.diagnostics.push(diagnostic);
    }

    /// Report an error at a location
    #[inline]
    pub fn error(&mut self, message: impl Into<CompactString>, loc: &SourceLocation) {
        self.report(LintDiagnostic::error(
            self.current_rule,
            message,
            loc.start.offset,
            loc.end.offset,
        ));
    }

    /// Report a warning at a location
    #[inline]
    pub fn warn(&mut self, message: impl Into<CompactString>, loc: &SourceLocation) {
        self.report(LintDiagnostic::warn(
            self.current_rule,
            message,
            loc.start.offset,
            loc.end.offset,
        ));
    }

    /// Report an error with help message
    #[inline]
    pub fn error_with_help(
        &mut self,
        message: impl Into<CompactString>,
        loc: &SourceLocation,
        help: impl Into<CompactString>,
    ) {
        self.report(
            LintDiagnostic::error(self.current_rule, message, loc.start.offset, loc.end.offset)
                .with_help(help),
        );
    }

    /// Report a warning with help message
    #[inline]
    pub fn warn_with_help(
        &mut self,
        message: impl Into<CompactString>,
        loc: &SourceLocation,
        help: impl Into<CompactString>,
    ) {
        self.report(
            LintDiagnostic::warn(self.current_rule, message, loc.start.offset, loc.end.offset)
                .with_help(help),
        );
    }

    /// Report a diagnostic with related label
    #[inline]
    pub fn error_with_label(
        &mut self,
        message: impl Into<CompactString>,
        loc: &SourceLocation,
        label_message: impl Into<CompactString>,
        label_loc: &SourceLocation,
    ) {
        self.report(
            LintDiagnostic::error(self.current_rule, message, loc.start.offset, loc.end.offset)
                .with_label(label_message, label_loc.start.offset, label_loc.end.offset),
        );
    }

    /// Get collected diagnostics
    #[inline]
    pub fn into_diagnostics(self) -> Vec<LintDiagnostic> {
        self.diagnostics
    }

    /// Get reference to collected diagnostics
    #[inline]
    pub fn diagnostics(&self) -> &[LintDiagnostic] {
        &self.diagnostics
    }

    /// Push an element onto the context stack
    #[inline]
    pub fn push_element(&mut self, ctx: ElementContext) {
        // Add v-for vars to scope
        for var in &ctx.v_for_vars {
            self.scope_variables.insert(var.clone());
        }
        self.element_stack.push(ctx);
    }

    /// Pop an element from the context stack
    #[inline]
    pub fn pop_element(&mut self) -> Option<ElementContext> {
        if let Some(ctx) = self.element_stack.pop() {
            // Remove v-for vars from scope
            for var in &ctx.v_for_vars {
                self.scope_variables.remove(var);
            }
            Some(ctx)
        } else {
            None
        }
    }

    /// Check if inside a v-for loop
    #[inline]
    pub fn is_in_v_for(&self) -> bool {
        self.element_stack.iter().any(|e| e.has_v_for)
    }

    /// Get all v-for variables in current scope
    #[inline]
    pub fn v_for_vars(&self) -> impl Iterator<Item = &str> {
        self.element_stack
            .iter()
            .flat_map(|e| e.v_for_vars.iter().map(|s| s.as_str()))
    }

    /// Check if a variable is defined by a parent v-for
    #[inline]
    pub fn is_v_for_var(&self, name: &str) -> bool {
        self.scope_variables.contains(name)
    }

    /// Check if a variable is defined by a PARENT v-for (excluding current element)
    ///
    /// This is useful for shadow detection where we want to check if a variable
    /// in the current v-for shadows a variable from an outer scope.
    #[inline]
    pub fn is_parent_v_for_var(&self, name: &str) -> bool {
        // Check all elements except the last one (current element)
        if self.element_stack.len() < 2 {
            return false;
        }
        for elem in self.element_stack.iter().take(self.element_stack.len() - 1) {
            for var in &elem.v_for_vars {
                if var.as_str() == name {
                    return true;
                }
            }
        }
        false
    }

    /// Get current element context (top of stack)
    #[inline]
    pub fn current_element(&self) -> Option<&ElementContext> {
        self.element_stack.last()
    }

    /// Get parent element context
    #[inline]
    pub fn parent_element(&self) -> Option<&ElementContext> {
        if self.element_stack.len() >= 2 {
            self.element_stack.get(self.element_stack.len() - 2)
        } else {
            None
        }
    }

    /// Get the error count (cached, O(1))
    #[inline]
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    /// Get the warning count (cached, O(1))
    #[inline]
    pub fn warning_count(&self) -> usize {
        self.warning_count
    }
}
