//! Type checker for Vue SFC templates.

use crate::context::TypeContext;
use crate::diagnostic::{TypeDiagnostic, TypeErrorCode};
use crate::types::{CompletionItem, CompletionKind, TypeInfo};
use crate::CheckResult;

/// Type checker for Vue SFC templates.
///
/// The TypeChecker validates template expressions against the type context
/// derived from the script block.
#[derive(Debug, Default)]
pub struct TypeChecker {
    /// Enable strict mode (no implicit any).
    pub strict: bool,
    /// Enable Vue-specific checks.
    pub vue_checks: bool,
}

impl TypeChecker {
    /// Create a new type checker with default settings.
    pub fn new() -> Self {
        Self {
            strict: false,
            vue_checks: true,
        }
    }

    /// Create a strict type checker.
    pub fn strict() -> Self {
        Self {
            strict: true,
            vue_checks: true,
        }
    }

    /// Enable or disable strict mode.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Enable or disable Vue-specific checks.
    pub fn with_vue_checks(mut self, vue_checks: bool) -> Self {
        self.vue_checks = vue_checks;
        self
    }

    /// Check a template against a type context.
    ///
    /// # Arguments
    /// * `template` - The template content to check
    /// * `ctx` - The type context from the script block
    ///
    /// # Returns
    /// A CheckResult containing any type errors found
    pub fn check_template(&self, template: &str, ctx: &TypeContext) -> CheckResult {
        let mut result = CheckResult::new();

        // Find all expression interpolations {{ ... }}
        self.check_interpolations(template, ctx, &mut result);

        // Find all directive expressions (v-if, v-for, etc.)
        self.check_directives(template, ctx, &mut result);

        // Find all event handlers (@click, v-on:click)
        self.check_event_handlers(template, ctx, &mut result);

        // Find all v-bind expressions (:prop, v-bind:prop)
        self.check_bindings(template, ctx, &mut result);

        result
    }

    /// Check interpolation expressions {{ expr }}.
    fn check_interpolations(&self, template: &str, ctx: &TypeContext, result: &mut CheckResult) {
        let mut pos = 0;
        while let Some(start) = template[pos..].find("{{") {
            let abs_start = pos + start;
            if let Some(end) = template[abs_start..].find("}}") {
                let expr_start = abs_start + 2;
                let expr_end = abs_start + end;
                let expr = template[expr_start..expr_end].trim();

                if !expr.is_empty() {
                    self.check_expression(expr, expr_start as u32, expr_end as u32, ctx, result);
                }

                pos = abs_start + end + 2;
            } else {
                break;
            }
        }
    }

    /// Check directive expressions.
    fn check_directives(&self, template: &str, ctx: &TypeContext, result: &mut CheckResult) {
        // Check v-if, v-else-if, v-show expressions
        for directive in ["v-if", "v-else-if", "v-show"] {
            self.check_directive_values(template, directive, ctx, result);
        }

        // v-for has special syntax: "item in items" or "(item, index) in items"
        self.check_vfor_expressions(template, ctx, result);
    }

    /// Check values of a specific directive.
    fn check_directive_values(
        &self,
        template: &str,
        directive: &str,
        ctx: &TypeContext,
        result: &mut CheckResult,
    ) {
        let pattern = format!("{}=\"", directive);
        let mut pos = 0;

        while let Some(start) = template[pos..].find(&pattern) {
            let abs_start = pos + start + pattern.len();
            if let Some(end) = template[abs_start..].find('"') {
                let expr = &template[abs_start..abs_start + end];
                if !expr.is_empty() {
                    self.check_expression(
                        expr,
                        abs_start as u32,
                        (abs_start + end) as u32,
                        ctx,
                        result,
                    );
                }
                pos = abs_start + end + 1;
            } else {
                break;
            }
        }
    }

    /// Check v-for expressions.
    fn check_vfor_expressions(&self, template: &str, ctx: &TypeContext, result: &mut CheckResult) {
        let pattern = "v-for=\"";
        let mut pos = 0;

        while let Some(start) = template[pos..].find(pattern) {
            let abs_start = pos + start + pattern.len();
            if let Some(end) = template[abs_start..].find('"') {
                let expr = &template[abs_start..abs_start + end];

                // Parse "item in items" or "(item, index) in items"
                if let Some(in_pos) = expr.find(" in ") {
                    let iterable = expr[in_pos + 4..].trim();
                    let iterable_start = abs_start + in_pos + 4;
                    self.check_expression(
                        iterable,
                        iterable_start as u32,
                        (abs_start + end) as u32,
                        ctx,
                        result,
                    );
                }

                pos = abs_start + end + 1;
            } else {
                break;
            }
        }
    }

    /// Check event handlers.
    fn check_event_handlers(&self, template: &str, ctx: &TypeContext, result: &mut CheckResult) {
        // Check @event="handler" and v-on:event="handler"
        let patterns = ["@", "v-on:"];

        for pattern in patterns {
            let mut pos = 0;
            while let Some(start) = template[pos..].find(pattern) {
                let abs_start = pos + start + pattern.len();

                // Find the end of the event name and the ="
                if let Some(eq_pos) = template[abs_start..].find("=\"") {
                    let handler_start = abs_start + eq_pos + 2;
                    if let Some(end) = template[handler_start..].find('"') {
                        let handler = &template[handler_start..handler_start + end];

                        // Simple handler (just a function name)
                        if Self::is_simple_identifier(handler) {
                            self.check_identifier(
                                handler,
                                handler_start as u32,
                                (handler_start + end) as u32,
                                ctx,
                                result,
                            );
                        } else if !handler.is_empty() {
                            // Inline handler expression
                            self.check_expression(
                                handler,
                                handler_start as u32,
                                (handler_start + end) as u32,
                                ctx,
                                result,
                            );
                        }

                        pos = handler_start + end + 1;
                    } else {
                        break;
                    }
                } else {
                    pos = abs_start + 1;
                }
            }
        }
    }

    /// Check v-bind expressions.
    fn check_bindings(&self, template: &str, ctx: &TypeContext, result: &mut CheckResult) {
        // Check :prop="expr" and v-bind:prop="expr"
        let patterns = [(":", "="), ("v-bind:", "=")];

        for (prefix, suffix) in patterns {
            let mut pos = 0;
            while let Some(start) = template[pos..].find(prefix) {
                // Skip :: (CSS pseudo-selectors)
                if prefix == ":" && template[pos + start..].starts_with("::") {
                    pos = pos + start + 2;
                    continue;
                }

                let abs_start = pos + start + prefix.len();

                // Find ="
                if let Some(eq_pos) = template[abs_start..].find(&format!("{suffix}\"")) {
                    let expr_start = abs_start + eq_pos + 2;
                    if let Some(end) = template[expr_start..].find('"') {
                        let expr = &template[expr_start..expr_start + end];
                        if !expr.is_empty() {
                            self.check_expression(
                                expr,
                                expr_start as u32,
                                (expr_start + end) as u32,
                                ctx,
                                result,
                            );
                        }
                        pos = expr_start + end + 1;
                    } else {
                        break;
                    }
                } else {
                    pos = abs_start + 1;
                }
            }
        }
    }

    /// Check a single expression.
    fn check_expression(
        &self,
        expr: &str,
        start: u32,
        _end: u32,
        ctx: &TypeContext,
        result: &mut CheckResult,
    ) {
        // Extract identifiers from the expression and check each one
        for (ident, offset) in Self::extract_identifiers(expr) {
            let ident_start = start + offset as u32;
            let ident_end = ident_start + ident.len() as u32;
            self.check_identifier(ident, ident_start, ident_end, ctx, result);
        }
    }

    /// Check if an identifier exists in the context.
    fn check_identifier(
        &self,
        ident: &str,
        start: u32,
        end: u32,
        ctx: &TypeContext,
        result: &mut CheckResult,
    ) {
        // Skip keywords and literals
        if Self::is_keyword_or_literal(ident) {
            return;
        }

        // Skip $-prefixed globals ($event, $refs, etc.)
        if ident.starts_with('$') {
            return;
        }

        // Check if the identifier is defined
        if !ctx.has_binding(ident) && !ctx.globals.contains_key(ident) {
            result.add_diagnostic(TypeDiagnostic::error(
                TypeErrorCode::UnknownIdentifier,
                format!("Cannot find name '{}'", ident),
                start,
                end,
            ));
        }
    }

    /// Get type information at a specific offset.
    ///
    /// Returns the type of the expression or identifier at the given position.
    pub fn get_type_at(&self, template: &str, offset: u32, ctx: &TypeContext) -> Option<TypeInfo> {
        // Find what's at the offset
        let offset = offset as usize;

        // Check if we're in an interpolation
        if let Some((expr, expr_start)) = self.find_expression_at(template, offset) {
            let relative_offset = offset - expr_start;
            return self.get_type_in_expression(&expr, relative_offset, ctx);
        }

        None
    }

    /// Find the expression containing the given offset.
    fn find_expression_at(&self, template: &str, offset: usize) -> Option<(String, usize)> {
        // Check interpolations
        let mut pos = 0;
        while let Some(start) = template[pos..].find("{{") {
            let abs_start = pos + start;
            if let Some(end) = template[abs_start..].find("}}") {
                let expr_start = abs_start + 2;
                let expr_end = abs_start + end;

                if offset >= expr_start && offset <= expr_end {
                    return Some((
                        template[expr_start..expr_end].trim().to_string(),
                        expr_start,
                    ));
                }

                pos = abs_start + end + 2;
            } else {
                break;
            }
        }

        // Check directive values
        for directive in ["v-if", "v-else-if", "v-show", "v-for"] {
            if let Some((expr, start)) = self.find_directive_expr_at(template, directive, offset) {
                return Some((expr, start));
            }
        }

        None
    }

    /// Find a directive expression at offset.
    fn find_directive_expr_at(
        &self,
        template: &str,
        directive: &str,
        offset: usize,
    ) -> Option<(String, usize)> {
        let pattern = format!("{}=\"", directive);
        let mut pos = 0;

        while let Some(start) = template[pos..].find(&pattern) {
            let abs_start = pos + start + pattern.len();
            if let Some(end) = template[abs_start..].find('"') {
                if offset >= abs_start && offset <= abs_start + end {
                    return Some((template[abs_start..abs_start + end].to_string(), abs_start));
                }
                pos = abs_start + end + 1;
            } else {
                break;
            }
        }

        None
    }

    /// Get type information within an expression.
    fn get_type_in_expression(
        &self,
        expr: &str,
        offset: usize,
        ctx: &TypeContext,
    ) -> Option<TypeInfo> {
        // Find the identifier at the offset
        let ident = self.find_identifier_at(expr, offset)?;

        // Look up the type
        if let Some(binding) = ctx.get_binding(&ident) {
            return Some(binding.type_info.clone());
        }

        if let Some(type_info) = ctx.globals.get(&ident) {
            return Some(type_info.clone());
        }

        None
    }

    /// Find identifier at offset within an expression.
    fn find_identifier_at(&self, expr: &str, offset: usize) -> Option<String> {
        if offset >= expr.len() {
            return None;
        }

        let bytes = expr.as_bytes();

        // Check if we're on an identifier character
        if !Self::is_ident_char(bytes[offset] as char) {
            return None;
        }

        // Find the start of the identifier
        let mut start = offset;
        while start > 0 && Self::is_ident_char(bytes[start - 1] as char) {
            start -= 1;
        }

        // Find the end of the identifier
        let mut end = offset;
        while end < bytes.len() && Self::is_ident_char(bytes[end] as char) {
            end += 1;
        }

        // First char must be a valid start char
        if !Self::is_ident_start(bytes[start] as char) {
            return None;
        }

        Some(expr[start..end].to_string())
    }

    /// Get completions at a specific offset.
    pub fn get_completions(
        &self,
        _template: &str,
        _offset: u32,
        ctx: &TypeContext,
    ) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // Add all bindings as completions
        for (name, binding) in &ctx.bindings {
            let kind = match binding.kind {
                crate::context::BindingKind::Function => CompletionKind::Function,
                crate::context::BindingKind::Class => CompletionKind::Class,
                crate::context::BindingKind::Const
                | crate::context::BindingKind::Let
                | crate::context::BindingKind::Var => CompletionKind::Variable,
                crate::context::BindingKind::Ref
                | crate::context::BindingKind::Computed
                | crate::context::BindingKind::Reactive => CompletionKind::Variable,
                crate::context::BindingKind::Import => CompletionKind::Module,
                crate::context::BindingKind::Prop => CompletionKind::Property,
                _ => CompletionKind::Variable,
            };

            completions.push(
                CompletionItem::new(name, kind)
                    .with_detail(&binding.type_info.display)
                    .with_priority(10),
            );
        }

        // Add components
        for name in ctx.components.keys() {
            completions
                .push(CompletionItem::new(name, CompletionKind::Component).with_priority(20));
        }

        // Add globals
        for (name, type_info) in &ctx.globals {
            completions.push(
                CompletionItem::new(name, CompletionKind::Variable)
                    .with_detail(&type_info.display)
                    .with_priority(30),
            );
        }

        // Sort by priority then name
        completions.sort_by(|a, b| {
            a.sort_priority
                .cmp(&b.sort_priority)
                .then_with(|| a.label.cmp(&b.label))
        });

        completions
    }

    /// Extract identifiers from an expression.
    fn extract_identifiers(expr: &str) -> Vec<(&str, usize)> {
        let mut identifiers = Vec::new();
        let bytes = expr.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            // Skip non-identifier characters
            while i < bytes.len() && !Self::is_ident_start(bytes[i] as char) {
                i += 1;
            }

            if i >= bytes.len() {
                break;
            }

            let start = i;

            // Read the identifier
            while i < bytes.len() && Self::is_ident_char(bytes[i] as char) {
                i += 1;
            }

            if start < i {
                identifiers.push((&expr[start..i], start));
            }
        }

        identifiers
    }

    /// Check if a string is a simple identifier (no dots, brackets, etc.)
    fn is_simple_identifier(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        let mut chars = s.chars();
        let first = chars.next().unwrap();

        if !Self::is_ident_start(first) {
            return false;
        }

        chars.all(Self::is_ident_char)
    }

    /// Check if a character can start an identifier.
    fn is_ident_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_' || c == '$'
    }

    /// Check if a character can be part of an identifier.
    fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '$'
    }

    /// Check if a string is a keyword or literal.
    fn is_keyword_or_literal(s: &str) -> bool {
        matches!(
            s,
            "true"
                | "false"
                | "null"
                | "undefined"
                | "this"
                | "if"
                | "else"
                | "for"
                | "while"
                | "do"
                | "switch"
                | "case"
                | "default"
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
                | "instanceof"
                | "in"
                | "of"
                | "void"
                | "function"
                | "class"
                | "extends"
                | "const"
                | "let"
                | "var"
                | "import"
                | "export"
                | "async"
                | "await"
                | "yield"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{Binding, BindingKind};
    use crate::types::TypeKind;

    fn create_test_context() -> TypeContext {
        let mut ctx = TypeContext::new();
        ctx.add_binding(
            "count",
            Binding::new(
                "count",
                TypeInfo::new("Ref<number>", TypeKind::Ref),
                BindingKind::Ref,
            ),
        );
        ctx.add_binding(
            "message",
            Binding::new("message", TypeInfo::string(), BindingKind::Const),
        );
        ctx.add_binding(
            "handleClick",
            Binding::new(
                "handleClick",
                TypeInfo::new("() => void", TypeKind::Function),
                BindingKind::Function,
            ),
        );
        ctx
    }

    #[test]
    fn test_check_interpolation() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        let template = r#"<div>{{ count }}</div>"#;

        let result = checker.check_template(template, &ctx);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_check_unknown_identifier() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        let template = r#"<div>{{ unknownVar }}</div>"#;

        let result = checker.check_template(template, &ctx);
        assert!(result.has_errors());
        assert_eq!(result.error_count, 1);
        assert!(result.diagnostics[0].message.contains("unknownVar"));
    }

    #[test]
    fn test_check_directive() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        let template = r#"<div v-if="count > 0">visible</div>"#;

        let result = checker.check_template(template, &ctx);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_check_event_handler() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        let template = r#"<button @click="handleClick">Click</button>"#;

        let result = checker.check_template(template, &ctx);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_check_vbind() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        let template = r#"<input :value="message" />"#;

        let result = checker.check_template(template, &ctx);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_extract_identifiers() {
        let ids = TypeChecker::extract_identifiers("count + message.length");
        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0].0, "count");
        assert_eq!(ids[1].0, "message");
        assert_eq!(ids[2].0, "length");
    }

    #[test]
    fn test_is_simple_identifier() {
        assert!(TypeChecker::is_simple_identifier("foo"));
        assert!(TypeChecker::is_simple_identifier("_bar"));
        assert!(TypeChecker::is_simple_identifier("$baz"));
        assert!(!TypeChecker::is_simple_identifier("foo.bar"));
        assert!(!TypeChecker::is_simple_identifier("foo[0]"));
        assert!(!TypeChecker::is_simple_identifier("123"));
    }

    #[test]
    fn test_get_completions() {
        let checker = TypeChecker::new();
        let ctx = create_test_context();
        let template = r#"<div>{{ }}</div>"#;

        let completions = checker.get_completions(template, 8, &ctx);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "count"));
        assert!(completions.iter().any(|c| c.label == "message"));
    }
}
