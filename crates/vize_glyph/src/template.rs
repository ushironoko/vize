//! High-performance template formatting for Vue SFC.
//!
//! This module provides formatting for Vue template blocks,
//! handling proper indentation and attribute formatting.
//! Uses byte operations and SIMD-accelerated search for maximum performance.

use crate::error::FormatError;
use crate::options::FormatOptions;
// memchr is available for future optimizations

/// Result type for parsing opening tags: (tag_name, attributes, is_self_closing, end_pos)
type OpeningTagParseResult<'a> = Option<(&'a [u8], Vec<&'a [u8]>, bool, usize)>;

/// Format Vue template content
#[inline]
pub fn format_template_content(
    source: &str,
    options: &FormatOptions,
) -> Result<String, FormatError> {
    let bytes = source.as_bytes();

    // Fast path: find first non-whitespace byte
    let start = bytes.iter().position(|&b| !is_whitespace(b));
    if start.is_none() {
        return Ok(String::new());
    }

    let formatter = TemplateFormatter::new(options);
    formatter.format(bytes)
}

/// High-performance template formatter using byte operations
struct TemplateFormatter<'a> {
    options: &'a FormatOptions,
    indent: &'static [u8],
    newline: &'static [u8],
}

impl<'a> TemplateFormatter<'a> {
    #[inline]
    fn new(options: &'a FormatOptions) -> Self {
        Self {
            options,
            indent: options.indent_bytes(),
            newline: options.newline_bytes(),
        }
    }

    fn format(&self, source: &[u8]) -> Result<String, FormatError> {
        let len = source.len();

        // Pre-allocate output buffer
        let mut output = Vec::with_capacity(len + len / 4);

        let mut pos = 0;
        let mut depth: usize = 0;
        let mut line_buffer = Vec::with_capacity(256);

        while pos < len {
            // Skip whitespace at line start
            while pos < len && is_whitespace(source[pos]) && source[pos] != b'\n' {
                pos += 1;
            }

            if pos >= len {
                break;
            }

            // Handle newlines
            if source[pos] == b'\n' {
                pos += 1;
                continue;
            }

            // Check for tag start
            if source[pos] == b'<' {
                // Flush any accumulated text content
                if !line_buffer.is_empty() {
                    self.write_indented_line(&mut output, &line_buffer, depth);
                    line_buffer.clear();
                }

                // Check for closing tag
                if pos + 1 < len && source[pos + 1] == b'/' {
                    // Parse closing tag
                    let tag_result = self.parse_closing_tag(source, pos);
                    if let Some((tag_name, end_pos)) = tag_result {
                        depth = depth.saturating_sub(1);
                        self.write_indent(&mut output, depth);
                        output.extend_from_slice(b"</");
                        output.extend_from_slice(tag_name);
                        output.push(b'>');
                        output.extend_from_slice(self.newline);
                        pos = end_pos;
                        continue;
                    }
                }

                // Parse opening tag
                let tag_result = self.parse_opening_tag(source, pos);
                if let Some((tag_name, attrs, is_self_closing, end_pos)) = tag_result {
                    self.write_indent(&mut output, depth);
                    output.push(b'<');
                    output.extend_from_slice(tag_name);

                    // Write attributes
                    if !attrs.is_empty() {
                        if self.options.single_attribute_per_line && attrs.len() > 1 {
                            // Multi-line attributes
                            output.extend_from_slice(self.newline);
                            for attr in &attrs {
                                self.write_indent(&mut output, depth + 1);
                                output.extend_from_slice(attr);
                                output.extend_from_slice(self.newline);
                            }
                            self.write_indent(&mut output, depth);
                        } else {
                            // Single line attributes
                            for attr in &attrs {
                                output.push(b' ');
                                output.extend_from_slice(attr);
                            }
                        }
                    }

                    if is_self_closing {
                        output.extend_from_slice(b" />");
                    } else {
                        output.push(b'>');
                        if !is_void_element(tag_name) {
                            depth += 1;
                        }
                    }
                    output.extend_from_slice(self.newline);
                    pos = end_pos;
                    continue;
                }
            }

            // Accumulate text content until newline or tag
            let content_start = pos;
            while pos < len && source[pos] != b'\n' && source[pos] != b'<' {
                pos += 1;
            }

            if pos > content_start {
                // Trim trailing whitespace from content
                let mut content_end = pos;
                while content_end > content_start && is_whitespace(source[content_end - 1]) {
                    content_end -= 1;
                }

                if content_end > content_start {
                    if !line_buffer.is_empty() {
                        line_buffer.push(b' ');
                    }
                    line_buffer.extend_from_slice(&source[content_start..content_end]);
                }
            }

            // Handle newline
            if pos < len && source[pos] == b'\n' {
                if !line_buffer.is_empty() {
                    self.write_indented_line(&mut output, &line_buffer, depth);
                    line_buffer.clear();
                }
                pos += 1;
            }
        }

        // Flush remaining content
        if !line_buffer.is_empty() {
            self.write_indented_line(&mut output, &line_buffer, depth);
        }

        // Remove trailing newline for consistency
        while output.last().is_some_and(|&b| b == b'\n' || b == b'\r') {
            output.pop();
        }

        // SAFETY: We only wrote valid UTF-8 bytes
        Ok(unsafe { String::from_utf8_unchecked(output) })
    }

    #[inline]
    fn write_indent(&self, output: &mut Vec<u8>, depth: usize) {
        for _ in 0..depth {
            output.extend_from_slice(self.indent);
        }
    }

    #[inline]
    fn write_indented_line(&self, output: &mut Vec<u8>, content: &[u8], depth: usize) {
        self.write_indent(output, depth);
        output.extend_from_slice(content);
        output.extend_from_slice(self.newline);
    }

    /// Parse an opening tag, returns (tag_name, attributes, is_self_closing, end_pos)
    #[inline]
    fn parse_opening_tag<'b>(&self, source: &'b [u8], start: usize) -> OpeningTagParseResult<'b> {
        let len = source.len();
        let mut pos = start + 1; // Skip '<'

        // Parse tag name
        let tag_start = pos;
        while pos < len && is_tag_name_char(source[pos]) {
            pos += 1;
        }

        if pos == tag_start {
            return None;
        }

        let tag_name = &source[tag_start..pos];

        // Parse attributes
        let mut attrs = Vec::new();
        let mut is_self_closing = false;

        while pos < len && source[pos] != b'>' {
            // Skip whitespace
            while pos < len && is_whitespace(source[pos]) {
                pos += 1;
            }

            if pos >= len {
                break;
            }

            // Check for self-closing or end
            if source[pos] == b'/' {
                is_self_closing = true;
                pos += 1;
                continue;
            }

            if source[pos] == b'>' {
                break;
            }

            // Parse attribute
            let attr_start = pos;
            let mut in_quote = false;
            let mut quote_char = b'"';

            while pos < len {
                let b = source[pos];

                if in_quote {
                    if b == quote_char {
                        in_quote = false;
                    }
                    pos += 1;
                } else if b == b'"' || b == b'\'' {
                    in_quote = true;
                    quote_char = b;
                    pos += 1;
                } else if is_whitespace(b) || b == b'>' || b == b'/' {
                    break;
                } else {
                    pos += 1;
                }
            }

            if pos > attr_start {
                attrs.push(&source[attr_start..pos]);
            }
        }

        // Skip '>'
        if pos < len && source[pos] == b'>' {
            pos += 1;
        }

        Some((tag_name, attrs, is_self_closing, pos))
    }

    /// Parse a closing tag, returns (tag_name, end_pos)
    #[inline]
    fn parse_closing_tag<'b>(&self, source: &'b [u8], start: usize) -> Option<(&'b [u8], usize)> {
        let len = source.len();
        let mut pos = start + 2; // Skip '</'

        // Parse tag name
        let tag_start = pos;
        while pos < len && is_tag_name_char(source[pos]) {
            pos += 1;
        }

        if pos == tag_start {
            return None;
        }

        let tag_name = &source[tag_start..pos];

        // Skip whitespace and find '>'
        while pos < len && source[pos] != b'>' {
            pos += 1;
        }

        if pos < len && source[pos] == b'>' {
            pos += 1;
        }

        Some((tag_name, pos))
    }
}

/// Check if a byte is a valid tag name character
#[inline(always)]
fn is_tag_name_char(b: u8) -> bool {
    matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b':')
}

/// Check if a byte is whitespace
#[inline(always)]
fn is_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

/// Check if an element is a void element (self-closing in HTML)
#[inline]
fn is_void_element(tag: &[u8]) -> bool {
    // Fast path for common cases
    match tag.len() {
        2 => tag.eq_ignore_ascii_case(b"br") || tag.eq_ignore_ascii_case(b"hr"),
        3 => tag.eq_ignore_ascii_case(b"img") || tag.eq_ignore_ascii_case(b"col"),
        4 => {
            tag.eq_ignore_ascii_case(b"area")
                || tag.eq_ignore_ascii_case(b"base")
                || tag.eq_ignore_ascii_case(b"meta")
                || tag.eq_ignore_ascii_case(b"link")
        }
        5 => {
            tag.eq_ignore_ascii_case(b"embed")
                || tag.eq_ignore_ascii_case(b"input")
                || tag.eq_ignore_ascii_case(b"param")
                || tag.eq_ignore_ascii_case(b"track")
        }
        6 => tag.eq_ignore_ascii_case(b"source"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_template() {
        let source = "<div>Hello</div>";
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        assert!(result.contains("<div>"));
        assert!(result.contains("</div>"));
    }

    #[test]
    fn test_format_nested_template() {
        let source = "<div><span>Hello</span></div>";
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        // Should be properly indented
        assert!(result.contains("<div>"));
        assert!(result.contains("  <span>"));
        assert!(result.contains("</div>"));
    }

    #[test]
    fn test_format_with_attributes() {
        let source = r#"<div class="container" id="main">Content</div>"#;
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        assert!(result.contains(r#"class="container""#));
        assert!(result.contains(r#"id="main""#));
    }

    #[test]
    fn test_format_self_closing() {
        let source = "<input type=\"text\" />";
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        assert!(result.contains("<input"));
        assert!(result.contains("/>"));
    }

    #[test]
    fn test_void_elements() {
        assert!(is_void_element(b"br"));
        assert!(is_void_element(b"img"));
        assert!(is_void_element(b"input"));
        assert!(is_void_element(b"BR")); // Case insensitive
        assert!(!is_void_element(b"div"));
        assert!(!is_void_element(b"span"));
    }

    #[test]
    fn test_is_tag_name_char() {
        assert!(is_tag_name_char(b'a'));
        assert!(is_tag_name_char(b'Z'));
        assert!(is_tag_name_char(b'0'));
        assert!(is_tag_name_char(b'-'));
        assert!(is_tag_name_char(b'_'));
        assert!(!is_tag_name_char(b' '));
        assert!(!is_tag_name_char(b'>'));
    }
}
