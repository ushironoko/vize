//! Definition provider for Vue SFC files.
//!
//! Provides go-to-definition for:
//! - Template expressions -> script bindings
//! - Component usages -> component definitions
//! - Import statements -> imported files

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range};

use super::IdeContext;
use crate::virtual_code::BlockType;

/// Definition service for providing go-to-definition functionality.
pub struct DefinitionService;

impl DefinitionService {
    /// Get definition for the symbol at the current position.
    pub fn definition(ctx: &IdeContext) -> Option<GotoDefinitionResponse> {
        match ctx.block_type? {
            BlockType::Template => Self::definition_in_template(ctx),
            BlockType::Script | BlockType::ScriptSetup => Self::definition_in_script(ctx),
            BlockType::Style(_) => Self::definition_in_style(ctx),
        }
    }

    /// Find definition for a symbol in template context.
    fn definition_in_template(ctx: &IdeContext) -> Option<GotoDefinitionResponse> {
        // Get the word at the cursor position
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset)?;

        if word.is_empty() {
            return None;
        }

        // Try to find the binding in script setup
        if let Some(ref virtual_docs) = ctx.virtual_docs {
            if let Some(ref script_setup) = virtual_docs.script_setup {
                // Find binding location in script setup
                if let Some(binding_loc) =
                    Self::find_binding_location(&script_setup.content, &word, true)
                {
                    // Calculate the actual position in the SFC file
                    let (line, character) =
                        Self::offset_to_position(&script_setup.content, binding_loc.offset);

                    // Adjust line based on script block position in SFC
                    // We need to get the actual script block start line
                    let sfc_line =
                        Self::get_script_setup_start_line(&ctx.content).unwrap_or(0) + line;

                    return Some(GotoDefinitionResponse::Scalar(Location {
                        uri: ctx.uri.clone(),
                        range: Range {
                            start: Position {
                                line: sfc_line,
                                character,
                            },
                            end: Position {
                                line: sfc_line,
                                character: character + word.len() as u32,
                            },
                        },
                    }));
                }
            }

            // Try regular script block
            if let Some(ref script) = virtual_docs.script {
                if let Some(binding_loc) =
                    Self::find_binding_location(&script.content, &word, false)
                {
                    let (line, character) =
                        Self::offset_to_position(&script.content, binding_loc.offset);

                    let sfc_line = Self::get_script_start_line(&ctx.content).unwrap_or(0) + line;

                    return Some(GotoDefinitionResponse::Scalar(Location {
                        uri: ctx.uri.clone(),
                        range: Range {
                            start: Position {
                                line: sfc_line,
                                character,
                            },
                            end: Position {
                                line: sfc_line,
                                character: character + word.len() as u32,
                            },
                        },
                    }));
                }
            }
        }

        None
    }

    /// Find definition for a symbol in script context.
    fn definition_in_script(ctx: &IdeContext) -> Option<GotoDefinitionResponse> {
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset)?;

        if word.is_empty() {
            return None;
        }

        // Check for import statement - could be a component or module
        // For now, we'll handle simple cases

        // Look for local definitions in the same script block
        let script_content = Self::get_current_script_content(ctx)?;

        if let Some(binding_loc) = Self::find_binding_location(&script_content, &word, true) {
            let (line, character) = Self::offset_to_position(&script_content, binding_loc.offset);

            // Get the script block start line
            let sfc_line = match ctx.block_type {
                Some(BlockType::ScriptSetup) => {
                    Self::get_script_setup_start_line(&ctx.content).unwrap_or(0)
                }
                Some(BlockType::Script) => Self::get_script_start_line(&ctx.content).unwrap_or(0),
                _ => 0,
            } + line;

            return Some(GotoDefinitionResponse::Scalar(Location {
                uri: ctx.uri.clone(),
                range: Range {
                    start: Position {
                        line: sfc_line,
                        character,
                    },
                    end: Position {
                        line: sfc_line,
                        character: character + word.len() as u32,
                    },
                },
            }));
        }

        None
    }

    /// Find definition for a symbol in style context.
    fn definition_in_style(ctx: &IdeContext) -> Option<GotoDefinitionResponse> {
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset)?;

        if word.is_empty() {
            return None;
        }

        // Check for v-bind() references to script variables
        // Look backwards to see if we're inside v-bind()
        let before_cursor = &ctx.content[..ctx.offset];
        if before_cursor.contains("v-bind(") {
            // Try to find the binding in script setup
            if let Some(ref virtual_docs) = ctx.virtual_docs {
                if let Some(ref script_setup) = virtual_docs.script_setup {
                    if let Some(binding_loc) =
                        Self::find_binding_location(&script_setup.content, &word, true)
                    {
                        let (line, character) =
                            Self::offset_to_position(&script_setup.content, binding_loc.offset);

                        let sfc_line =
                            Self::get_script_setup_start_line(&ctx.content).unwrap_or(0) + line;

                        return Some(GotoDefinitionResponse::Scalar(Location {
                            uri: ctx.uri.clone(),
                            range: Range {
                                start: Position {
                                    line: sfc_line,
                                    character,
                                },
                                end: Position {
                                    line: sfc_line,
                                    character: character + word.len() as u32,
                                },
                            },
                        }));
                    }
                }
            }
        }

        None
    }

    /// Get the current script block content based on context.
    fn get_current_script_content(ctx: &IdeContext) -> Option<String> {
        if let Some(ref virtual_docs) = ctx.virtual_docs {
            match ctx.block_type {
                Some(BlockType::ScriptSetup) => virtual_docs
                    .script_setup
                    .as_ref()
                    .map(|d| d.content.clone()),
                Some(BlockType::Script) => virtual_docs.script.as_ref().map(|d| d.content.clone()),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get the word at a given offset.
    fn get_word_at_offset(content: &str, offset: usize) -> Option<String> {
        if offset >= content.len() {
            return None;
        }

        let bytes = content.as_bytes();

        // If the character at offset is not a word character, return None
        if !Self::is_word_char(bytes[offset]) {
            return None;
        }

        // Find word start
        let mut start = offset;
        while start > 0 {
            let c = bytes[start - 1];
            if !Self::is_word_char(c) {
                break;
            }
            start -= 1;
        }

        // Find word end
        let mut end = offset;
        while end < bytes.len() {
            let c = bytes[end];
            if !Self::is_word_char(c) {
                break;
            }
            end += 1;
        }

        if start == end {
            return None;
        }

        Some(String::from_utf8_lossy(&bytes[start..end]).to_string())
    }

    /// Check if a byte is a valid word character.
    #[inline]
    fn is_word_char(c: u8) -> bool {
        c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
    }

    /// Find the location of a binding definition in script content.
    fn find_binding_location(
        content: &str,
        name: &str,
        _is_setup: bool,
    ) -> Option<BindingLocation> {
        // Skip the header comments in virtual code
        let content_start = Self::skip_virtual_header(content);
        let search_content = &content[content_start..];

        // Search patterns for binding definitions
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
                // Find the actual name position within the pattern
                let name_offset = pattern.find(name).unwrap_or(0);
                let actual_offset = content_start + pos + name_offset;

                return Some(BindingLocation {
                    name: name.to_string(),
                    offset: actual_offset,
                    kind: BindingKind::from_pattern(pattern),
                });
            }
        }

        // Check for destructuring patterns: const { name } = ...
        let destructure_pattern = format!("{{ {}", name);
        if let Some(pos) = search_content.find(destructure_pattern.as_str()) {
            let name_offset = destructure_pattern.find(name).unwrap_or(0);
            let actual_offset = content_start + pos + name_offset;

            return Some(BindingLocation {
                name: name.to_string(),
                offset: actual_offset,
                kind: BindingKind::Destructure,
            });
        }

        // Check for: { name, ... } pattern with possible whitespace
        let destructure_patterns = [
            format!("{{ {}, ", name),
            format!("{{ {} }}", name),
            format!(", {} }}", name),
            format!(", {}, ", name),
        ];

        for pattern in &destructure_patterns {
            if let Some(pos) = search_content.find(pattern.as_str()) {
                let name_offset = pattern.find(name).unwrap_or(0);
                let actual_offset = content_start + pos + name_offset;

                return Some(BindingLocation {
                    name: name.to_string(),
                    offset: actual_offset,
                    kind: BindingKind::Destructure,
                });
            }
        }

        None
    }

    /// Skip virtual code header comments.
    fn skip_virtual_header(content: &str) -> usize {
        let mut offset = 0;
        for line in content.lines() {
            if line.starts_with("//") || line.trim().is_empty() {
                offset += line.len() + 1; // +1 for newline
            } else {
                break;
            }
        }
        offset
    }

    /// Convert byte offset to (line, character) position.
    fn offset_to_position(content: &str, offset: usize) -> (u32, u32) {
        let mut line = 0u32;
        let mut col = 0u32;
        let mut current_offset = 0usize;

        for ch in content.chars() {
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

    /// Get the start line of <script setup> block in SFC.
    fn get_script_setup_start_line(content: &str) -> Option<u32> {
        let options = vize_atelier_sfc::SfcParseOptions::default();
        let descriptor = vize_atelier_sfc::parse_sfc(content, options).ok()?;
        descriptor
            .script_setup
            .as_ref()
            .map(|s| s.loc.start_line as u32)
    }

    /// Get the start line of <script> block in SFC.
    fn get_script_start_line(content: &str) -> Option<u32> {
        let options = vize_atelier_sfc::SfcParseOptions::default();
        let descriptor = vize_atelier_sfc::parse_sfc(content, options).ok()?;
        descriptor.script.as_ref().map(|s| s.loc.start_line as u32)
    }
}

/// Location of a binding definition.
#[derive(Debug, Clone)]
pub struct BindingLocation {
    /// The binding name.
    pub name: String,
    /// Byte offset in the content.
    pub offset: usize,
    /// Kind of binding.
    pub kind: BindingKind,
}

/// Kind of binding definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    /// const declaration
    Const,
    /// let declaration
    Let,
    /// var declaration
    Var,
    /// function declaration
    Function,
    /// Destructuring pattern
    Destructure,
    /// Import binding
    Import,
    /// Unknown
    Unknown,
}

impl BindingKind {
    fn from_pattern(pattern: &str) -> Self {
        if pattern.starts_with("const") {
            BindingKind::Const
        } else if pattern.starts_with("let") {
            BindingKind::Let
        } else if pattern.starts_with("var") {
            BindingKind::Var
        } else if pattern.starts_with("function") {
            BindingKind::Function
        } else {
            BindingKind::Unknown
        }
    }
}

/// Extract bindings with their locations from script content.
pub fn extract_bindings_with_locations(content: &str, is_setup: bool) -> Vec<BindingLocation> {
    let mut bindings = Vec::new();

    if !is_setup {
        return bindings;
    }

    let content_start = DefinitionService::skip_virtual_header(content);
    let search_content = &content[content_start..];

    for line in search_content.lines() {
        let trimmed = line.trim();
        let line_start = search_content[..search_content.find(line).unwrap_or(0)].len();

        // const/let/var declarations
        for keyword in &["const ", "let ", "var "] {
            if trimmed.starts_with(keyword) {
                if let Some(rest) = trimmed.strip_prefix(keyword) {
                    // Handle destructuring: { a, b }
                    if rest.starts_with('{') {
                        if let Some(end) = rest.find('}') {
                            let inner = &rest[1..end];
                            for part in inner.split(',') {
                                let name = part.split(':').next().unwrap_or("").trim();
                                if !name.is_empty() && is_valid_identifier(name) {
                                    if let Some(name_pos) = line.find(name) {
                                        bindings.push(BindingLocation {
                                            name: name.to_string(),
                                            offset: content_start + line_start + name_pos,
                                            kind: BindingKind::Destructure,
                                        });
                                    }
                                }
                            }
                        }
                    }
                    // Simple: const x = ...
                    else if let Some(name) = rest.split(['=', ':', ' ']).next() {
                        let name = name.trim();
                        if is_valid_identifier(name) {
                            if let Some(name_pos) = line.find(name) {
                                let kind = match *keyword {
                                    "const " => BindingKind::Const,
                                    "let " => BindingKind::Let,
                                    "var " => BindingKind::Var,
                                    _ => BindingKind::Unknown,
                                };
                                bindings.push(BindingLocation {
                                    name: name.to_string(),
                                    offset: content_start + line_start + name_pos,
                                    kind,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Function declarations
        if trimmed.starts_with("function ") {
            if let Some(rest) = trimmed.strip_prefix("function ") {
                if let Some(name) = rest.split('(').next() {
                    let name = name.trim();
                    if is_valid_identifier(name) {
                        if let Some(name_pos) = line.find(name) {
                            bindings.push(BindingLocation {
                                name: name.to_string(),
                                offset: content_start + line_start + name_pos,
                                kind: BindingKind::Function,
                            });
                        }
                    }
                }
            }
        }
    }

    bindings
}

/// Check if a string is a valid JavaScript identifier.
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_alphabetic() && first != '_' && first != '$' {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_binding_location_const() {
        let content = r#"// Virtual TypeScript
// Generated

const message = ref('hello')
const count = ref(0)
"#;

        let loc = DefinitionService::find_binding_location(content, "message", true);
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.name, "message");
        assert_eq!(loc.kind, BindingKind::Const);
    }

    #[test]
    fn test_find_binding_location_function() {
        let content = r#"// Virtual TypeScript
// Generated

function handleClick() {
  console.log('clicked')
}
"#;

        let loc = DefinitionService::find_binding_location(content, "handleClick", true);
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.name, "handleClick");
        assert_eq!(loc.kind, BindingKind::Function);
    }

    #[test]
    fn test_find_binding_location_destructure() {
        let content = r#"// Virtual TypeScript
// Generated

const { data, error } = useFetch('/api')
"#;

        let loc = DefinitionService::find_binding_location(content, "data", true);
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.name, "data");
        assert_eq!(loc.kind, BindingKind::Destructure);
    }

    #[test]
    fn test_offset_to_position() {
        let content = "line1\nline2\nline3";

        // Start of line1
        let (line, col) = DefinitionService::offset_to_position(content, 0);
        assert_eq!(line, 0);
        assert_eq!(col, 0);

        // Middle of line1
        let (line, col) = DefinitionService::offset_to_position(content, 3);
        assert_eq!(line, 0);
        assert_eq!(col, 3);

        // Start of line2
        let (line, col) = DefinitionService::offset_to_position(content, 6);
        assert_eq!(line, 1);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_get_word_at_offset() {
        let content = "const message = 'hello'";

        let word = DefinitionService::get_word_at_offset(content, 6);
        assert_eq!(word, Some("message".to_string()));

        let word = DefinitionService::get_word_at_offset(content, 5);
        assert_eq!(word, None); // space

        let word = DefinitionService::get_word_at_offset(content, 0);
        assert_eq!(word, Some("const".to_string()));
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("foo"));
        assert!(is_valid_identifier("_foo"));
        assert!(is_valid_identifier("$foo"));
        assert!(is_valid_identifier("foo123"));
        assert!(!is_valid_identifier("123foo"));
        assert!(!is_valid_identifier(""));
    }
}
