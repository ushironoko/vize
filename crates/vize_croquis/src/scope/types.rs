//! Type definitions for scope analysis.
//!
//! This module contains all the type definitions used in scope analysis:
//! - `ScopeId` - Unique identifier for scopes
//! - `ScopeKind` - Kind of scope (Module, Function, VFor, etc.)
//! - Scope data structures for different scope types
//! - `ScopeBinding` - Binding information within a scope

use vize_carton::{bitflags, CompactString, SmallVec};
use vize_relief::BindingType;

/// Maximum parameters typically seen in v-for/v-slot/callbacks
/// Stack-allocated up to this count, heap-allocated beyond
pub const PARAM_INLINE_CAP: usize = 4;

/// Type alias for parameter name lists (stack-allocated for small counts)
pub type ParamNames = SmallVec<[CompactString; PARAM_INLINE_CAP]>;

/// Parent scope references (typically 1-2 parents)
pub type ParentScopes = SmallVec<[ScopeId; 2]>;

/// Unique identifier for a scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ScopeId(u32);

impl ScopeId {
    /// The root scope (SFC level)
    pub const ROOT: Self = Self(0);

    /// Create a new scope ID
    #[inline(always)]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Kind of scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ScopeKind {
    /// SFC (Single File Component) level scope
    /// This is the root scope that contains script setup/non-script-setup scopes
    Module = 0,
    /// Function scope
    Function = 1,
    /// Block scope (if, for, etc.)
    Block = 2,
    /// v-for scope (template)
    VFor = 3,
    /// v-slot scope (template)
    VSlot = 4,
    /// Event handler scope (@click, etc.)
    EventHandler = 5,
    /// Callback/arrow function scope in expressions
    Callback = 6,
    /// Script setup scope (`<script setup>`)
    ScriptSetup = 7,
    /// Non-script setup scope (Options API, regular `<script>`)
    NonScriptSetup = 8,
    /// Universal scope (SSR - runs on both server and client)
    Universal = 9,
    /// Client-only scope (onMounted, onBeforeUnmount, etc.)
    ClientOnly = 10,
    /// Universal JavaScript global scope (console, Math, Object, Array, etc.)
    /// Works in all runtimes
    JsGlobalUniversal = 11,
    /// Browser-only JavaScript global scope (window, document, navigator, localStorage, etc.)
    /// WARNING: Not available in SSR server context
    JsGlobalBrowser = 12,
    /// Node.js-only JavaScript global scope (process, Buffer, __dirname, require, etc.)
    /// WARNING: Not available in browser context
    JsGlobalNode = 13,
    /// Deno-only JavaScript global scope (Deno namespace)
    JsGlobalDeno = 14,
    /// Bun-only JavaScript global scope (Bun namespace)
    JsGlobalBun = 15,
    /// Vue global scope ($refs, $emit, $slots, $attrs, etc.)
    VueGlobal = 16,
    /// External module scope (imported modules)
    ExternalModule = 17,
    /// Closure scope (function declaration, function expression, arrow function)
    /// Has access to arguments, this, and local variables
    Closure = 18,
}

impl ScopeKind {
    /// Get the display prefix for this scope kind
    /// - `~` = universal (works on both client and server)
    /// - `!` = client only (requires client API: window, document, etc.)
    /// - `#` = server private (reserved for future Server Components)
    #[inline]
    pub const fn prefix(&self) -> &'static str {
        match self {
            // Client-only (requires client API)
            Self::ClientOnly | Self::JsGlobalBrowser => "!",
            // Server private (reserved for future Server Components)
            Self::JsGlobalNode | Self::JsGlobalDeno | Self::JsGlobalBun => "#",
            // Universal (works on both)
            _ => "~",
        }
    }

    /// Get the display name for this scope kind
    #[inline]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Module => "mod",
            Self::Function => "fn",
            Self::Block => "block",
            Self::VFor => "v-for",
            Self::VSlot => "v-slot",
            Self::EventHandler => "event",
            Self::Callback => "cb",
            Self::ScriptSetup => "setup",
            Self::NonScriptSetup => "plain",
            Self::Universal => "universal",
            Self::ClientOnly => "client",
            Self::JsGlobalUniversal => "univ",
            Self::JsGlobalBrowser => "client",
            Self::JsGlobalNode => "server",
            Self::JsGlobalDeno => "server",
            Self::JsGlobalBun => "server",
            Self::VueGlobal => "vue",
            Self::ExternalModule => "extern",
            Self::Closure => "closure",
        }
    }

    /// Format for VIR display (zero allocation)
    #[inline]
    pub const fn to_display(&self) -> &'static str {
        match self {
            Self::Module => "mod",
            Self::Function => "fn",
            Self::Block => "block",
            Self::VFor => "v-for",
            Self::VSlot => "v-slot",
            Self::EventHandler => "event",
            Self::Callback => "cb",
            Self::ScriptSetup => "setup",
            Self::NonScriptSetup => "plain",
            Self::Universal => "universal",
            Self::ClientOnly => "client",
            Self::JsGlobalUniversal => "univ",
            Self::JsGlobalBrowser => "client",
            Self::JsGlobalNode => "server",
            Self::JsGlobalDeno => "server",
            Self::JsGlobalBun => "server",
            Self::VueGlobal => "vue",
            Self::ExternalModule => "extern",
            Self::Closure => "closure",
        }
    }

    /// Get reference prefix for parent scope references
    /// - `~` = universal (works on both client and server)
    /// - `!` = client only (requires client API)
    /// - `#` = server private (reserved for future Server Components)
    #[inline]
    pub const fn ref_prefix(&self) -> &'static str {
        match self {
            Self::ClientOnly | Self::JsGlobalBrowser => "!",
            Self::JsGlobalNode | Self::JsGlobalDeno | Self::JsGlobalBun => "#",
            _ => "~",
        }
    }
}

/// Data specific to v-for scope
#[derive(Debug, Clone)]
pub struct VForScopeData {
    /// The value alias (e.g., "item" in v-for="item in items")
    pub value_alias: CompactString,
    /// The key alias (e.g., "key" in v-for="(item, key) in items")
    pub key_alias: Option<CompactString>,
    /// The index alias (e.g., "index" in v-for="(item, index) in items")
    pub index_alias: Option<CompactString>,
    /// The source expression (e.g., "items")
    pub source: CompactString,
    /// The :key expression if present (e.g., "item.id")
    pub key_expression: Option<CompactString>,
}

/// Data specific to v-slot scope
#[derive(Debug, Clone)]
pub struct VSlotScopeData {
    /// Slot name
    pub name: CompactString,
    /// Props pattern (e.g., "{ item, index }" in v-slot="{ item, index }")
    pub props_pattern: Option<CompactString>,
    /// Extracted prop names (stack-allocated for typical cases)
    pub prop_names: ParamNames,
}

/// Data specific to event handler scope
#[derive(Debug, Clone)]
pub struct EventHandlerScopeData {
    /// Event name (e.g., "click")
    pub event_name: CompactString,
    /// Whether this handler has implicit $event
    pub has_implicit_event: bool,
    /// Explicit parameter names (stack-allocated for typical cases)
    pub param_names: ParamNames,
    /// The handler expression (e.g., "handleClick" or "handleClick($event)")
    pub handler_expression: Option<CompactString>,
    /// Target component name (for component custom events, e.g., "TodoItem")
    /// None for DOM element events
    pub target_component: Option<CompactString>,
}

/// Data specific to callback scope
#[derive(Debug, Clone)]
pub struct CallbackScopeData {
    /// Parameter names (stack-allocated for typical cases)
    pub param_names: ParamNames,
    /// Context description (for debugging)
    pub context: CompactString,
}

/// Data specific to script setup scope
#[derive(Debug, Clone)]
pub struct ScriptSetupScopeData {
    /// Whether this is TypeScript
    pub is_ts: bool,
    /// Whether async setup
    pub is_async: bool,
    /// Generic type parameter from `<script setup generic="T">`
    pub generic: Option<CompactString>,
}

/// Data specific to non-script-setup scope (Options API, regular script)
#[derive(Debug, Clone)]
pub struct NonScriptSetupScopeData {
    /// Whether this is TypeScript
    pub is_ts: bool,
    /// Whether using defineComponent
    pub has_define_component: bool,
}

/// Data specific to client-only scope (onMounted, onBeforeUnmount, etc.)
#[derive(Debug, Clone)]
pub struct ClientOnlyScopeData {
    /// The lifecycle hook name (e.g., "onMounted", "onBeforeUnmount")
    pub hook_name: CompactString,
}

/// Data specific to universal scope (SSR - runs on both server and client)
#[derive(Debug, Clone)]
pub struct UniversalScopeData {
    /// Context description
    pub context: CompactString,
}

/// Runtime environment for JavaScript globals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum JsRuntime {
    /// Universal - works in all runtimes (console, Math, Object, Array, JSON, etc.)
    Universal = 0,
    /// Browser - window, document, navigator, localStorage, etc.
    Browser = 1,
    /// Node.js - process, Buffer, __dirname, __filename, require, etc.
    Node = 2,
    /// Deno - Deno namespace
    Deno = 3,
    /// Bun - Bun namespace
    Bun = 4,
}

impl JsRuntime {
    /// Get the corresponding ScopeKind for this runtime
    #[inline]
    pub const fn to_scope_kind(self) -> ScopeKind {
        match self {
            JsRuntime::Universal => ScopeKind::JsGlobalUniversal,
            JsRuntime::Browser => ScopeKind::JsGlobalBrowser,
            JsRuntime::Node => ScopeKind::JsGlobalNode,
            JsRuntime::Deno => ScopeKind::JsGlobalDeno,
            JsRuntime::Bun => ScopeKind::JsGlobalBun,
        }
    }

    /// Get the corresponding BindingType for this runtime
    #[inline]
    pub const fn to_binding_type(self) -> BindingType {
        match self {
            JsRuntime::Universal => BindingType::JsGlobalUniversal,
            JsRuntime::Browser => BindingType::JsGlobalBrowser,
            JsRuntime::Node => BindingType::JsGlobalNode,
            JsRuntime::Deno => BindingType::JsGlobalDeno,
            JsRuntime::Bun => BindingType::JsGlobalBun,
        }
    }
}

/// Data specific to JavaScript global scope
#[derive(Debug, Clone)]
pub struct JsGlobalScopeData {
    /// Runtime environment
    pub runtime: JsRuntime,
    /// Known JS globals for this runtime
    pub globals: ParamNames,
}

/// Data specific to Vue global scope
#[derive(Debug, Clone)]
pub struct VueGlobalScopeData {
    /// Known Vue globals ($refs, $emit, $slots, $attrs, $el, etc.)
    pub globals: ParamNames,
}

/// Data specific to external module scope
#[derive(Debug, Clone)]
pub struct ExternalModuleScopeData {
    /// Module source path
    pub source: CompactString,
    /// Whether this is a type-only import
    pub is_type_only: bool,
}

/// Data specific to closure scope (function declaration, function expression, arrow function)
#[derive(Debug, Clone)]
pub struct ClosureScopeData {
    /// Function name (if named function)
    pub name: Option<CompactString>,
    /// Parameter names
    pub param_names: ParamNames,
    /// Whether this is an arrow function (no `arguments`, no `this` binding)
    pub is_arrow: bool,
    /// Whether this is async
    pub is_async: bool,
    /// Whether this is a generator
    pub is_generator: bool,
}

/// Block kind for block scopes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockKind {
    Block,
    If,
    Else,
    For,
    ForIn,
    ForOf,
    While,
    DoWhile,
    Switch,
    Try,
    Catch,
    Finally,
    With,
}

impl BlockKind {
    /// Get the display name for this block kind
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Block => "block",
            Self::If => "if",
            Self::Else => "else",
            Self::For => "for",
            Self::ForIn => "for-in",
            Self::ForOf => "for-of",
            Self::While => "while",
            Self::DoWhile => "do-while",
            Self::Switch => "switch",
            Self::Try => "try",
            Self::Catch => "catch",
            Self::Finally => "finally",
            Self::With => "with",
        }
    }
}

/// Data specific to block scope (if, for, switch, etc.)
#[derive(Debug, Clone, Copy)]
pub struct BlockScopeData {
    /// Block kind
    pub kind: BlockKind,
}

/// Scope-specific data
#[derive(Debug, Clone)]
pub enum ScopeData {
    /// No additional data
    None,
    /// v-for specific data
    VFor(VForScopeData),
    /// v-slot specific data
    VSlot(VSlotScopeData),
    /// Event handler specific data
    EventHandler(EventHandlerScopeData),
    /// Callback specific data
    Callback(CallbackScopeData),
    /// Script setup specific data
    ScriptSetup(ScriptSetupScopeData),
    /// Non-script-setup specific data
    NonScriptSetup(NonScriptSetupScopeData),
    /// Client-only specific data
    ClientOnly(ClientOnlyScopeData),
    /// Universal scope specific data
    Universal(UniversalScopeData),
    /// JavaScript global specific data (with runtime info)
    JsGlobal(JsGlobalScopeData),
    /// Vue global specific data
    VueGlobal(VueGlobalScopeData),
    /// External module specific data
    ExternalModule(ExternalModuleScopeData),
    /// Closure specific data
    Closure(ClosureScopeData),
    /// Block specific data
    Block(BlockScopeData),
}

impl Default for ScopeData {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}

/// Source span
#[derive(Debug, Clone, Copy, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    #[inline(always)]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

bitflags! {
    /// Binding flags for tracking usage and mutation
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct BindingFlags: u8 {
        /// Binding has been referenced
        const USED = 1 << 0;
        /// Binding has been mutated
        const MUTATED = 1 << 1;
        /// Binding is a rest parameter
        const REST = 1 << 2;
        /// Binding has a default value
        const HAS_DEFAULT = 1 << 3;
    }
}

/// A binding within a scope
#[derive(Debug, Clone, Copy)]
pub struct ScopeBinding {
    /// The type of binding
    pub binding_type: BindingType,
    /// Source location of the declaration (offset in source)
    pub declaration_offset: u32,
    /// Binding flags
    flags: BindingFlags,
}

impl ScopeBinding {
    /// Create a new scope binding
    #[inline]
    pub const fn new(binding_type: BindingType, declaration_offset: u32) -> Self {
        Self {
            binding_type,
            declaration_offset,
            flags: BindingFlags::empty(),
        }
    }

    /// Check if binding is used
    #[inline]
    pub const fn is_used(&self) -> bool {
        self.flags.contains(BindingFlags::USED)
    }

    /// Check if binding is mutated
    #[inline]
    pub const fn is_mutated(&self) -> bool {
        self.flags.contains(BindingFlags::MUTATED)
    }

    /// Mark as used
    #[inline]
    pub fn mark_used(&mut self) {
        self.flags.insert(BindingFlags::USED);
    }

    /// Mark as mutated
    #[inline]
    pub fn mark_mutated(&mut self) {
        self.flags.insert(BindingFlags::MUTATED);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn test_scope_id_constants() {
        assert_eq!(ScopeId::ROOT.as_u32(), 0);
        assert_eq!(ScopeId::new(42).as_u32(), 42);
    }

    #[test]
    fn test_scope_kind_display() {
        assert_snapshot!(
            "scope_kind_display",
            format!(
                "Module: {} (prefix: {})\n\
             Function: {} (prefix: {})\n\
             Block: {} (prefix: {})\n\
             VFor: {} (prefix: {})\n\
             VSlot: {} (prefix: {})\n\
             EventHandler: {} (prefix: {})\n\
             Callback: {} (prefix: {})\n\
             ScriptSetup: {} (prefix: {})\n\
             NonScriptSetup: {} (prefix: {})\n\
             Universal: {} (prefix: {})\n\
             ClientOnly: {} (prefix: {})\n\
             JsGlobalUniversal: {} (prefix: {})\n\
             JsGlobalBrowser: {} (prefix: {})\n\
             JsGlobalNode: {} (prefix: {})\n\
             VueGlobal: {} (prefix: {})\n\
             ExternalModule: {} (prefix: {})\n\
             Closure: {} (prefix: {})",
                ScopeKind::Module.to_display(),
                ScopeKind::Module.prefix(),
                ScopeKind::Function.to_display(),
                ScopeKind::Function.prefix(),
                ScopeKind::Block.to_display(),
                ScopeKind::Block.prefix(),
                ScopeKind::VFor.to_display(),
                ScopeKind::VFor.prefix(),
                ScopeKind::VSlot.to_display(),
                ScopeKind::VSlot.prefix(),
                ScopeKind::EventHandler.to_display(),
                ScopeKind::EventHandler.prefix(),
                ScopeKind::Callback.to_display(),
                ScopeKind::Callback.prefix(),
                ScopeKind::ScriptSetup.to_display(),
                ScopeKind::ScriptSetup.prefix(),
                ScopeKind::NonScriptSetup.to_display(),
                ScopeKind::NonScriptSetup.prefix(),
                ScopeKind::Universal.to_display(),
                ScopeKind::Universal.prefix(),
                ScopeKind::ClientOnly.to_display(),
                ScopeKind::ClientOnly.prefix(),
                ScopeKind::JsGlobalUniversal.to_display(),
                ScopeKind::JsGlobalUniversal.prefix(),
                ScopeKind::JsGlobalBrowser.to_display(),
                ScopeKind::JsGlobalBrowser.prefix(),
                ScopeKind::JsGlobalNode.to_display(),
                ScopeKind::JsGlobalNode.prefix(),
                ScopeKind::VueGlobal.to_display(),
                ScopeKind::VueGlobal.prefix(),
                ScopeKind::ExternalModule.to_display(),
                ScopeKind::ExternalModule.prefix(),
                ScopeKind::Closure.to_display(),
                ScopeKind::Closure.prefix(),
            )
        );
    }

    #[test]
    fn test_block_kind_display() {
        assert_snapshot!(
            "block_kind_display",
            format!(
                "Block: {}\n\
             If: {}\n\
             Else: {}\n\
             For: {}\n\
             ForIn: {}\n\
             ForOf: {}\n\
             While: {}\n\
             DoWhile: {}\n\
             Switch: {}\n\
             Try: {}\n\
             Catch: {}\n\
             Finally: {}\n\
             With: {}",
                BlockKind::Block.as_str(),
                BlockKind::If.as_str(),
                BlockKind::Else.as_str(),
                BlockKind::For.as_str(),
                BlockKind::ForIn.as_str(),
                BlockKind::ForOf.as_str(),
                BlockKind::While.as_str(),
                BlockKind::DoWhile.as_str(),
                BlockKind::Switch.as_str(),
                BlockKind::Try.as_str(),
                BlockKind::Catch.as_str(),
                BlockKind::Finally.as_str(),
                BlockKind::With.as_str(),
            )
        );
    }

    #[test]
    fn test_js_runtime_conversions() {
        assert_eq!(
            JsRuntime::Universal.to_scope_kind(),
            ScopeKind::JsGlobalUniversal
        );
        assert_eq!(
            JsRuntime::Browser.to_scope_kind(),
            ScopeKind::JsGlobalBrowser
        );
        assert_eq!(JsRuntime::Node.to_scope_kind(), ScopeKind::JsGlobalNode);
        assert_eq!(JsRuntime::Deno.to_scope_kind(), ScopeKind::JsGlobalDeno);
        assert_eq!(JsRuntime::Bun.to_scope_kind(), ScopeKind::JsGlobalBun);

        assert_eq!(
            JsRuntime::Universal.to_binding_type(),
            BindingType::JsGlobalUniversal
        );
        assert_eq!(
            JsRuntime::Browser.to_binding_type(),
            BindingType::JsGlobalBrowser
        );
        assert_eq!(JsRuntime::Node.to_binding_type(), BindingType::JsGlobalNode);
    }

    #[test]
    fn test_binding_flags() {
        let mut binding = ScopeBinding::new(BindingType::SetupRef, 0);
        assert!(!binding.is_used());
        assert!(!binding.is_mutated());

        binding.mark_used();
        assert!(binding.is_used());
        assert!(!binding.is_mutated());

        binding.mark_mutated();
        assert!(binding.is_used());
        assert!(binding.is_mutated());
    }

    #[test]
    fn test_span() {
        let span = Span::new(10, 50);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 50);

        let default_span = Span::default();
        assert_eq!(default_span.start, 0);
        assert_eq!(default_span.end, 0);
    }

    #[test]
    fn test_scope_data_default() {
        let data = ScopeData::default();
        assert!(matches!(data, ScopeData::None));
    }
}
