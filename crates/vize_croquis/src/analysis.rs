//! Analysis summary for Vue SFC semantic analysis.
//!
//! This module provides the `AnalysisSummary` struct that aggregates all
//! semantic analysis results from a Vue SFC. It serves as the bridge between
//! the parser and downstream consumers (linter, transformer, codegen).
//!
//! ## Architecture
//!
//! ```text
//! vize_armature (Parse)
//!        ↓
//!   vize_relief (AST)
//!        ↓
//!  vize_croquis (Semantic Analysis)
//!        ↓
//!   AnalysisSummary ←── This module
//!        ↓
//!  ┌─────┴─────┐
//!  ↓           ↓
//! patina    atelier
//! (lint)    (transform)
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use vize_croquis::{Analyzer, AnalysisSummary};
//!
//! let summary = Analyzer::new()
//!     .analyze_script(script_content)
//!     .analyze_template(template_ast)
//!     .finish();
//!
//! // Use in linter
//! let lint_ctx = LintContext::with_analysis(&summary);
//!
//! // Use in transformer
//! let transform_ctx = TransformContext::with_analysis(&summary);
//! ```

use crate::hoist::HoistTracker;
use crate::macros::MacroTracker;
use crate::reactivity::ReactivityTracker;
use crate::types::TypeResolver;
use crate::{ScopeChain, SymbolTable};
use vize_carton::{CompactString, FxHashMap, FxHashSet};
use vize_relief::BindingType;

/// Complete semantic analysis summary for a Vue SFC.
///
/// This struct aggregates all analysis results and provides a unified
/// interface for downstream consumers (linter, transformer).
#[derive(Debug, Default)]
pub struct AnalysisSummary {
    /// Scope chain for template expressions
    pub scopes: ScopeChain,

    /// Symbol table for script bindings
    pub symbols: SymbolTable,

    /// Compiler macro information (defineProps, defineEmits, etc.)
    pub macros: MacroTracker,

    /// Reactivity tracking (ref, reactive, computed)
    pub reactivity: ReactivityTracker,

    /// TypeScript type resolution
    pub types: TypeResolver,

    /// Hoisting analysis for template optimization
    pub hoists: HoistTracker,

    /// Script binding metadata (for template access)
    pub bindings: BindingMetadata,

    /// Components used in template
    pub used_components: FxHashSet<CompactString>,

    /// Directives used in template
    pub used_directives: FxHashSet<CompactString>,

    /// Variables referenced in template but not defined
    pub undefined_refs: Vec<UndefinedRef>,

    /// Unused bindings (defined but not referenced in template)
    pub unused_bindings: Vec<CompactString>,
}

/// Binding metadata extracted from script analysis.
///
/// This is compatible with the existing BindingMetadata in atelier_core
/// but uses CompactString for efficiency.
#[derive(Debug, Default, Clone)]
pub struct BindingMetadata {
    /// Binding name to type mapping
    pub bindings: FxHashMap<CompactString, BindingType>,

    /// Whether this is from script setup
    pub is_script_setup: bool,

    /// Props aliases (local name -> prop key)
    pub props_aliases: FxHashMap<CompactString, CompactString>,
}

impl BindingMetadata {
    /// Create new empty binding metadata
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create for script setup
    #[inline]
    pub fn script_setup() -> Self {
        Self {
            is_script_setup: true,
            ..Default::default()
        }
    }

    /// Add a binding
    #[inline]
    pub fn add(&mut self, name: impl Into<CompactString>, binding_type: BindingType) {
        self.bindings.insert(name.into(), binding_type);
    }

    /// Get binding type for a name
    #[inline]
    pub fn get(&self, name: &str) -> Option<BindingType> {
        self.bindings.get(name).copied()
    }

    /// Check if a binding exists
    #[inline]
    pub fn contains(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }

    /// Check if a binding is a ref (needs .value in script)
    #[inline]
    pub fn is_ref(&self, name: &str) -> bool {
        matches!(
            self.get(name),
            Some(BindingType::SetupRef | BindingType::SetupMaybeRef)
        )
    }

    /// Check if a binding is from props
    #[inline]
    pub fn is_prop(&self, name: &str) -> bool {
        matches!(
            self.get(name),
            Some(BindingType::Props | BindingType::PropsAliased)
        )
    }

    /// Iterate over all bindings
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&str, BindingType)> {
        self.bindings.iter().map(|(k, v)| (k.as_str(), *v))
    }
}

/// An undefined reference in template
#[derive(Debug, Clone)]
pub struct UndefinedRef {
    /// The identifier name
    pub name: CompactString,
    /// Source offset
    pub offset: u32,
    /// Context (e.g., "v-if expression", "interpolation")
    pub context: CompactString,
}

impl AnalysisSummary {
    /// Create a new empty analysis summary
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a variable is defined in any scope
    #[inline]
    pub fn is_defined(&self, name: &str) -> bool {
        self.scopes.is_defined(name) || self.bindings.contains(name)
    }

    /// Get the binding type for a name
    #[inline]
    pub fn get_binding_type(&self, name: &str) -> Option<BindingType> {
        // First check scope chain (template-local variables)
        if let Some((_, binding)) = self.scopes.lookup(name) {
            return Some(binding.binding_type);
        }
        // Then check script bindings
        self.bindings.get(name)
    }

    /// Check if a name needs .value access in template
    ///
    /// In templates, refs are auto-unwrapped, so this returns false.
    /// Use `needs_value_in_script` for script context.
    #[inline]
    pub fn needs_value_in_template(&self, _name: &str) -> bool {
        // Templates auto-unwrap refs
        false
    }

    /// Check if a name needs .value access in script
    #[inline]
    pub fn needs_value_in_script(&self, name: &str) -> bool {
        self.reactivity.needs_value_access(name)
    }

    /// Check if a component is registered/imported
    #[inline]
    pub fn is_component_registered(&self, name: &str) -> bool {
        // Check if it's in used_components or is a known const binding
        // Components are typically imported as SetupConst
        self.used_components.contains(name)
            || self
                .bindings
                .get(name)
                .is_some_and(|t| matches!(t, BindingType::SetupConst))
    }

    /// Get props defined via defineProps
    pub fn get_props(&self) -> impl Iterator<Item = (&str, bool)> {
        self.macros
            .props()
            .iter()
            .map(|p| (p.name.as_str(), p.required))
    }

    /// Get emits defined via defineEmits
    pub fn get_emits(&self) -> impl Iterator<Item = &str> {
        self.macros.emits().iter().map(|e| e.name.as_str())
    }

    /// Get models defined via defineModel
    pub fn get_models(&self) -> impl Iterator<Item = &str> {
        self.macros.models().iter().map(|m| m.name.as_str())
    }

    /// Check if component uses async setup (top-level await)
    #[inline]
    pub fn is_async(&self) -> bool {
        self.macros.is_async()
    }

    /// Get analysis statistics for debugging
    pub fn stats(&self) -> AnalysisStats {
        AnalysisStats {
            scope_count: self.scopes.len(),
            symbol_count: self.symbols.len(),
            binding_count: self.bindings.bindings.len(),
            macro_count: self.macros.all_calls().len(),
            prop_count: self.macros.props().len(),
            emit_count: self.macros.emits().len(),
            model_count: self.macros.models().len(),
            hoist_count: self.hoists.count(),
            used_components: self.used_components.len(),
            used_directives: self.used_directives.len(),
            undefined_ref_count: self.undefined_refs.len(),
            unused_binding_count: self.unused_bindings.len(),
        }
    }
}

/// Statistics about the analysis
#[derive(Debug, Clone, Default)]
pub struct AnalysisStats {
    pub scope_count: usize,
    pub symbol_count: usize,
    pub binding_count: usize,
    pub macro_count: usize,
    pub prop_count: usize,
    pub emit_count: usize,
    pub model_count: usize,
    pub hoist_count: usize,
    pub used_components: usize,
    pub used_directives: usize,
    pub undefined_ref_count: usize,
    pub unused_binding_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binding_metadata() {
        let mut meta = BindingMetadata::script_setup();
        meta.add("count", BindingType::SetupRef);
        meta.add("state", BindingType::SetupReactiveConst);
        meta.add("msg", BindingType::Props);

        assert!(meta.is_script_setup);
        assert!(meta.is_ref("count"));
        assert!(!meta.is_ref("state"));
        assert!(meta.is_prop("msg"));
    }

    #[test]
    fn test_analysis_summary() {
        let mut summary = AnalysisSummary::new();
        summary.bindings.add("foo", BindingType::SetupRef);

        assert!(summary.is_defined("foo"));
        assert!(!summary.is_defined("bar"));
        assert_eq!(summary.get_binding_type("foo"), Some(BindingType::SetupRef));
    }
}
