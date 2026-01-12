//! css/prefer-nested-selectors
//!
//! Recommend using CSS nesting for descendant selectors.
//!
//! CSS nesting allows writing more maintainable and readable styles
//! by nesting child selectors inside parent selectors.
//!
//! ## Examples
//!
//! Before:
//! ```css
//! .parent .child { color: red; }
//! ```
//!
//! After:
//! ```css
//! .parent {
//!   .child { color: red; }
//! }
//! ```

use lightningcss::stylesheet::StyleSheet;

use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};

use super::{CssLintResult, CssRule, CssRuleMeta};

static META: CssRuleMeta = CssRuleMeta {
    name: "css/prefer-nested-selectors",
    description: "Recommend using CSS nesting for descendant selectors",
    default_severity: Severity::Warning,
};

/// Prefer nested selectors rule
pub struct PreferNestedSelectors;

impl CssRule for PreferNestedSelectors {
    fn meta(&self) -> &'static CssRuleMeta {
        &META
    }

    fn check<'i>(
        &self,
        source: &'i str,
        _stylesheet: &StyleSheet<'i, 'i>,
        offset: usize,
        result: &mut CssLintResult,
    ) {
        // Use pattern matching to find descendant selectors
        // Pattern: ".class .child" or "element child" with space separator
        let bytes = source.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            // Find a selector start (., #, or letter for element)
            if let Some(selector_start) = find_selector_start(bytes, i) {
                // Find the selector end (before {)
                if let Some(brace_pos) = find_next_brace(bytes, selector_start) {
                    let selector = &source[selector_start..brace_pos];
                    let trimmed = selector.trim();

                    // Check if this is a descendant selector (has space but not inside [])
                    if is_descendant_selector(trimmed) {
                        // Find the split point (space outside brackets)
                        if let Some((parent, child)) = split_descendant_selector(trimmed) {
                            let start = (offset + selector_start) as u32;
                            let end = (offset + brace_pos) as u32;

                            // Build fix - find the full rule
                            let rule_end = find_closing_brace(bytes, brace_pos);
                            let declarations = if rule_end > brace_pos + 1 {
                                source[brace_pos + 1..rule_end].trim()
                            } else {
                                ""
                            };

                            let nested_css =
                                format!("{} {{\n  {} {{ {} }}\n}}", parent, child, declarations);

                            let fix = Fix::new(
                                format!("Convert to nested selector under '{}'", parent),
                                TextEdit::replace(
                                    start,
                                    (offset + rule_end + 1) as u32,
                                    nested_css,
                                ),
                            );

                            result.add_diagnostic(
                                LintDiagnostic::warn(
                                    META.name,
                                    format!(
                                        "Consider using CSS nesting for '{}' descendant selectors",
                                        parent
                                    ),
                                    start,
                                    end,
                                )
                                .with_help(format!(
                                    "Use CSS nesting: `{} {{ {} {{ ... }} }}`",
                                    parent, child
                                ))
                                .with_fix(fix),
                            );
                        }
                    }
                    i = brace_pos + 1;
                } else {
                    i += 1;
                }
            } else {
                break;
            }
        }
    }
}

/// Find the start of a selector
#[inline]
fn find_selector_start(bytes: &[u8], start: usize) -> Option<usize> {
    for (offset, &byte) in bytes[start..].iter().enumerate() {
        match byte {
            b'.' | b'#' => return Some(start + offset),
            b'a'..=b'z' | b'A'..=b'Z' => {
                // Check it's not inside a comment or string
                return Some(start + offset);
            }
            b' ' | b'\n' | b'\r' | b'\t' | b'}' => continue,
            _ => continue,
        }
    }
    None
}

/// Find the next opening brace
#[inline]
fn find_next_brace(bytes: &[u8], start: usize) -> Option<usize> {
    for (offset, &byte) in bytes[start..].iter().enumerate() {
        if byte == b'{' {
            return Some(start + offset);
        }
        // Stop at @ rules or }
        if byte == b'@' || byte == b'}' {
            return None;
        }
    }
    None
}

/// Find the closing brace for a rule
#[inline]
fn find_closing_brace(bytes: &[u8], open_pos: usize) -> usize {
    let mut depth = 1;
    for (offset, &byte) in bytes[open_pos + 1..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return open_pos + 1 + offset;
                }
            }
            _ => {}
        }
    }
    bytes.len()
}

/// Check if a selector is a descendant selector
#[inline]
fn is_descendant_selector(selector: &str) -> bool {
    let bytes = selector.as_bytes();
    let mut bracket_depth: usize = 0;
    let mut paren_depth: usize = 0;
    let mut in_quote = false;
    let mut quote_char: u8 = 0;

    for &b in bytes {
        // Handle quotes
        if !in_quote && (b == b'"' || b == b'\'') {
            in_quote = true;
            quote_char = b;
            continue;
        }
        if in_quote && b == quote_char {
            in_quote = false;
            continue;
        }
        if in_quote {
            continue;
        }

        match b {
            b'[' => bracket_depth += 1,
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b'(' => paren_depth += 1,
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b' ' if bracket_depth == 0 && paren_depth == 0 => {
                // Found a space outside brackets/parens - this is a descendant selector
                return true;
            }
            b'>' | b'+' | b'~' if bracket_depth == 0 && paren_depth == 0 => {
                // Also handle child, adjacent, and sibling combinators
                return true;
            }
            _ => {}
        }
    }
    false
}

/// Split a descendant selector into parent and child parts
#[inline]
fn split_descendant_selector(selector: &str) -> Option<(&str, &str)> {
    let bytes = selector.as_bytes();
    let mut bracket_depth: usize = 0;
    let mut paren_depth: usize = 0;

    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'[' => bracket_depth += 1,
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b'(' => paren_depth += 1,
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b' ' | b'>' | b'+' | b'~' if bracket_depth == 0 && paren_depth == 0 => {
                let parent = selector[..i].trim();
                let child = selector[i..]
                    .trim()
                    .trim_start_matches([' ', '>', '+', '~'])
                    .trim();
                if !parent.is_empty() && !child.is_empty() {
                    return Some((parent, child));
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::css::CssLinter;

    fn create_linter() -> CssLinter {
        let mut linter = CssLinter::new();
        linter.add_rule(Box::new(PreferNestedSelectors));
        linter
    }

    #[test]
    fn test_simple_selector() {
        let linter = create_linter();
        let result = linter.lint(".button { color: red; }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_descendant_selector() {
        let linter = create_linter();
        let result = linter.lint(".parent .child { color: red; }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_child_selector() {
        let linter = create_linter();
        let result = linter.lint(".parent > .child { color: red; }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_element_descendant() {
        let linter = create_linter();
        let result = linter.lint("div span { color: red; }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_has_fix() {
        let linter = create_linter();
        let result = linter.lint(".parent .child { color: red; }", 0);
        assert_eq!(result.warning_count, 1);
        assert!(result.diagnostics[0].fix.is_some());
    }

    #[test]
    fn test_attribute_selector() {
        let linter = create_linter();
        // Space inside attribute selector should not trigger
        let result = linter.lint("[data-foo=\"bar baz\"] { color: red; }", 0);
        assert_eq!(result.warning_count, 0);
    }
}
