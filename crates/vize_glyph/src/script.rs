//! High-performance Script/TypeScript formatting using oxc_codegen.
//!
//! This module provides formatting for JavaScript/TypeScript code
//! using OXC's code generator with custom options for Prettier-like output.
//! Optimized for minimal allocations using arena allocation and byte operations.

use crate::error::FormatError;
use crate::options::FormatOptions;
use memchr::memchr;
use oxc_allocator::Allocator as OxcAllocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::Allocator;

/// Format JavaScript/TypeScript content using oxc_codegen
///
/// Uses arena allocation for efficient memory management.
#[inline]
pub fn format_script_content(
    source: &str,
    options: &FormatOptions,
    _allocator: &Allocator,
) -> Result<String, FormatError> {
    // Fast path for empty content
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    // Use OXC's allocator for parsing (required by oxc_parser)
    let oxc_allocator = OxcAllocator::default();

    // Determine source type (default to TypeScript module)
    let source_type = SourceType::ts().with_module(true);

    // Parse the source
    let parser = Parser::new(&oxc_allocator, source, source_type);
    let parsed = parser.parse();

    if !parsed.errors.is_empty() {
        let error_messages: Vec<String> = parsed.errors.iter().map(|e| e.to_string()).collect();
        return Err(FormatError::ScriptParseError(error_messages.join("; ")));
    }

    // Configure codegen options
    let codegen_options = convert_to_codegen_options(options);

    // Generate formatted code
    let codegen = Codegen::new().with_options(codegen_options);
    let formatted = codegen.build(&parsed.program).code;

    // Post-process the output for better formatting
    let formatted = post_process_script_fast(&formatted, options);

    Ok(formatted)
}

/// Convert our options to oxc_codegen options
#[inline]
fn convert_to_codegen_options(options: &FormatOptions) -> CodegenOptions {
    CodegenOptions {
        single_quote: options.single_quote,
        minify: false,
        comments: true,
        annotation_comments: true,
        source_map_path: None,
        ..Default::default()
    }
}

/// Post-process the generated code for Prettier-like output using byte operations
#[inline]
fn post_process_script_fast(source: &str, options: &FormatOptions) -> String {
    let bytes = source.as_bytes();
    let len = bytes.len();

    // Pre-allocate with estimated size
    let mut result = Vec::with_capacity(len + len / 4);

    let newline = options.newline_bytes();
    let tab_width = options.tab_width as usize;
    let use_tabs = options.use_tabs;

    let mut pos = 0;

    while pos < len {
        // Find start of content (skip leading whitespace on line)
        let mut leading_spaces = 0;

        // Count leading spaces/tabs
        while pos < len {
            match bytes[pos] {
                b' ' => {
                    leading_spaces += 1;
                    pos += 1;
                }
                b'\t' => {
                    leading_spaces += tab_width;
                    pos += 1;
                }
                _ => break,
            }
        }

        // Find end of line using memchr (SIMD-accelerated)
        let line_end = if let Some(newline_pos) = memchr(b'\n', &bytes[pos..]) {
            pos + newline_pos
        } else {
            len
        };

        // Calculate proper indentation
        let indent_level = leading_spaces / 2; // OXC uses 1 space per level, we normalize

        // Write indentation
        if use_tabs {
            result.extend(std::iter::repeat_n(b'\t', indent_level));
        } else {
            result.extend(std::iter::repeat_n(b' ', indent_level * tab_width));
        }

        // Write content (trimmed)
        let content_end = if line_end > 0 && bytes[line_end - 1] == b'\r' {
            line_end - 1
        } else {
            line_end
        };

        if pos < content_end {
            result.extend_from_slice(&bytes[pos..content_end]);
        }

        result.extend_from_slice(newline);

        // Move to next line
        pos = if line_end < len { line_end + 1 } else { len };
    }

    // Handle semicolons if semi is false
    let result = if !options.semi {
        remove_optional_semicolons_fast(&result, newline)
    } else {
        result
    };

    // Ensure bracket spacing
    let result = if options.bracket_spacing {
        ensure_bracket_spacing_fast(&result)
    } else {
        result
    };

    // Trim trailing whitespace and ensure single newline
    let mut final_result = trim_trailing_whitespace(&result);
    final_result.extend_from_slice(newline);

    // SAFETY: We only processed valid UTF-8 input
    unsafe { String::from_utf8_unchecked(final_result) }
}

/// Remove optional semicolons using byte operations
#[inline]
fn remove_optional_semicolons_fast(source: &[u8], _newline: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(source.len());
    let mut i = 0;
    let len = source.len();

    while i < len {
        let b = source[i];

        // Skip string literals
        if b == b'"' || b == b'\'' || b == b'`' {
            let quote = b;
            result.push(b);
            i += 1;

            while i < len {
                let c = source[i];
                result.push(c);
                i += 1;

                if c == quote && (i < 2 || source[i - 2] != b'\\') {
                    break;
                }
            }
            continue;
        }

        // Check for semicolon at end of line
        if b == b';' {
            let next_idx = i + 1;
            if next_idx >= len
                || source[next_idx] == b'\n'
                || (next_idx + 1 < len
                    && source[next_idx] == b'\r'
                    && source[next_idx + 1] == b'\n')
            {
                // Skip semicolon at end of line
                i += 1;
                continue;
            }
        }

        result.push(b);
        i += 1;
    }

    result
}

/// Ensure proper spacing in brackets using byte operations
#[inline]
fn ensure_bracket_spacing_fast(source: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(source.len() + source.len() / 10);
    let mut i = 0;
    let len = source.len();

    while i < len {
        let b = source[i];

        // Skip string literals
        if b == b'"' || b == b'\'' || b == b'`' {
            let quote = b;
            result.push(b);
            i += 1;

            while i < len {
                let c = source[i];
                result.push(c);
                i += 1;

                if c == quote && (i < 2 || source[i - 2] != b'\\') {
                    break;
                }
            }
            continue;
        }

        // Handle opening brace
        if b == b'{' {
            result.push(b);
            i += 1;

            // Add space after { if not followed by }, space, or newline
            if i < len {
                let next = source[i];
                if next != b'}' && next != b' ' && next != b'\n' && next != b'\r' {
                    result.push(b' ');
                }
            }
            continue;
        }

        // Handle closing brace
        if b == b'}' {
            // Add space before } if not preceded by {, space, or newline
            if !result.is_empty() {
                let last = *result.last().unwrap();
                if last != b'{' && last != b' ' && last != b'\n' && last != b'\r' {
                    result.push(b' ');
                }
            }
            result.push(b);
            i += 1;
            continue;
        }

        result.push(b);
        i += 1;
    }

    result
}

/// Trim trailing whitespace from each line
#[inline]
fn trim_trailing_whitespace(source: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(source.len());
    let mut line_start = 0;

    for (i, &b) in source.iter().enumerate() {
        if b == b'\n' {
            // Find last non-whitespace character in line
            let mut line_end = i;
            while line_end > line_start {
                let c = source[line_end - 1];
                if c != b' ' && c != b'\t' && c != b'\r' {
                    break;
                }
                line_end -= 1;
            }

            result.extend_from_slice(&source[line_start..line_end]);
            result.push(b'\n');
            line_start = i + 1;
        }
    }

    // Handle last line without newline
    if line_start < source.len() {
        let mut line_end = source.len();
        while line_end > line_start {
            let c = source[line_end - 1];
            if c != b' ' && c != b'\t' && c != b'\r' && c != b'\n' {
                break;
            }
            line_end -= 1;
        }
        result.extend_from_slice(&source[line_start..line_end]);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_script() {
        let source = "const x=1";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content(source, &options, &allocator).unwrap();

        assert!(result.contains("const x = 1"));
    }

    #[test]
    fn test_format_with_imports() {
        let source = "import {ref,computed} from 'vue'";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content(source, &options, &allocator).unwrap();

        assert!(result.contains("ref"));
        assert!(result.contains("computed"));
        assert!(result.contains("vue"));
    }

    #[test]
    fn test_format_object() {
        let source = "const obj={a:1,b:2}";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content(source, &options, &allocator).unwrap();

        assert!(result.contains("a:"));
        assert!(result.contains("b:"));
    }

    #[test]
    fn test_format_empty_source() {
        let source = "";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content(source, &options, &allocator).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_format_whitespace_only() {
        let source = "   \n\t  ";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content(source, &options, &allocator).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_bracket_spacing() {
        let input = b"const x = {a: 1}";
        let result = ensure_bracket_spacing_fast(input);
        let result_str = std::str::from_utf8(&result).unwrap();
        assert!(result_str.contains("{ a: 1 }"));
    }
}
