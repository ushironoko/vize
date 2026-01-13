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

use crate::analysis::{
    AnalysisSummary, BindingMetadata, InvalidExport, InvalidExportKind, TypeExport, TypeExportKind,
    UndefinedRef,
};
use crate::macros::{EmitDefinition, ModelDefinition, PropDefinition};
use crate::reactivity::ReactiveKind;
use crate::types::TypeResolver;
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
    /// This is a fast pass that extracts:
    /// - defineProps/defineEmits/defineModel calls
    /// - Top-level bindings (const, let, function, class)
    /// - Import statements
    /// - Reactivity wrappers (ref, reactive, computed)
    ///
    /// Performance: O(n) single pass through tokens
    pub fn analyze_script(&mut self, source: &str) -> &mut Self {
        if !self.options.analyze_script {
            return self;
        }

        self.script_analyzed = true;
        self.summary.bindings = BindingMetadata::script_setup();

        // Fast tokenized analysis (no full AST parse)
        self.extract_macros_fast(source);
        self.extract_bindings_fast(source);

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
    // Fast Script Analysis (Token-based, no full AST)
    // =========================================================================

    /// Extract Vue compiler macros using fast string scanning.
    ///
    /// This avoids full AST parsing for the common cases.
    fn extract_macros_fast(&mut self, source: &str) {
        let bytes = source.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        while i < len {
            // Skip whitespace
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }

            if i >= len {
                break;
            }

            // Look for macro calls
            if self.try_parse_macro(source, &mut i) {
                continue;
            }

            // Skip to next line or token
            i += 1;
        }
    }

    /// Try to parse a macro call at current position.
    ///
    /// Returns true if a macro was found and parsed.
    #[inline]
    fn try_parse_macro(&mut self, source: &str, pos: &mut usize) -> bool {
        let rest = &source[*pos..];

        // defineProps
        if rest.starts_with("defineProps") {
            if let Some((props, advance)) = self.parse_define_props(rest) {
                for prop in props {
                    let name = prop.name.clone();
                    self.summary.macros.add_prop(prop);
                    self.summary.bindings.add(name, BindingType::Props);
                }
                *pos += advance;
                return true;
            }
        }

        // defineEmits
        if rest.starts_with("defineEmits") {
            if let Some((emits, advance)) = self.parse_define_emits(rest) {
                for emit in emits {
                    self.summary.macros.add_emit(emit);
                }
                *pos += advance;
                return true;
            }
        }

        // defineModel
        if rest.starts_with("defineModel") {
            if let Some((model, advance)) = self.parse_define_model(rest) {
                self.summary.macros.add_model(model.clone());
                self.summary
                    .bindings
                    .add(model.name.clone(), BindingType::SetupRef);
                *pos += advance;
                return true;
            }
        }

        // withDefaults (wraps defineProps)
        if rest.starts_with("withDefaults") {
            if let Some(advance) = self.parse_with_defaults(rest) {
                *pos += advance;
                return true;
            }
        }

        false
    }

    /// Parse defineProps call (simplified fast version)
    fn parse_define_props(&mut self, source: &str) -> Option<(Vec<PropDefinition>, usize)> {
        // Find opening paren or angle bracket
        let start = source.find(['(', '<'])?;
        let opener = source.as_bytes()[start];

        let (closer, is_type) = match opener {
            b'<' => (b'>', true),
            b'(' => (b')', false),
            _ => return None,
        };

        // Find matching closer
        let content_start = start + 1;
        let content_end = self.find_matching(source, content_start, opener, closer)?;

        let content = &source[content_start..content_end];
        let props = if is_type {
            self.parse_props_from_type(content)
        } else {
            self.parse_props_from_runtime(content)
        };

        Some((props, content_end + 1))
    }

    /// Parse defineEmits call
    fn parse_define_emits(&mut self, source: &str) -> Option<(Vec<EmitDefinition>, usize)> {
        let start = source.find(['(', '<'])?;
        let opener = source.as_bytes()[start];

        let (closer, is_type) = match opener {
            b'<' => (b'>', true),
            b'(' => (b')', false),
            _ => return None,
        };

        let content_start = start + 1;
        let content_end = self.find_matching(source, content_start, opener, closer)?;

        let content = &source[content_start..content_end];
        let emits = if is_type {
            self.parse_emits_from_type(content)
        } else {
            self.parse_emits_from_runtime(content)
        };

        Some((emits, content_end + 1))
    }

    /// Parse defineModel call
    fn parse_define_model(&mut self, source: &str) -> Option<(ModelDefinition, usize)> {
        let paren_start = source.find('(')?;
        let paren_end = self.find_matching(source, paren_start + 1, b'(', b')')?;

        let content = source[paren_start + 1..paren_end].trim();

        // Extract model name (first string argument or 'modelValue' by default)
        let name = if content.starts_with('\'') || content.starts_with('"') {
            let quote = content.as_bytes()[0];
            let end = content[1..].find(|c: char| c as u8 == quote)?;
            CompactString::new(&content[1..=end])
        } else {
            CompactString::new("modelValue")
        };

        Some((
            ModelDefinition {
                name: name.clone(),
                local_name: name,
                model_type: None,
                required: false,
                default_value: None,
            },
            paren_end + 1,
        ))
    }

    /// Parse withDefaults wrapper
    fn parse_with_defaults(&mut self, source: &str) -> Option<usize> {
        let paren_start = source.find('(')?;
        let paren_end = self.find_matching(source, paren_start + 1, b'(', b')')?;

        let inner = &source[paren_start + 1..paren_end];

        // withDefaults wraps defineProps - parse the inner call
        if inner.trim_start().starts_with("defineProps") {
            if let Some((props, _)) = self.parse_define_props(inner.trim_start()) {
                for prop in props {
                    self.summary.macros.add_prop(prop);
                }
            }
        }

        Some(paren_end + 1)
    }

    /// Find matching bracket/paren
    #[inline]
    fn find_matching(&self, source: &str, start: usize, open: u8, close: u8) -> Option<usize> {
        let bytes = source.as_bytes();
        let mut depth = 1;
        let mut i = start;
        let mut in_string = false;
        let mut string_char = 0u8;

        while i < bytes.len() && depth > 0 {
            let c = bytes[i];

            if in_string {
                if c == string_char && (i == 0 || bytes[i - 1] != b'\\') {
                    in_string = false;
                }
            } else {
                match c {
                    b'"' | b'\'' | b'`' => {
                        in_string = true;
                        string_char = c;
                    }
                    _ if c == open => depth += 1,
                    _ if c == close => depth -= 1,
                    _ => {}
                }
            }
            i += 1;
        }

        if depth == 0 {
            Some(i - 1)
        } else {
            None
        }
    }

    /// Parse props from type annotation
    fn parse_props_from_type(&self, content: &str) -> Vec<PropDefinition> {
        let mut props = Vec::new();
        let trimmed = content.trim();

        // Handle { prop1: Type, prop2?: Type }
        if !trimmed.starts_with('{') {
            // Type reference - can't extract props without type resolution
            return props;
        }

        let inner = &trimmed[1..trimmed.len().saturating_sub(1)];

        for segment in self.split_type_members(inner) {
            let segment = segment.trim();
            if segment.is_empty() {
                continue;
            }

            if let Some(colon_pos) = segment.find(':') {
                let name_part = segment[..colon_pos].trim();
                let optional = name_part.ends_with('?');
                let name = name_part.trim_end_matches('?').trim();

                if !name.is_empty() && is_identifier(name) {
                    props.push(PropDefinition {
                        name: CompactString::new(name),
                        required: !optional,
                        prop_type: None,
                        default_value: None,
                    });
                }
            }
        }

        props
    }

    /// Parse props from runtime definition
    fn parse_props_from_runtime(&self, content: &str) -> Vec<PropDefinition> {
        let mut props = Vec::new();
        let trimmed = content.trim();

        // Handle ['prop1', 'prop2'] array syntax
        if trimmed.starts_with('[') {
            for name in self.extract_string_array(trimmed) {
                props.push(PropDefinition {
                    name: CompactString::new(name),
                    required: false,
                    prop_type: None,
                    default_value: None,
                });
            }
            return props;
        }

        // Handle { prop1: Type, prop2: { type: Type, required: true } }
        if trimmed.starts_with('{') {
            let inner = &trimmed[1..trimmed.len().saturating_sub(1)];
            for segment in self.split_object_members(inner) {
                let segment = segment.trim();
                if let Some(colon_pos) = segment.find(':') {
                    let name = segment[..colon_pos].trim();
                    if is_identifier(name) {
                        props.push(PropDefinition {
                            name: CompactString::new(name),
                            required: segment.contains("required: true")
                                || segment.contains("required:true"),
                            prop_type: None,
                            default_value: None,
                        });
                    }
                }
            }
        }

        props
    }

    /// Parse emits from type annotation
    fn parse_emits_from_type(&self, content: &str) -> Vec<EmitDefinition> {
        let mut emits = Vec::new();
        let resolver = TypeResolver::new();
        let emit_names = resolver.extract_emits(content);

        for name in emit_names {
            emits.push(EmitDefinition {
                name,
                payload_type: None,
            });
        }

        emits
    }

    /// Parse emits from runtime definition
    fn parse_emits_from_runtime(&self, content: &str) -> Vec<EmitDefinition> {
        let mut emits = Vec::new();

        for name in self.extract_string_array(content) {
            emits.push(EmitDefinition {
                name: CompactString::new(name),
                payload_type: None,
            });
        }

        emits
    }

    /// Extract string values from array syntax ['a', 'b']
    fn extract_string_array<'a>(&self, content: &'a str) -> Vec<&'a str> {
        let mut strings = Vec::new();
        let trimmed = content.trim();

        if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
            return strings;
        }

        let inner = &trimmed[1..trimmed.len() - 1];
        let bytes = inner.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            // Skip whitespace and commas
            while i < bytes.len() && (bytes[i].is_ascii_whitespace() || bytes[i] == b',') {
                i += 1;
            }

            if i >= bytes.len() {
                break;
            }

            // Found string start
            let quote = bytes[i];
            if quote == b'\'' || quote == b'"' {
                let start = i + 1;
                i += 1;

                while i < bytes.len() && bytes[i] != quote {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }

                if i < bytes.len() {
                    strings.push(&inner[start..i]);
                }
            }
            i += 1;
        }

        strings
    }

    /// Split type members (handles nested braces)
    fn split_type_members<'a>(&self, content: &'a str) -> Vec<&'a str> {
        let mut members = Vec::new();
        let mut depth = 0;
        let mut start = 0;

        for (i, c) in content.char_indices() {
            match c {
                '{' | '<' | '(' | '[' => depth += 1,
                '}' | '>' | ')' | ']' => depth -= 1,
                // Split on comma, semicolon, or newline at depth 0
                ',' | ';' | '\n' if depth == 0 => {
                    let segment = &content[start..i];
                    if !segment.trim().is_empty() {
                        members.push(segment);
                    }
                    start = i + 1;
                }
                _ => {}
            }
        }

        if start < content.len() {
            let segment = &content[start..];
            if !segment.trim().is_empty() {
                members.push(segment);
            }
        }

        members
    }

    /// Split object members
    fn split_object_members<'a>(&self, content: &'a str) -> Vec<&'a str> {
        let mut members = Vec::new();
        let mut depth = 0;
        let mut start = 0;
        let mut in_string = false;
        let mut string_char = ' ';

        for (i, c) in content.char_indices() {
            if in_string {
                if c == string_char {
                    in_string = false;
                }
                continue;
            }

            match c {
                '"' | '\'' => {
                    in_string = true;
                    string_char = c;
                }
                '{' | '[' | '(' => depth += 1,
                '}' | ']' | ')' => depth -= 1,
                ',' if depth == 0 => {
                    members.push(&content[start..i]);
                    start = i + 1;
                }
                _ => {}
            }
        }

        if start < content.len() {
            members.push(&content[start..]);
        }

        members
    }

    /// Extract top-level bindings (fast scan)
    fn extract_bindings_fast(&mut self, source: &str) {
        let bytes = source.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        while i < len {
            // Skip whitespace
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }

            if i >= len {
                break;
            }

            // Look for binding declarations
            let rest = &source[i..];

            // const x = ref(...) / reactive(...) / computed(...)
            if rest.starts_with("const ") {
                if let Some(advance) = self.parse_const_binding(rest) {
                    i += advance;
                    continue;
                }
            }

            // let x = ...
            if rest.starts_with("let ") {
                if let Some(advance) = self.parse_let_binding(rest) {
                    i += advance;
                    continue;
                }
            }

            // function x() / async function x()
            if rest.starts_with("function ") || rest.starts_with("async function ") {
                if let Some(advance) = self.parse_function_binding(rest) {
                    i += advance;
                    continue;
                }
            }

            // import ... from ...
            if rest.starts_with("import ") {
                if let Some(advance) = self.parse_import_binding(rest) {
                    i += advance;
                    continue;
                }
            }

            // export type / export interface (valid - hoisted)
            if rest.starts_with("export type ") {
                if let Some(advance) = self.parse_type_export(rest, TypeExportKind::Type, i as u32)
                {
                    i += advance;
                    continue;
                }
            }
            if rest.starts_with("export interface ") {
                if let Some(advance) =
                    self.parse_type_export(rest, TypeExportKind::Interface, i as u32)
                {
                    i += advance;
                    continue;
                }
            }

            // export const/let/var/function/class/default (invalid in script setup)
            if rest.starts_with("export ") {
                if let Some(advance) = self.parse_invalid_export(rest, i as u32) {
                    i += advance;
                    continue;
                }
            }

            i += 1;
        }
    }

    /// Parse const binding and detect reactivity
    fn parse_const_binding(&mut self, source: &str) -> Option<usize> {
        // const name = ...
        let after_const = &source[6..];
        let name_end = after_const.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')?;
        let name = &after_const[..name_end];

        if name.is_empty() || !is_identifier(name) {
            return None;
        }

        // Find = sign
        let eq_pos = after_const[name_end..].find('=')?;
        let value_start = name_end + eq_pos + 1;
        let value = after_const[value_start..].trim_start();

        // Detect reactivity wrapper
        let binding_type = if value.starts_with("ref(") || value.starts_with("shallowRef(") {
            self.summary
                .reactivity
                .register(CompactString::new(name), ReactiveKind::Ref, 0);
            BindingType::SetupRef
        } else if value.starts_with("computed(") {
            self.summary
                .reactivity
                .register(CompactString::new(name), ReactiveKind::Computed, 0);
            BindingType::SetupRef
        } else if value.starts_with("reactive(") || value.starts_with("shallowReactive(") {
            self.summary
                .reactivity
                .register(CompactString::new(name), ReactiveKind::Reactive, 0);
            BindingType::SetupReactiveConst
        } else if value.starts_with("toRef(") || value.starts_with("toRefs(") {
            BindingType::SetupMaybeRef
        } else {
            BindingType::SetupConst
        };

        self.summary.bindings.add(name, binding_type);

        // Find end of statement (simplified)
        let end = source.find('\n').unwrap_or(source.len());
        Some(end)
    }

    /// Parse let binding
    fn parse_let_binding(&mut self, source: &str) -> Option<usize> {
        let after_let = &source[4..];
        let name_end = after_let.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')?;
        let name = &after_let[..name_end];

        if !name.is_empty() && is_identifier(name) {
            self.summary.bindings.add(name, BindingType::SetupLet);
        }

        let end = source.find('\n').unwrap_or(source.len());
        Some(end)
    }

    /// Parse function binding
    fn parse_function_binding(&mut self, source: &str) -> Option<usize> {
        let start = if source.starts_with("async ") {
            source.find("function ")? + 9
        } else {
            9
        };

        let rest = &source[start..];
        let name_end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')?;
        let name = &rest[..name_end];

        if !name.is_empty() && is_identifier(name) {
            self.summary.bindings.add(name, BindingType::SetupConst);
        }

        // Skip to end of function (find matching brace)
        let brace_start = source.find('{')?;
        let brace_end = self.find_matching(source, brace_start + 1, b'{', b'}')?;
        Some(brace_end + 1)
    }

    /// Parse import binding
    fn parse_import_binding(&mut self, source: &str) -> Option<usize> {
        // import { x, y } from '...' or import x from '...'
        let end = source.find('\n').unwrap_or(source.len());
        let line = &source[..end];

        // Skip type-only imports
        if line.contains("import type") {
            return Some(end);
        }

        // Extract imported names
        if let Some(brace_start) = line.find('{') {
            if let Some(brace_end) = line.find('}') {
                let imports = &line[brace_start + 1..brace_end];
                for part in imports.split(',') {
                    let part = part.trim();
                    // Handle 'x as y' syntax
                    let name = if let Some(as_pos) = part.find(" as ") {
                        &part[as_pos + 4..]
                    } else {
                        part
                    };
                    let name = name.trim();
                    if !name.is_empty() && is_identifier(name) {
                        self.summary.bindings.add(name, BindingType::SetupConst);
                    }
                }
            }
        } else if let Some(from_pos) = line.find(" from ") {
            // Default import: import x from '...'
            let after_import = &line[7..from_pos];
            let name = after_import.trim();
            if !name.is_empty() && is_identifier(name) {
                self.summary.bindings.add(name, BindingType::SetupConst);
            }
        }

        Some(end)
    }

    /// Parse type export (export type / export interface) - valid in script setup, hoisted
    fn parse_type_export(
        &mut self,
        source: &str,
        kind: TypeExportKind,
        start_offset: u32,
    ) -> Option<usize> {
        // export type Foo = ... or export interface Foo { ... }
        let prefix_len = match kind {
            TypeExportKind::Type => 12,      // "export type "
            TypeExportKind::Interface => 17, // "export interface "
        };

        let after_keyword = &source[prefix_len..];
        let name_end = after_keyword.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')?;
        let name = &after_keyword[..name_end];

        if name.is_empty() || !is_identifier(name) {
            return None;
        }

        // Find end of declaration
        let end = match kind {
            TypeExportKind::Type => {
                // Find end of type alias (simplified: look for newline or semicolon at depth 0)
                let mut depth = 0;
                let mut pos = 0;
                for (i, c) in source.char_indices() {
                    match c {
                        '{' | '<' | '(' | '[' => depth += 1,
                        '}' | '>' | ')' | ']' => depth -= 1,
                        ';' | '\n' if depth == 0 && i > prefix_len + name_end => {
                            pos = i;
                            break;
                        }
                        _ => {}
                    }
                }
                if pos == 0 {
                    source.len()
                } else {
                    pos + 1
                }
            }
            TypeExportKind::Interface => {
                // Find matching brace
                if let Some(brace_start) = source.find('{') {
                    self.find_matching(source, brace_start + 1, b'{', b'}')
                        .map(|e| e + 1)
                        .unwrap_or(source.len())
                } else {
                    source.find('\n').unwrap_or(source.len())
                }
            }
        };

        self.summary.type_exports.push(TypeExport {
            name: CompactString::new(name),
            kind,
            start: start_offset,
            end: start_offset + end as u32,
            hoisted: true,
        });

        Some(end)
    }

    /// Parse invalid export (const/let/var/function/class/default) - invalid in script setup
    fn parse_invalid_export(&mut self, source: &str, start_offset: u32) -> Option<usize> {
        let after_export = source[7..].trim_start(); // Skip "export "

        // Determine kind and extract name
        let (kind, name, advance) = if let Some(rest) = after_export.strip_prefix("const ") {
            let name_end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')?;
            let name = &rest[..name_end];
            let end = source.find('\n').unwrap_or(source.len());
            (InvalidExportKind::Const, name, end)
        } else if let Some(rest) = after_export.strip_prefix("let ") {
            let name_end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')?;
            let name = &rest[..name_end];
            let end = source.find('\n').unwrap_or(source.len());
            (InvalidExportKind::Let, name, end)
        } else if let Some(rest) = after_export.strip_prefix("var ") {
            let name_end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')?;
            let name = &rest[..name_end];
            let end = source.find('\n').unwrap_or(source.len());
            (InvalidExportKind::Var, name, end)
        } else if let Some(rest) = after_export.strip_prefix("function ") {
            let name_end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')?;
            let name = &rest[..name_end];
            // Skip to end of function
            let end = if let Some(brace_start) = source.find('{') {
                self.find_matching(source, brace_start + 1, b'{', b'}')
                    .map(|e| e + 1)
                    .unwrap_or(source.len())
            } else {
                source.find('\n').unwrap_or(source.len())
            };
            (InvalidExportKind::Function, name, end)
        } else if let Some(rest) = after_export.strip_prefix("async function ") {
            let name_end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')?;
            let name = &rest[..name_end];
            let end = if let Some(brace_start) = source.find('{') {
                self.find_matching(source, brace_start + 1, b'{', b'}')
                    .map(|e| e + 1)
                    .unwrap_or(source.len())
            } else {
                source.find('\n').unwrap_or(source.len())
            };
            (InvalidExportKind::Function, name, end)
        } else if let Some(rest) = after_export.strip_prefix("class ") {
            let name_end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')?;
            let name = &rest[..name_end];
            let end = if let Some(brace_start) = source.find('{') {
                self.find_matching(source, brace_start + 1, b'{', b'}')
                    .map(|e| e + 1)
                    .unwrap_or(source.len())
            } else {
                source.find('\n').unwrap_or(source.len())
            };
            (InvalidExportKind::Class, name, end)
        } else if after_export.starts_with("default ") {
            let end = source.find('\n').unwrap_or(source.len());
            (InvalidExportKind::Default, "default", end)
        } else {
            // Not a recognized export pattern (might be re-export like "export { ... }")
            return None;
        };

        if !name.is_empty() && is_identifier(name) {
            self.summary.invalid_exports.push(InvalidExport {
                name: CompactString::new(name),
                kind,
                start: start_offset,
                end: start_offset + advance as u32,
            });
        }

        Some(advance)
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

/// Check if a string is a valid identifier
#[inline]
fn is_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    (first.is_ascii_alphabetic() || first == '_' || first == '$')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

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
    fn test_is_identifier() {
        assert!(is_identifier("count"));
        assert!(is_identifier("_private"));
        assert!(is_identifier("$ref"));
        assert!(!is_identifier("123abc"));
        assert!(!is_identifier(""));
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
    fn test_extract_string_array() {
        let analyzer = Analyzer::new();
        let strings = analyzer.extract_string_array("['foo', 'bar', 'baz']");
        assert_eq!(strings, vec!["foo", "bar", "baz"]);

        let strings = analyzer.extract_string_array(r#"["a", "b"]"#);
        assert_eq!(strings, vec!["a", "b"]);
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
