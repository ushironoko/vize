//! Scope analysis for Vue templates and scripts.
//!
//! Provides a hierarchical scope chain that tracks variable visibility
//! across different contexts (module, function, block, v-for, v-slot).
//!
//! ## Performance Optimizations
//!
//! - Uses `CompactString` instead of `String` for identifier names (SSO for short strings)
//! - Uses `SmallVec` for parameter lists (stack-allocated for small counts)
//! - Bitflags for binding properties to reduce memory and improve cache locality
//! - `#[inline]` hints for hot path functions

use vize_carton::{bitflags, CompactString, FxHashMap, SmallVec};
use vize_relief::BindingType;

/// Maximum parameters typically seen in v-for/v-slot/callbacks
/// Stack-allocated up to this count, heap-allocated beyond
const PARAM_INLINE_CAP: usize = 4;

/// Type alias for parameter name lists (stack-allocated for small counts)
pub type ParamNames = SmallVec<[CompactString; PARAM_INLINE_CAP]>;

/// Unique identifier for a scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ScopeId(u32);

impl ScopeId {
    /// The root scope (module level)
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
    /// Module-level scope (top-level of script setup)
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
}

/// Data specific to callback scope
#[derive(Debug, Clone)]
pub struct CallbackScopeData {
    /// Parameter names (stack-allocated for typical cases)
    pub param_names: ParamNames,
    /// Context description (for debugging)
    pub context: CompactString,
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

/// A single scope in the scope chain
#[derive(Debug)]
pub struct Scope {
    /// Unique identifier
    pub id: ScopeId,
    /// Parent scope (None for root)
    pub parent: Option<ScopeId>,
    /// Kind of scope
    pub kind: ScopeKind,
    /// Bindings declared in this scope
    bindings: FxHashMap<CompactString, ScopeBinding>,
    /// Scope-specific data
    data: ScopeData,
    /// Source span
    pub span: Span,
}

impl Scope {
    /// Create a new scope
    #[inline]
    pub fn new(id: ScopeId, parent: Option<ScopeId>, kind: ScopeKind) -> Self {
        Self {
            id,
            parent,
            kind,
            bindings: FxHashMap::default(),
            data: ScopeData::None,
            span: Span::default(),
        }
    }

    /// Create a new scope with span
    #[inline]
    pub fn with_span(
        id: ScopeId,
        parent: Option<ScopeId>,
        kind: ScopeKind,
        start: u32,
        end: u32,
    ) -> Self {
        Self {
            id,
            parent,
            kind,
            bindings: FxHashMap::default(),
            data: ScopeData::None,
            span: Span::new(start, end),
        }
    }

    /// Set scope-specific data
    #[inline]
    pub fn set_data(&mut self, data: ScopeData) {
        self.data = data;
    }

    /// Get scope-specific data
    #[inline]
    pub fn data(&self) -> &ScopeData {
        &self.data
    }

    /// Add a binding to this scope
    #[inline]
    pub fn add_binding(&mut self, name: CompactString, binding: ScopeBinding) {
        self.bindings.insert(name, binding);
    }

    /// Get a binding by name (only in this scope, not parents)
    #[inline]
    pub fn get_binding(&self, name: &str) -> Option<&ScopeBinding> {
        self.bindings.get(name)
    }

    /// Get a mutable binding by name
    #[inline]
    pub fn get_binding_mut(&mut self, name: &str) -> Option<&mut ScopeBinding> {
        self.bindings.get_mut(name)
    }

    /// Check if this scope has a binding
    #[inline]
    pub fn has_binding(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }

    /// Iterate over all bindings in this scope
    #[inline]
    pub fn bindings(&self) -> impl Iterator<Item = (&str, &ScopeBinding)> {
        self.bindings.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Number of bindings in this scope
    #[inline]
    pub fn binding_count(&self) -> usize {
        self.bindings.len()
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

/// Manages the scope chain during analysis
#[derive(Debug)]
pub struct ScopeChain {
    /// All scopes (indexed by ScopeId)
    scopes: Vec<Scope>,
    /// Current scope ID
    current: ScopeId,
}

impl Default for ScopeChain {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeChain {
    /// Create a new scope chain with a root module scope
    #[inline]
    pub fn new() -> Self {
        let root = Scope::new(ScopeId::ROOT, None, ScopeKind::Module);
        Self {
            scopes: vec![root],
            current: ScopeId::ROOT,
        }
    }

    /// Create with pre-allocated capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let root = Scope::new(ScopeId::ROOT, None, ScopeKind::Module);
        let mut scopes = Vec::with_capacity(capacity);
        scopes.push(root);
        Self {
            scopes,
            current: ScopeId::ROOT,
        }
    }

    /// Get the current scope
    #[inline]
    pub fn current_scope(&self) -> &Scope {
        // SAFETY: current is always a valid index
        unsafe { self.scopes.get_unchecked(self.current.as_u32() as usize) }
    }

    /// Get the current scope mutably
    #[inline]
    pub fn current_scope_mut(&mut self) -> &mut Scope {
        let idx = self.current.as_u32() as usize;
        // SAFETY: current is always a valid index
        unsafe { self.scopes.get_unchecked_mut(idx) }
    }

    /// Get a scope by ID
    #[inline]
    pub fn get_scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(id.as_u32() as usize)
    }

    /// Get a scope by ID (unchecked)
    ///
    /// # Safety
    /// Caller must ensure id is valid
    #[inline]
    pub unsafe fn get_scope_unchecked(&self, id: ScopeId) -> &Scope {
        self.scopes.get_unchecked(id.as_u32() as usize)
    }

    /// Current scope ID
    #[inline]
    pub const fn current_id(&self) -> ScopeId {
        self.current
    }

    /// Number of scopes
    #[inline]
    pub fn len(&self) -> usize {
        self.scopes.len()
    }

    /// Check if empty (only root scope)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.scopes.len() == 1
    }

    /// Enter a new scope
    #[inline]
    pub fn enter_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let scope = Scope::new(id, Some(self.current), kind);
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Exit the current scope and return to parent
    #[inline]
    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.current_scope().parent {
            self.current = parent;
        }
    }

    /// Enter a v-for scope with the given data
    pub fn enter_v_for_scope(&mut self, data: VForScopeData, start: u32, end: u32) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let mut scope = Scope::with_span(id, Some(self.current), ScopeKind::VFor, start, end);

        // Add value alias as binding
        scope.add_binding(
            data.value_alias.clone(),
            ScopeBinding::new(BindingType::SetupConst, start),
        );

        // Add key alias if present
        if let Some(ref key) = data.key_alias {
            scope.add_binding(
                key.clone(),
                ScopeBinding::new(BindingType::SetupConst, start),
            );
        }

        // Add index alias if present
        if let Some(ref index) = data.index_alias {
            scope.add_binding(
                index.clone(),
                ScopeBinding::new(BindingType::SetupConst, start),
            );
        }

        scope.set_data(ScopeData::VFor(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter a v-slot scope with the given data
    pub fn enter_v_slot_scope(&mut self, data: VSlotScopeData, start: u32, end: u32) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let mut scope = Scope::with_span(id, Some(self.current), ScopeKind::VSlot, start, end);

        // Add prop names as bindings
        for prop_name in &data.prop_names {
            scope.add_binding(
                prop_name.clone(),
                ScopeBinding::new(BindingType::SetupConst, start),
            );
        }

        scope.set_data(ScopeData::VSlot(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter an event handler scope
    pub fn enter_event_handler_scope(
        &mut self,
        data: EventHandlerScopeData,
        start: u32,
        end: u32,
    ) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let mut scope =
            Scope::with_span(id, Some(self.current), ScopeKind::EventHandler, start, end);

        // Add implicit $event binding if applicable
        if data.has_implicit_event {
            scope.add_binding(
                CompactString::const_new("$event"),
                ScopeBinding::new(BindingType::SetupConst, start),
            );
        }

        // Add explicit parameter names as bindings
        for param_name in &data.param_names {
            scope.add_binding(
                param_name.clone(),
                ScopeBinding::new(BindingType::SetupConst, start),
            );
        }

        scope.set_data(ScopeData::EventHandler(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Enter a callback/arrow function scope
    pub fn enter_callback_scope(
        &mut self,
        data: CallbackScopeData,
        start: u32,
        end: u32,
    ) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        let mut scope = Scope::with_span(id, Some(self.current), ScopeKind::Callback, start, end);

        // Add parameter names as bindings
        for param_name in &data.param_names {
            scope.add_binding(
                param_name.clone(),
                ScopeBinding::new(BindingType::SetupConst, start),
            );
        }

        scope.set_data(ScopeData::Callback(data));
        self.scopes.push(scope);
        self.current = id;
        id
    }

    /// Look up a binding by name, searching up the scope chain
    #[inline]
    pub fn lookup(&self, name: &str) -> Option<(&Scope, &ScopeBinding)> {
        let mut scope_id = Some(self.current);

        while let Some(id) = scope_id {
            // SAFETY: scope_id is always valid (from parent chain)
            let scope = unsafe { self.scopes.get_unchecked(id.as_u32() as usize) };
            if let Some(binding) = scope.get_binding(name) {
                return Some((scope, binding));
            }
            scope_id = scope.parent;
        }

        None
    }

    /// Check if a name is defined in the current scope chain
    #[inline]
    pub fn is_defined(&self, name: &str) -> bool {
        let mut scope_id = Some(self.current);

        while let Some(id) = scope_id {
            // SAFETY: scope_id is always valid (from parent chain)
            let scope = unsafe { self.scopes.get_unchecked(id.as_u32() as usize) };
            if scope.has_binding(name) {
                return true;
            }
            scope_id = scope.parent;
        }

        false
    }

    /// Add a binding to the current scope
    #[inline]
    pub fn add_binding(&mut self, name: CompactString, binding: ScopeBinding) {
        self.current_scope_mut().add_binding(name, binding);
    }

    /// Mark a binding as used
    pub fn mark_used(&mut self, name: &str) {
        let mut scope_id = Some(self.current);

        while let Some(id) = scope_id {
            let scope = &mut self.scopes[id.as_u32() as usize];
            if let Some(binding) = scope.get_binding_mut(name) {
                binding.mark_used();
                return;
            }
            scope_id = scope.parent;
        }
    }

    /// Mark a binding as mutated
    pub fn mark_mutated(&mut self, name: &str) {
        let mut scope_id = Some(self.current);

        while let Some(id) = scope_id {
            let scope = &mut self.scopes[id.as_u32() as usize];
            if let Some(binding) = scope.get_binding_mut(name) {
                binding.mark_mutated();
                return;
            }
            scope_id = scope.parent;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_chain_basic() {
        let mut chain = ScopeChain::new();

        // Add binding to root scope
        chain.add_binding(
            CompactString::new("foo"),
            ScopeBinding::new(BindingType::SetupRef, 0),
        );

        assert!(chain.is_defined("foo"));
        assert!(!chain.is_defined("bar"));

        // Enter a new scope
        chain.enter_scope(ScopeKind::Function);
        chain.add_binding(
            CompactString::new("bar"),
            ScopeBinding::new(BindingType::SetupLet, 10),
        );

        // Can see both foo and bar
        assert!(chain.is_defined("foo"));
        assert!(chain.is_defined("bar"));

        // Exit scope
        chain.exit_scope();

        // Can only see foo now
        assert!(chain.is_defined("foo"));
        assert!(!chain.is_defined("bar"));
    }

    #[test]
    fn test_scope_shadowing() {
        let mut chain = ScopeChain::new();

        chain.add_binding(
            CompactString::new("x"),
            ScopeBinding::new(BindingType::SetupRef, 0),
        );

        chain.enter_scope(ScopeKind::Block);
        chain.add_binding(
            CompactString::new("x"),
            ScopeBinding::new(BindingType::SetupLet, 10),
        );

        // Should find the inner binding
        let (scope, binding) = chain.lookup("x").unwrap();
        assert_eq!(scope.kind, ScopeKind::Block);
        assert_eq!(binding.binding_type, BindingType::SetupLet);
    }

    #[test]
    fn test_v_for_scope() {
        let mut chain = ScopeChain::new();

        chain.enter_v_for_scope(
            VForScopeData {
                value_alias: CompactString::new("item"),
                key_alias: Some(CompactString::new("key")),
                index_alias: Some(CompactString::new("index")),
                source: CompactString::new("items"),
            },
            0,
            100,
        );

        assert!(chain.is_defined("item"));
        assert!(chain.is_defined("key"));
        assert!(chain.is_defined("index"));
        assert!(!chain.is_defined("items")); // source is not a binding
    }

    #[test]
    fn test_v_slot_scope() {
        let mut chain = ScopeChain::new();

        chain.enter_v_slot_scope(
            VSlotScopeData {
                name: CompactString::new("default"),
                props_pattern: Some(CompactString::new("{ item, index }")),
                prop_names: vize_carton::smallvec![
                    CompactString::new("item"),
                    CompactString::new("index")
                ],
            },
            0,
            100,
        );

        assert!(chain.is_defined("item"));
        assert!(chain.is_defined("index"));
    }

    #[test]
    fn test_event_handler_scope() {
        let mut chain = ScopeChain::new();

        chain.enter_event_handler_scope(
            EventHandlerScopeData {
                event_name: CompactString::new("click"),
                has_implicit_event: true,
                param_names: vize_carton::smallvec![],
            },
            0,
            50,
        );

        // $event should be available
        assert!(chain.is_defined("$event"));
    }

    #[test]
    fn test_event_handler_scope_with_params() {
        let mut chain = ScopeChain::new();

        // @click="(e, extra) => handle(e, extra)"
        chain.enter_event_handler_scope(
            EventHandlerScopeData {
                event_name: CompactString::new("click"),
                has_implicit_event: false,
                param_names: vize_carton::smallvec![
                    CompactString::new("e"),
                    CompactString::new("extra")
                ],
            },
            0,
            50,
        );

        // Explicit params should be available
        assert!(chain.is_defined("e"));
        assert!(chain.is_defined("extra"));
        // $event should NOT be available (explicit params used)
        assert!(!chain.is_defined("$event"));
    }

    #[test]
    fn test_callback_scope() {
        let mut chain = ScopeChain::new();

        chain.enter_callback_scope(
            CallbackScopeData {
                param_names: vize_carton::smallvec![CompactString::new("item")],
                context: CompactString::new(":class callback"),
            },
            0,
            50,
        );

        // Callback param should be available
        assert!(chain.is_defined("item"));
        assert_eq!(chain.current_scope().kind, ScopeKind::Callback);
    }

    #[test]
    fn test_nested_v_for() {
        let mut chain = ScopeChain::new();

        // Outer v-for
        chain.enter_v_for_scope(
            VForScopeData {
                value_alias: CompactString::new("row"),
                key_alias: None,
                index_alias: Some(CompactString::new("rowIndex")),
                source: CompactString::new("rows"),
            },
            0,
            200,
        );

        // Inner v-for
        chain.enter_v_for_scope(
            VForScopeData {
                value_alias: CompactString::new("cell"),
                key_alias: None,
                index_alias: Some(CompactString::new("cellIndex")),
                source: CompactString::new("row.cells"),
            },
            50,
            150,
        );

        // All bindings should be visible
        assert!(chain.is_defined("row"));
        assert!(chain.is_defined("rowIndex"));
        assert!(chain.is_defined("cell"));
        assert!(chain.is_defined("cellIndex"));

        // Exit inner
        chain.exit_scope();

        // Inner bindings no longer visible
        assert!(chain.is_defined("row"));
        assert!(chain.is_defined("rowIndex"));
        assert!(!chain.is_defined("cell"));
        assert!(!chain.is_defined("cellIndex"));
    }

    #[test]
    fn test_nested_callback_in_v_for() {
        let mut chain = ScopeChain::new();

        // v-for="item in items"
        chain.enter_v_for_scope(
            VForScopeData {
                value_alias: CompactString::new("item"),
                key_alias: None,
                index_alias: Some(CompactString::new("index")),
                source: CompactString::new("items"),
            },
            0,
            200,
        );

        // @click="(e) => handleClick(item, e)"
        chain.enter_event_handler_scope(
            EventHandlerScopeData {
                event_name: CompactString::new("click"),
                has_implicit_event: false,
                param_names: vize_carton::smallvec![CompactString::new("e")],
            },
            50,
            100,
        );

        // Both v-for bindings and event params should be visible
        assert!(chain.is_defined("item"));
        assert!(chain.is_defined("index"));
        assert!(chain.is_defined("e"));
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
}
