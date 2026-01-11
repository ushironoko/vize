//! References provider for Vue SFC files.
//!
//! Provides find-all-references for:
//! - Script bindings used in template
//! - Script bindings used in other script code
//! - Script bindings used in style v-bind()

use tower_lsp::lsp_types::{Location, Position, Range};

use super::IdeContext;

/// References service for finding all references to a symbol.
pub struct ReferencesService;

impl ReferencesService {
    /// Find all references to the symbol at the current position.
    pub fn references(ctx: &IdeContext, include_declaration: bool) -> Option<Vec<Location>> {
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset)?;

        if word.is_empty() {
            return None;
        }

        let mut locations = Vec::new();

        // Find definition location if requested
        if include_declaration {
            if let Some(def_loc) = Self::find_definition_location(ctx, &word) {
                locations.push(def_loc);
            }
        }

        // Find references in template
        locations.extend(Self::find_references_in_template(ctx, &word));

        // Find references in script
        locations.extend(Self::find_references_in_script(ctx, &word));

        // Find references in style
        locations.extend(Self::find_references_in_style(ctx, &word));

        if locations.is_empty() {
            None
        } else {
            // Remove duplicates
            locations.sort_by(|a, b| {
                a.range
                    .start
                    .line
                    .cmp(&b.range.start.line)
                    .then(a.range.start.character.cmp(&b.range.start.character))
            });
            locations.dedup_by(|a, b| a.range.start == b.range.start && a.range.end == b.range.end);
            Some(locations)
        }
    }

    /// Find the definition location of a symbol.
    fn find_definition_location(ctx: &IdeContext, word: &str) -> Option<Location> {
        // Check script setup first
        if let Some(ref virtual_docs) = ctx.virtual_docs {
            if let Some(ref script_setup) = virtual_docs.script_setup {
                if let Some(loc) = Self::find_binding_in_script(&script_setup.content, word) {
                    let script_start_line = Self::get_script_setup_start_line(&ctx.content)?;
                    let (line, character) = Self::offset_to_position(&script_setup.content, loc);

                    return Some(Location {
                        uri: ctx.uri.clone(),
                        range: Range {
                            start: Position {
                                line: script_start_line + line,
                                character,
                            },
                            end: Position {
                                line: script_start_line + line,
                                character: character + word.len() as u32,
                            },
                        },
                    });
                }
            }

            // Check regular script
            if let Some(ref script) = virtual_docs.script {
                if let Some(loc) = Self::find_binding_in_script(&script.content, word) {
                    let script_start_line = Self::get_script_start_line(&ctx.content)?;
                    let (line, character) = Self::offset_to_position(&script.content, loc);

                    return Some(Location {
                        uri: ctx.uri.clone(),
                        range: Range {
                            start: Position {
                                line: script_start_line + line,
                                character,
                            },
                            end: Position {
                                line: script_start_line + line,
                                character: character + word.len() as u32,
                            },
                        },
                    });
                }
            }
        }

        None
    }

    /// Find references to a symbol in the template block.
    fn find_references_in_template(ctx: &IdeContext, word: &str) -> Vec<Location> {
        let mut locations = Vec::new();

        let options = vize_atelier_sfc::SfcParseOptions::default();
        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
            return locations;
        };

        let Some(ref template) = descriptor.template else {
            return locations;
        };

        let template_content = template.content.as_ref();
        let template_start_line = template.loc.start_line as u32;

        // Find all occurrences of the word in template
        // This includes:
        // - Interpolations: {{ word }}
        // - Directive expressions: v-if="word", :prop="word", @event="word"

        // Parse template to find expressions
        let allocator = vize_carton::Bump::new();
        let (ast, _) = vize_armature::parse(&allocator, template_content);

        // Extract expression locations from the AST
        let expressions = Self::extract_template_expressions(&ast);

        for (expr_text, expr_offset) in expressions {
            // Find word occurrences within the expression
            let word_positions = Self::find_word_occurrences(&expr_text, word);

            for word_offset_in_expr in word_positions {
                let absolute_offset = expr_offset + word_offset_in_expr;
                let (line, character) = Self::offset_to_position(template_content, absolute_offset);

                locations.push(Location {
                    uri: ctx.uri.clone(),
                    range: Range {
                        start: Position {
                            line: template_start_line + line - 1,
                            character,
                        },
                        end: Position {
                            line: template_start_line + line - 1,
                            character: character + word.len() as u32,
                        },
                    },
                });
            }
        }

        // Also do a simple text search for the word in template
        // This catches cases the AST might miss
        let simple_refs = Self::find_simple_references_in_content(
            template_content,
            word,
            template_start_line - 1,
        );

        for loc in simple_refs {
            // Check if this location is already in our list
            let is_duplicate = locations.iter().any(|existing| {
                existing.range.start.line == loc.range.start.line
                    && existing.range.start.character == loc.range.start.character
            });
            if !is_duplicate {
                locations.push(Location {
                    uri: ctx.uri.clone(),
                    range: loc.range,
                });
            }
        }

        locations
    }

    /// Find references to a symbol in the script block.
    fn find_references_in_script(ctx: &IdeContext, word: &str) -> Vec<Location> {
        let mut locations = Vec::new();

        let options = vize_atelier_sfc::SfcParseOptions::default();
        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
            return locations;
        };

        // Check script setup
        if let Some(ref script_setup) = descriptor.script_setup {
            let script_content = script_setup.content.as_ref();
            let script_start_line = script_setup.loc.start_line as u32;

            let refs = Self::find_identifier_references_in_script(script_content, word);
            for (line, character) in refs {
                locations.push(Location {
                    uri: ctx.uri.clone(),
                    range: Range {
                        start: Position {
                            line: script_start_line + line - 1,
                            character,
                        },
                        end: Position {
                            line: script_start_line + line - 1,
                            character: character + word.len() as u32,
                        },
                    },
                });
            }
        }

        // Check regular script
        if let Some(ref script) = descriptor.script {
            let script_content = script.content.as_ref();
            let script_start_line = script.loc.start_line as u32;

            let refs = Self::find_identifier_references_in_script(script_content, word);
            for (line, character) in refs {
                locations.push(Location {
                    uri: ctx.uri.clone(),
                    range: Range {
                        start: Position {
                            line: script_start_line + line - 1,
                            character,
                        },
                        end: Position {
                            line: script_start_line + line - 1,
                            character: character + word.len() as u32,
                        },
                    },
                });
            }
        }

        locations
    }

    /// Find references to a symbol in style blocks (v-bind).
    fn find_references_in_style(ctx: &IdeContext, word: &str) -> Vec<Location> {
        let mut locations = Vec::new();

        let options = vize_atelier_sfc::SfcParseOptions::default();
        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
            return locations;
        };

        for style in &descriptor.styles {
            let style_content = style.content.as_ref();
            let style_start_line = style.loc.start_line as u32;

            // Find v-bind() references
            let refs = Self::find_vbind_references_in_style(style_content, word);
            for (line, character) in refs {
                locations.push(Location {
                    uri: ctx.uri.clone(),
                    range: Range {
                        start: Position {
                            line: style_start_line + line - 1,
                            character,
                        },
                        end: Position {
                            line: style_start_line + line - 1,
                            character: character + word.len() as u32,
                        },
                    },
                });
            }
        }

        locations
    }

    /// Extract expressions from template AST.
    fn extract_template_expressions<'a>(ast: &vize_armature::RootNode<'a>) -> Vec<(String, usize)> {
        let mut expressions = Vec::new();
        Self::visit_children_for_expressions(&ast.children, &mut expressions);
        expressions
    }

    /// Visit children to extract expressions.
    fn visit_children_for_expressions<'a>(
        children: &[vize_relief::ast::TemplateChildNode<'a>],
        expressions: &mut Vec<(String, usize)>,
    ) {
        use vize_relief::ast::*;

        for child in children {
            match child {
                TemplateChildNode::Element(el) => {
                    // Check directives
                    for prop in &el.props {
                        if let PropNode::Directive(dir) = prop {
                            if let Some(ref exp) = dir.exp {
                                if let Some((text, offset)) = Self::get_expression_info(exp) {
                                    expressions.push((text, offset));
                                }
                            }
                        }
                    }
                    // Visit children
                    Self::visit_children_for_expressions(&el.children, expressions);
                }
                TemplateChildNode::Interpolation(interp) => {
                    if let Some((text, offset)) = Self::get_expression_info(&interp.content) {
                        expressions.push((text, offset));
                    }
                }
                TemplateChildNode::If(if_node) => {
                    for branch in &if_node.branches {
                        if let Some(ref cond) = branch.condition {
                            if let Some((text, offset)) = Self::get_expression_info(cond) {
                                expressions.push((text, offset));
                            }
                        }
                        Self::visit_children_for_expressions(&branch.children, expressions);
                    }
                }
                TemplateChildNode::For(for_node) => {
                    if let Some((text, offset)) = Self::get_expression_info(&for_node.source) {
                        expressions.push((text, offset));
                    }
                    Self::visit_children_for_expressions(&for_node.children, expressions);
                }
                TemplateChildNode::IfBranch(branch) => {
                    if let Some(ref cond) = branch.condition {
                        if let Some((text, offset)) = Self::get_expression_info(cond) {
                            expressions.push((text, offset));
                        }
                    }
                    Self::visit_children_for_expressions(&branch.children, expressions);
                }
                _ => {}
            }
        }
    }

    /// Get expression text and offset from ExpressionNode.
    fn get_expression_info(expr: &vize_relief::ast::ExpressionNode) -> Option<(String, usize)> {
        use vize_relief::ast::*;

        match expr {
            ExpressionNode::Simple(simple) => {
                if simple.content.is_empty() {
                    None
                } else {
                    Some((simple.content.to_string(), simple.loc.start.offset as usize))
                }
            }
            ExpressionNode::Compound(compound) => {
                // For compound expressions, we can't easily get the text
                // Return the location but mark as compound
                Some(("<compound>".to_string(), compound.loc.start.offset as usize))
            }
        }
    }

    /// Find all occurrences of a word in a string.
    fn find_word_occurrences(text: &str, word: &str) -> Vec<usize> {
        let mut positions = Vec::new();
        let mut start = 0;

        while let Some(pos) = text[start..].find(word) {
            let absolute_pos = start + pos;

            // Check word boundaries
            let before_ok =
                absolute_pos == 0 || !Self::is_identifier_char(text.as_bytes()[absolute_pos - 1]);
            let after_ok = absolute_pos + word.len() >= text.len()
                || !Self::is_identifier_char(text.as_bytes()[absolute_pos + word.len()]);

            if before_ok && after_ok {
                positions.push(absolute_pos);
            }

            start = absolute_pos + 1;
        }

        positions
    }

    /// Find simple text references in content.
    fn find_simple_references_in_content(
        content: &str,
        word: &str,
        base_line: u32,
    ) -> Vec<Location> {
        let mut locations = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let line_positions = Self::find_word_occurrences(line, word);

            for pos in line_positions {
                // Check if this is in a binding context
                // (inside {{ }}, after v-*, after :, after @, etc.)
                if Self::is_in_binding_context(line, pos) {
                    locations.push(Location {
                        uri: tower_lsp::lsp_types::Url::parse("file:///dummy").unwrap(),
                        range: Range {
                            start: Position {
                                line: base_line + line_idx as u32,
                                character: pos as u32,
                            },
                            end: Position {
                                line: base_line + line_idx as u32,
                                character: pos as u32 + word.len() as u32,
                            },
                        },
                    });
                }
            }
        }

        locations
    }

    /// Check if a position is in a binding context.
    fn is_in_binding_context(line: &str, pos: usize) -> bool {
        let before = &line[..pos];

        // Check for interpolation: {{
        if before.contains("{{") {
            let last_open = before.rfind("{{").unwrap();
            let close_before = before[last_open..].contains("}}");
            if !close_before {
                return true;
            }
        }

        // Check for directive expressions: ="
        if let Some(eq_pos) = before.rfind('=') {
            let after_eq = &before[eq_pos..];
            if after_eq.contains('"') && !after_eq[after_eq.find('"').unwrap() + 1..].contains('"')
            {
                return true;
            }
        }

        false
    }

    /// Find identifier references in script content.
    fn find_identifier_references_in_script(content: &str, word: &str) -> Vec<(u32, u32)> {
        let mut refs = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let positions = Self::find_word_occurrences(line, word);

            for pos in positions {
                refs.push((line_idx as u32 + 1, pos as u32));
            }
        }

        refs
    }

    /// Find v-bind references in style content.
    fn find_vbind_references_in_style(content: &str, word: &str) -> Vec<(u32, u32)> {
        let mut refs = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            // Look for v-bind(word) pattern
            if let Some(vbind_pos) = line.find("v-bind(") {
                let after_vbind = &line[vbind_pos + 7..];
                if let Some(close_paren) = after_vbind.find(')') {
                    let binding_name = after_vbind[..close_paren].trim();
                    if binding_name == word {
                        refs.push((
                            line_idx as u32 + 1,
                            (vbind_pos + 7 + (binding_name.len() - binding_name.trim_start().len()))
                                as u32,
                        ));
                    }
                }
            }
        }

        refs
    }

    /// Find a binding definition in script content.
    fn find_binding_in_script(content: &str, name: &str) -> Option<usize> {
        let content_start = Self::skip_virtual_header(content);
        let search_content = &content[content_start..];

        let patterns = [
            format!("const {} ", name),
            format!("const {}=", name),
            format!("let {} ", name),
            format!("let {}=", name),
            format!("var {} ", name),
            format!("var {}=", name),
            format!("function {}(", name),
            format!("function {} (", name),
        ];

        for pattern in &patterns {
            if let Some(pos) = search_content.find(pattern.as_str()) {
                let name_offset = pattern.find(name).unwrap_or(0);
                return Some(content_start + pos + name_offset);
            }
        }

        // Check destructuring
        let destructure_patterns = [
            format!("{{ {}", name),
            format!("{{ {}, ", name),
            format!("{{ {} }}", name),
            format!(", {} }}", name),
            format!(", {}, ", name),
        ];

        for pattern in &destructure_patterns {
            if let Some(pos) = search_content.find(pattern.as_str()) {
                let name_offset = pattern.find(name).unwrap_or(0);
                return Some(content_start + pos + name_offset);
            }
        }

        None
    }

    /// Skip virtual code header.
    fn skip_virtual_header(content: &str) -> usize {
        let mut offset = 0;
        for line in content.lines() {
            if line.starts_with("//") || line.trim().is_empty() {
                offset += line.len() + 1;
            } else {
                break;
            }
        }
        offset
    }

    /// Get the word at an offset.
    fn get_word_at_offset(content: &str, offset: usize) -> Option<String> {
        if offset >= content.len() {
            return None;
        }

        let bytes = content.as_bytes();

        if !Self::is_identifier_char(bytes[offset]) {
            return None;
        }

        let mut start = offset;
        while start > 0 && Self::is_identifier_char(bytes[start - 1]) {
            start -= 1;
        }

        let mut end = offset;
        while end < bytes.len() && Self::is_identifier_char(bytes[end]) {
            end += 1;
        }

        if start == end {
            return None;
        }

        Some(String::from_utf8_lossy(&bytes[start..end]).to_string())
    }

    /// Check if a byte is an identifier character.
    #[inline]
    fn is_identifier_char(c: u8) -> bool {
        c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
    }

    /// Convert offset to (line, character).
    fn offset_to_position(content: &str, offset: usize) -> (u32, u32) {
        let mut line = 0u32;
        let mut col = 0u32;
        let mut current = 0usize;

        for ch in content.chars() {
            if current >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
            current += ch.len_utf8();
        }

        (line, col)
    }

    /// Get script setup start line.
    fn get_script_setup_start_line(content: &str) -> Option<u32> {
        let options = vize_atelier_sfc::SfcParseOptions::default();
        let descriptor = vize_atelier_sfc::parse_sfc(content, options).ok()?;
        descriptor
            .script_setup
            .as_ref()
            .map(|s| s.loc.start_line as u32)
    }

    /// Get script start line.
    fn get_script_start_line(content: &str) -> Option<u32> {
        let options = vize_atelier_sfc::SfcParseOptions::default();
        let descriptor = vize_atelier_sfc::parse_sfc(content, options).ok()?;
        descriptor.script.as_ref().map(|s| s.loc.start_line as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_word_occurrences() {
        let text = "message + message2 + getMessage()";

        let positions = ReferencesService::find_word_occurrences(text, "message");
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0], 0);

        let positions = ReferencesService::find_word_occurrences(text, "message2");
        assert_eq!(positions.len(), 1);
    }

    #[test]
    fn test_find_identifier_references_in_script() {
        let content = r#"
const message = ref('hello')
console.log(message)
const other = message.value
"#;

        let refs = ReferencesService::find_identifier_references_in_script(content, "message");
        assert_eq!(refs.len(), 3);
    }

    #[test]
    fn test_find_vbind_references_in_style() {
        let content = r#"
.container {
  color: v-bind(textColor);
  background: v-bind(bgColor);
}
"#;

        let refs = ReferencesService::find_vbind_references_in_style(content, "textColor");
        assert_eq!(refs.len(), 1);

        let refs = ReferencesService::find_vbind_references_in_style(content, "bgColor");
        assert_eq!(refs.len(), 1);
    }

    #[test]
    fn test_is_in_binding_context() {
        // Inside interpolation
        assert!(ReferencesService::is_in_binding_context("{{ message }}", 3));

        // Inside directive
        assert!(ReferencesService::is_in_binding_context("v-if=\"show\"", 7));

        // Not in binding
        assert!(!ReferencesService::is_in_binding_context(
            "<div>text</div>",
            5
        ));
    }

    #[test]
    fn test_get_word_at_offset() {
        let content = "const message = ref('hello')";

        let word = ReferencesService::get_word_at_offset(content, 6);
        assert_eq!(word, Some("message".to_string()));

        let word = ReferencesService::get_word_at_offset(content, 5);
        assert_eq!(word, None); // space
    }

    #[test]
    fn test_find_binding_in_script() {
        let content = r#"// Virtual TypeScript
// Generated

const message = ref('hello')
function handleClick() {}
"#;

        let loc = ReferencesService::find_binding_in_script(content, "message");
        assert!(loc.is_some());

        let loc = ReferencesService::find_binding_in_script(content, "handleClick");
        assert!(loc.is_some());

        let loc = ReferencesService::find_binding_in_script(content, "notFound");
        assert!(loc.is_none());
    }
}
