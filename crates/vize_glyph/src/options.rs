//! Format options for vize_glyph.
//!
//! These options are designed to be compatible with Prettier and oxfmt.

use serde::{Deserialize, Serialize};

/// Formatting options for Vue SFC
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatOptions {
    /// Maximum line width (default: 100)
    #[serde(default = "default_print_width")]
    pub print_width: u32,

    /// Number of spaces per indentation level (default: 2)
    #[serde(default = "default_tab_width")]
    pub tab_width: u8,

    /// Use tabs instead of spaces (default: false)
    #[serde(default)]
    pub use_tabs: bool,

    /// Print semicolons at the ends of statements (default: true)
    #[serde(default = "default_true")]
    pub semi: bool,

    /// Use single quotes instead of double quotes (default: false)
    #[serde(default)]
    pub single_quote: bool,

    /// Use single quotes in JSX (default: false)
    #[serde(default)]
    pub jsx_single_quote: bool,

    /// Print trailing commas wherever possible (default: All)
    #[serde(default)]
    pub trailing_comma: TrailingComma,

    /// Print spaces between brackets in object literals (default: true)
    #[serde(default = "default_true")]
    pub bracket_spacing: bool,

    /// Put the > of a multi-line HTML element at the end of the last line (default: false)
    #[serde(default)]
    pub bracket_same_line: bool,

    /// Include parentheses around a sole arrow function parameter (default: Always)
    #[serde(default)]
    pub arrow_parens: ArrowParens,

    /// End of line style (default: Lf)
    #[serde(default)]
    pub end_of_line: EndOfLine,

    /// Change when properties in objects are quoted (default: AsNeeded)
    #[serde(default)]
    pub quote_props: QuoteProps,

    /// Put each HTML attribute on its own line (default: false)
    #[serde(default)]
    pub single_attribute_per_line: bool,

    /// Indent script and style tags in Vue files (default: false)
    #[serde(default)]
    pub vue_indent_script_and_style: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            print_width: default_print_width(),
            tab_width: default_tab_width(),
            use_tabs: false,
            semi: true,
            single_quote: false,
            jsx_single_quote: false,
            trailing_comma: TrailingComma::default(),
            bracket_spacing: true,
            bracket_same_line: false,
            arrow_parens: ArrowParens::default(),
            end_of_line: EndOfLine::default(),
            quote_props: QuoteProps::default(),
            single_attribute_per_line: false,
            vue_indent_script_and_style: false,
        }
    }
}

fn default_print_width() -> u32 {
    100
}

fn default_tab_width() -> u8 {
    2
}

fn default_true() -> bool {
    true
}

/// Trailing comma options
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrailingComma {
    /// No trailing commas
    None,
    /// Trailing commas where valid in ES5 (objects, arrays, etc.)
    Es5,
    /// Trailing commas wherever possible
    #[default]
    All,
}

/// Arrow function parentheses options
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArrowParens {
    /// Always include parentheses
    #[default]
    Always,
    /// Omit parentheses when possible
    Avoid,
}

/// End of line options
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EndOfLine {
    /// Line Feed only (\n)
    #[default]
    Lf,
    /// Carriage Return + Line Feed (\r\n)
    Crlf,
    /// Carriage Return only (\r)
    Cr,
    /// Maintain existing line endings
    Auto,
}

/// Quote properties options
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QuoteProps {
    /// Only add quotes around object properties where required
    #[default]
    AsNeeded,
    /// If at least one property in an object requires quotes, quote all properties
    Consistent,
    /// Respect the input use of quotes in object properties
    Preserve,
}

impl FormatOptions {
    /// Create options with Prettier defaults
    #[inline]
    pub fn prettier_compat() -> Self {
        Self {
            print_width: 80,
            ..Default::default()
        }
    }

    /// Get the indent string based on options
    #[inline]
    pub fn indent_string(&self) -> String {
        if self.use_tabs {
            "\t".to_string()
        } else {
            " ".repeat(self.tab_width as usize)
        }
    }

    /// Get the indent as bytes (more efficient for byte operations)
    #[inline]
    pub fn indent_bytes(&self) -> &'static [u8] {
        if self.use_tabs {
            b"\t"
        } else {
            match self.tab_width {
                1 => b" ",
                2 => b"  ",
                4 => b"    ",
                8 => b"        ",
                _ => b"  ", // Default to 2 spaces
            }
        }
    }

    /// Get the newline string based on options
    #[inline]
    pub fn newline_string(&self) -> &'static str {
        match self.end_of_line {
            EndOfLine::Lf | EndOfLine::Auto => "\n",
            EndOfLine::Crlf => "\r\n",
            EndOfLine::Cr => "\r",
        }
    }

    /// Get the newline as bytes (more efficient for byte operations)
    #[inline]
    pub fn newline_bytes(&self) -> &'static [u8] {
        match self.end_of_line {
            EndOfLine::Lf | EndOfLine::Auto => b"\n",
            EndOfLine::Crlf => b"\r\n",
            EndOfLine::Cr => b"\r",
        }
    }

    /// Get the quote character based on options
    #[inline]
    pub fn quote_char(&self) -> char {
        if self.single_quote {
            '\''
        } else {
            '"'
        }
    }

    /// Get the quote as a byte
    #[inline]
    pub fn quote_byte(&self) -> u8 {
        if self.single_quote {
            b'\''
        } else {
            b'"'
        }
    }
}
