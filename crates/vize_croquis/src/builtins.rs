//! Vue built-in identifiers for semantic analysis.
//!
//! Provides lookup for various built-in identifiers:
//! - JavaScript globals (Array, Object, Math, etc.)
//! - Vue template builtins ($slots, $emit, $attrs, etc.)
//! - Render function locals (_ctx, _cache, $event, etc.)
//! - Built-in components (Transition, KeepAlive, etc.)
//!
//! Note: For directive checking, use `vize_carton::is_builtin_directive` which
//! provides the complete list for compilation purposes.

use std::sync::LazyLock;
use vize_carton::FxHashSet;

// =============================================================================
// JavaScript Globals
// =============================================================================

/// JavaScript global objects and built-in constructors.
/// These should never be prefixed with _ctx in templates.
pub static JS_GLOBALS: &[&str] = &[
    // ES primitives/values
    "Infinity",
    "undefined",
    "NaN",
    // Built-in constructors
    "Array",
    "Boolean",
    "Date",
    "Error",
    "Function",
    "JSON",
    "Math",
    "Number",
    "Object",
    "Promise",
    "Proxy",
    "Reflect",
    "RegExp",
    "Set",
    "String",
    "Symbol",
    "Map",
    "WeakMap",
    "WeakSet",
    "BigInt",
    // Global functions
    "parseInt",
    "parseFloat",
    "isNaN",
    "isFinite",
    "decodeURI",
    "decodeURIComponent",
    "encodeURI",
    "encodeURIComponent",
    // Browser/Node globals
    "arguments",
    "console",
    "window",
    "document",
    "navigator",
    "globalThis",
    // Module system
    "require",
    "import",
    "exports",
    "module",
];

// =============================================================================
// Render Function Locals
// =============================================================================

/// Local parameters in render function scope.
/// These are always available without _ctx prefix.
pub static RENDER_LOCALS: &[&str] = &[
    "_ctx",    // Component context
    "_cache",  // Cache array for handlers/memoized values
    "_push",   // SSR push function
    "_parent", // SSR parent
];

/// Event handler implicit arguments.
/// $event is automatically available in v-on handlers.
pub static EVENT_LOCALS: &[&str] = &["$event"];

// =============================================================================
// Vue Template Builtins
// =============================================================================

/// Template built-in variables available on component instance.
/// These are accessed via _ctx in compiled code but available
/// directly in template expressions.
pub static VUE_BUILTINS: &[&str] = &[
    "$slots",
    "$refs",
    "$parent",
    "$root",
    "$emit",
    "$attrs",
    "$data",
    "$props",
    "$el",
    "$options",
    "$forceUpdate",
    "$nextTick",
    "$watch",
];

// =============================================================================
// Built-in Components
// =============================================================================

/// Vue built-in components that don't need resolution.
/// These are imported directly from Vue runtime.
pub static BUILTIN_COMPONENTS: &[&str] = &[
    "Transition",
    "TransitionGroup",
    "KeepAlive",
    "Suspense",
    "Teleport",
    "BaseTransition",
];

// =============================================================================
// Pre-computed Sets for Fast Lookup
// =============================================================================

static JS_GLOBALS_SET: LazyLock<FxHashSet<&'static str>> =
    LazyLock::new(|| JS_GLOBALS.iter().copied().collect());

static RENDER_LOCALS_SET: LazyLock<FxHashSet<&'static str>> =
    LazyLock::new(|| RENDER_LOCALS.iter().copied().collect());

static EVENT_LOCALS_SET: LazyLock<FxHashSet<&'static str>> =
    LazyLock::new(|| EVENT_LOCALS.iter().copied().collect());

static VUE_BUILTINS_SET: LazyLock<FxHashSet<&'static str>> =
    LazyLock::new(|| VUE_BUILTINS.iter().copied().collect());

static BUILTIN_COMPONENTS_SET: LazyLock<FxHashSet<&'static str>> =
    LazyLock::new(|| BUILTIN_COMPONENTS.iter().copied().collect());

/// Combined set of all identifiers that should not be prefixed with _ctx.
/// Includes: JS globals + render locals + event locals.
static GLOBAL_ALLOWLIST: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    let mut set = FxHashSet::default();
    set.extend(JS_GLOBALS.iter().copied());
    set.extend(RENDER_LOCALS.iter().copied());
    set.extend(EVENT_LOCALS.iter().copied());
    set
});

// =============================================================================
// Lookup Functions
// =============================================================================

/// Check if a name is a JavaScript global.
#[inline]
pub fn is_js_global(name: &str) -> bool {
    JS_GLOBALS_SET.contains(name)
}

/// Check if a name is a render function local (_ctx, _cache, etc.)
#[inline]
pub fn is_render_local(name: &str) -> bool {
    RENDER_LOCALS_SET.contains(name)
}

/// Check if a name is an event handler local ($event)
#[inline]
pub fn is_event_local(name: &str) -> bool {
    EVENT_LOCALS_SET.contains(name)
}

/// Check if a name is a Vue template builtin ($slots, $emit, etc.)
#[inline]
pub fn is_vue_builtin(name: &str) -> bool {
    VUE_BUILTINS_SET.contains(name)
}

/// Check if a name is a built-in component (Transition, KeepAlive, etc.)
#[inline]
pub fn is_builtin_component(name: &str) -> bool {
    BUILTIN_COMPONENTS_SET.contains(name)
}

/// Check if a name should NOT be prefixed with _ctx.
/// Returns true for: JS globals, render locals, event locals.
#[inline]
pub fn is_global_allowed(name: &str) -> bool {
    GLOBAL_ALLOWLIST.contains(name)
}

/// Get the combined global allowlist set (for iteration).
#[inline]
pub fn global_allowlist() -> &'static FxHashSet<&'static str> {
    &GLOBAL_ALLOWLIST
}

/// Get the Vue builtins set (for iteration).
#[inline]
pub fn vue_builtins_set() -> &'static FxHashSet<&'static str> {
    &VUE_BUILTINS_SET
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_globals() {
        assert!(is_js_global("Array"));
        assert!(is_js_global("Object"));
        assert!(is_js_global("console"));
        assert!(!is_js_global("myVar"));
    }

    #[test]
    fn test_render_locals() {
        assert!(is_render_local("_ctx"));
        assert!(is_render_local("_cache"));
        assert!(!is_render_local("_myLocal"));
    }

    #[test]
    fn test_event_locals() {
        assert!(is_event_local("$event"));
        assert!(!is_event_local("event"));
    }

    #[test]
    fn test_vue_builtins() {
        assert!(is_vue_builtin("$slots"));
        assert!(is_vue_builtin("$emit"));
        assert!(is_vue_builtin("$attrs"));
        assert!(!is_vue_builtin("count"));
    }

    #[test]
    fn test_builtin_components() {
        assert!(is_builtin_component("Transition"));
        assert!(is_builtin_component("KeepAlive"));
        assert!(!is_builtin_component("MyComponent"));
    }

    #[test]
    fn test_global_allowed() {
        // JS globals
        assert!(is_global_allowed("Array"));
        assert!(is_global_allowed("console"));
        // Render locals
        assert!(is_global_allowed("_ctx"));
        assert!(is_global_allowed("_cache"));
        // Event locals
        assert!(is_global_allowed("$event"));
        // Vue builtins are NOT in the allowlist (they need _ctx prefix)
        assert!(!is_global_allowed("$slots"));
        assert!(!is_global_allowed("$emit"));
        // Random identifiers
        assert!(!is_global_allowed("myVar"));
    }
}
