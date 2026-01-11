//! Hover information provider.
//!
//! Provides contextual hover information for:
//! - Template expressions and bindings
//! - Vue directives
//! - Script bindings and imports
//! - CSS properties and Vue-specific selectors

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Range};

use super::IdeContext;
use crate::virtual_code::BlockType;

/// Hover service for providing contextual information.
pub struct HoverService;

impl HoverService {
    /// Get hover information for the given context.
    pub fn hover(ctx: &IdeContext) -> Option<Hover> {
        match ctx.block_type? {
            BlockType::Template => Self::hover_template(ctx),
            BlockType::Script => Self::hover_script(ctx, false),
            BlockType::ScriptSetup => Self::hover_script(ctx, true),
            BlockType::Style(index) => Self::hover_style(ctx, index),
        }
    }

    /// Get hover for template context.
    fn hover_template(ctx: &IdeContext) -> Option<Hover> {
        // Try to find what's under the cursor
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset);

        if word.is_empty() {
            return None;
        }

        // Check for Vue directives
        if let Some(hover) = Self::hover_directive(&word) {
            return Some(hover);
        }

        // Try to get type information from vize_canon
        if let Some(type_info) = super::TypeService::get_type_at(ctx) {
            let mut value = format!("**{}**\n\n```typescript\n{}\n```", word, type_info.display);

            if let Some(ref doc) = type_info.documentation {
                value.push_str(&format!("\n\n{}", doc));
            }

            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value,
                }),
                range: None,
            });
        }

        // Check for template bindings from script setup
        if let Some(ref virtual_docs) = ctx.virtual_docs {
            if let Some(ref script_setup) = virtual_docs.script_setup {
                let bindings =
                    crate::virtual_code::extract_simple_bindings(&script_setup.content, true);
                if bindings.contains(&word) {
                    return Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!("**{}**\n\n*Binding from `<script setup>`*", word),
                        }),
                        range: None,
                    });
                }
            }
        }

        // Default: show it's a template expression
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**{}**\n\n*Template expression*", word),
            }),
            range: None,
        })
    }

    /// Get hover for script context.
    fn hover_script(ctx: &IdeContext, is_setup: bool) -> Option<Hover> {
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset);

        if word.is_empty() {
            return None;
        }

        // Check for Vue Composition API
        if let Some(hover) = Self::hover_vue_api(&word) {
            return Some(hover);
        }

        // Check for Vue macros (script setup only)
        if is_setup {
            if let Some(hover) = Self::hover_vue_macro(&word) {
                return Some(hover);
            }
        }

        None
    }

    /// Get hover for style context.
    fn hover_style(ctx: &IdeContext, _index: usize) -> Option<Hover> {
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset);

        if word.is_empty() {
            return None;
        }

        // Check for Vue-specific CSS features
        if let Some(hover) = Self::hover_vue_css(&word) {
            return Some(hover);
        }

        None
    }

    /// Get hover for Vue directives.
    fn hover_directive(word: &str) -> Option<Hover> {
        let (title, description) = match word {
            "v-if" => ("v-if", "Conditionally render the element based on the truthy-ness of the expression value."),
            "v-else-if" => ("v-else-if", "Denote the \"else if block\" for `v-if`. Can be chained."),
            "v-else" => ("v-else", "Denote the \"else block\" for `v-if` or `v-if`/`v-else-if` chain."),
            "v-for" => ("v-for", "Render the element or template block multiple times based on the source data."),
            "v-on" | "@" => ("v-on", "Attach an event listener to the element. The event type is denoted by the argument."),
            "v-bind" | ":" => ("v-bind", "Dynamically bind one or more attributes, or a component prop to an expression."),
            "v-model" => ("v-model", "Create a two-way binding on a form input element or a component."),
            "v-slot" | "#" => ("v-slot", "Denote named slots or scoped slots that expect to receive props."),
            "v-pre" => ("v-pre", "Skip compilation for this element and all its children."),
            "v-once" => ("v-once", "Render the element and component once only, and skip future updates."),
            "v-memo" => ("v-memo", "Memoize a sub-tree of the template. Can be used on both elements and components."),
            "v-cloak" => ("v-cloak", "Used to hide un-compiled template until it is ready."),
            "v-show" => ("v-show", "Toggle the element's visibility based on the truthy-ness of the expression value."),
            "v-text" => ("v-text", "Update the element's text content."),
            "v-html" => ("v-html", "Update the element's innerHTML."),
            _ => return None,
        };

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**{}**\n\n{}\n\n[Vue Documentation](https://vuejs.org/api/built-in-directives.html)", title, description),
            }),
            range: None,
        })
    }

    /// Get hover for Vue Composition API.
    fn hover_vue_api(word: &str) -> Option<Hover> {
        let (signature, description) = match word {
            "ref" => (
                "function ref<T>(value: T): Ref<T>",
                "Takes an inner value and returns a reactive and mutable ref object, which has a single property `.value` that points to the inner value.",
            ),
            "reactive" => (
                "function reactive<T extends object>(target: T): T",
                "Returns a reactive proxy of the object. The reactive conversion is \"deep\": it affects all nested properties.",
            ),
            "computed" => (
                "function computed<T>(getter: () => T): ComputedRef<T>",
                "Takes a getter function and returns a readonly reactive ref object for the returned value from the getter.",
            ),
            "watch" => (
                "function watch<T>(source: WatchSource<T>, callback: WatchCallback<T>): WatchStopHandle",
                "Watches one or more reactive data sources and invokes a callback function when the sources change.",
            ),
            "watchEffect" => (
                "function watchEffect(effect: () => void): WatchStopHandle",
                "Runs a function immediately while reactively tracking its dependencies and re-runs it whenever the dependencies are changed.",
            ),
            "onMounted" => (
                "function onMounted(callback: () => void): void",
                "Registers a callback to be called after the component has been mounted.",
            ),
            "onUnmounted" => (
                "function onUnmounted(callback: () => void): void",
                "Registers a callback to be called after the component has been unmounted.",
            ),
            "onBeforeMount" => (
                "function onBeforeMount(callback: () => void): void",
                "Registers a hook to be called right before the component is to be mounted.",
            ),
            "onBeforeUnmount" => (
                "function onBeforeUnmount(callback: () => void): void",
                "Registers a hook to be called right before a component instance is to be unmounted.",
            ),
            "onUpdated" => (
                "function onUpdated(callback: () => void): void",
                "Registers a callback to be called after the component has updated its DOM tree due to a reactive state change.",
            ),
            "onBeforeUpdate" => (
                "function onBeforeUpdate(callback: () => void): void",
                "Registers a hook to be called right before the component is about to update its DOM tree due to a reactive state change.",
            ),
            "toRef" => (
                "function toRef<T extends object, K extends keyof T>(object: T, key: K): Ref<T[K]>",
                "Creates a ref that is synced with a property of a reactive object.",
            ),
            "toRefs" => (
                "function toRefs<T extends object>(object: T): ToRefs<T>",
                "Converts a reactive object to a plain object where each property is a ref pointing to the corresponding property of the original object.",
            ),
            "unref" => (
                "function unref<T>(ref: T | Ref<T>): T",
                "Returns the inner value if the argument is a ref, otherwise return the argument itself.",
            ),
            "isRef" => (
                "function isRef<T>(r: Ref<T> | unknown): r is Ref<T>",
                "Checks if a value is a ref object.",
            ),
            "shallowRef" => (
                "function shallowRef<T>(value: T): ShallowRef<T>",
                "Shallow version of `ref()`. The inner value is stored and exposed as-is, and will not be made deeply reactive.",
            ),
            "shallowReactive" => (
                "function shallowReactive<T extends object>(target: T): T",
                "Shallow version of `reactive()`. Only the root level is reactive, nested objects are not converted.",
            ),
            "readonly" => (
                "function readonly<T extends object>(target: T): DeepReadonly<T>",
                "Takes an object and returns a readonly proxy of the original.",
            ),
            "nextTick" => (
                "function nextTick(callback?: () => void): Promise<void>",
                "Utility for waiting for the next DOM update flush.",
            ),
            _ => return None,
        };

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "```typescript\n{}\n```\n\n{}\n\n[Vue Documentation](https://vuejs.org/api/)",
                    signature, description
                ),
            }),
            range: None,
        })
    }

    /// Get hover for Vue macros.
    fn hover_vue_macro(word: &str) -> Option<Hover> {
        let (signature, description) = match word {
            "defineProps" => (
                "function defineProps<T>(): T",
                "Compiler macro to declare component props. Only usable inside `<script setup>`.",
            ),
            "defineEmits" => (
                "function defineEmits<T>(): T",
                "Compiler macro to declare component emits. Only usable inside `<script setup>`.",
            ),
            "defineExpose" => (
                "function defineExpose(exposed: Record<string, any>): void",
                "Compiler macro to explicitly expose properties to the parent via template refs.",
            ),
            "defineOptions" => (
                "function defineOptions(options: ComponentOptions): void",
                "Compiler macro to declare component options. Only usable inside `<script setup>`.",
            ),
            "defineSlots" => (
                "function defineSlots<T>(): T",
                "Compiler macro for typed slots. Only usable inside `<script setup>`.",
            ),
            "defineModel" => (
                "function defineModel<T>(name?: string, options?: DefineModelOptions): ModelRef<T>",
                "Compiler macro to declare a two-way binding prop with corresponding update event.",
            ),
            "withDefaults" => (
                "function withDefaults<T>(props: T, defaults: Partial<T>): T",
                "Provides default values for props when using type-only props declaration.",
            ),
            _ => return None,
        };

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "```typescript\n{}\n```\n\n{}\n\n*Compiler macro - only usable inside `<script setup>`*",
                    signature, description
                ),
            }),
            range: None,
        })
    }

    /// Get hover for Vue CSS features.
    fn hover_vue_css(word: &str) -> Option<Hover> {
        let (title, description) = match word {
            "v-bind" => (
                "v-bind() in CSS",
                "Link CSS values to dynamic component state. The value will be compiled into a hashed CSS custom property.",
            ),
            ":deep" => (
                ":deep()",
                "Affects child component styles in scoped CSS. The selector inside `:deep()` will be compiled with the scoped attribute.",
            ),
            ":slotted" => (
                ":slotted()",
                "Target content passed via slots in scoped CSS. Only works inside scoped `<style>` blocks.",
            ),
            ":global" => (
                ":global()",
                "Apply styles globally, escaping the scoped CSS encapsulation.",
            ),
            _ => return None,
        };

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "**{}**\n\n{}\n\n[Vue SFC CSS Features](https://vuejs.org/api/sfc-css-features.html)",
                    title, description
                ),
            }),
            range: None,
        })
    }

    /// Get the word at a given offset.
    fn get_word_at_offset(content: &str, offset: usize) -> String {
        if offset >= content.len() {
            return String::new();
        }

        let bytes = content.as_bytes();

        // If the character at offset is not a word character, return empty
        if !Self::is_word_char(bytes[offset]) {
            return String::new();
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
            return String::new();
        }

        String::from_utf8_lossy(&bytes[start..end]).to_string()
    }

    /// Check if a byte is a valid word character.
    #[inline]
    fn is_word_char(c: u8) -> bool {
        c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'$' || c == b':'
    }
}

/// Hover content builder for creating rich hover information.
pub struct HoverBuilder {
    sections: Vec<String>,
}

impl HoverBuilder {
    /// Create a new hover builder.
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    /// Add a title.
    pub fn title(mut self, title: &str) -> Self {
        self.sections.push(format!("**{}**", title));
        self
    }

    /// Add a code block.
    pub fn code(mut self, language: &str, code: &str) -> Self {
        self.sections
            .push(format!("```{}\n{}\n```", language, code));
        self
    }

    /// Add a description.
    pub fn description(mut self, text: &str) -> Self {
        self.sections.push(text.to_string());
        self
    }

    /// Add a documentation link.
    pub fn link(mut self, text: &str, url: &str) -> Self {
        self.sections.push(format!("[{}]({})", text, url));
        self
    }

    /// Build the hover.
    pub fn build(self) -> Hover {
        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: self.sections.join("\n\n"),
            }),
            range: None,
        }
    }

    /// Build the hover with a range.
    pub fn build_with_range(self, range: Range) -> Hover {
        Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: self.sections.join("\n\n"),
            }),
            range: Some(range),
        }
    }
}

impl Default for HoverBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_word_at_offset() {
        let content = "const message = 'hello'";

        assert_eq!(HoverService::get_word_at_offset(content, 0), "const");
        assert_eq!(HoverService::get_word_at_offset(content, 6), "message");
        assert_eq!(HoverService::get_word_at_offset(content, 5), "");
    }

    #[test]
    fn test_hover_directive() {
        let hover = HoverService::hover_directive("v-if");
        assert!(hover.is_some());

        let hover = HoverService::hover_directive("unknown");
        assert!(hover.is_none());
    }

    #[test]
    fn test_hover_vue_api() {
        let hover = HoverService::hover_vue_api("ref");
        assert!(hover.is_some());

        let hover = HoverService::hover_vue_api("unknown");
        assert!(hover.is_none());
    }

    #[test]
    fn test_hover_builder() {
        let hover = HoverBuilder::new()
            .title("ref")
            .code("typescript", "function ref<T>(value: T): Ref<T>")
            .description("Creates a reactive reference.")
            .link("Documentation", "https://vuejs.org")
            .build();

        if let HoverContents::Markup(content) = hover.contents {
            assert!(content.value.contains("**ref**"));
            assert!(content.value.contains("```typescript"));
        } else {
            panic!("Expected Markup content");
        }
    }
}
