//! vue/single-style-block
//!
//! Recommend having a single style block in Vue SFCs.
//!
//! Multiple style blocks can make CSS harder to organize and maintain.
//! Using a single style block encourages better structure and makes
//! it easier to understand the component's styling.
//!
//! ## Exceptions
//!
//! This rule does not warn when style blocks have different purposes:
//! - One scoped and one global style block
//!
//! ## Examples
//!
//! Bad:
//! ```vue
//! <style scoped>
//! .component { color: red; }
//! </style>
//!
//! <style scoped>
//! .other { color: blue; }
//! </style>
//! ```
//!
//! Good:
//! ```vue
//! <style scoped>
//! .component { color: red; }
//! .other { color: blue; }
//! </style>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::RootNode;

static META: RuleMeta = RuleMeta {
    name: "vue/single-style-block",
    description: "Recommend having a single style block",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Single style block rule
#[derive(Default)]
pub struct SingleStyleBlock;

impl Rule for SingleStyleBlock {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        let source = ctx.source;

        // Find all <style tags and collect their info
        #[derive(Debug)]
        struct StyleBlock {
            pos: usize,
            scoped: bool,
        }

        let mut style_blocks: Vec<StyleBlock> = Vec::new();
        let mut pos = 0;

        while let Some(style_start) = source[pos..].find("<style") {
            let abs_pos = pos + style_start;
            pos = abs_pos + 6;

            // Find the closing >
            if let Some(tag_end) = source[abs_pos..].find('>') {
                let tag_content = &source[abs_pos..abs_pos + tag_end + 1];
                let scoped = tag_content.contains("scoped");

                style_blocks.push(StyleBlock {
                    pos: abs_pos,
                    scoped,
                });
            }
        }

        // If there are multiple style blocks
        if style_blocks.len() > 1 {
            // Check if they're all the same type (all scoped or all non-scoped)
            let all_scoped = style_blocks.iter().all(|s| s.scoped);
            let all_non_scoped = style_blocks.iter().all(|s| !s.scoped);

            // Only warn if all style blocks have the same scoped status
            // (having one scoped and one global is a valid pattern)
            if all_scoped || all_non_scoped {
                // Warn on the second and subsequent style blocks
                for style in style_blocks.iter().skip(1) {
                    let scope_type = if style.scoped { "scoped" } else { "global" };
                    ctx.report(
                        LintDiagnostic::warn(
                            META.name,
                            format!(
                                "Consider merging multiple {} style blocks into one",
                                scope_type
                            ),
                            style.pos as u32,
                            (style.pos + 6) as u32, // "<style"
                        )
                        .with_help("Multiple style blocks of the same type can be merged for better organization"),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests would need SFC-level testing infrastructure
}
