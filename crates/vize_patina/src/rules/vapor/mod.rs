//! Vapor migration lint rules.
//!
//! These rules help identify patterns that are NOT supported in Vue's Vapor mode.
//!
//! Based on Vue 3.6.0-beta.1 release notes:
//! <https://github.com/vuejs/core/releases/tag/v3.6.0-beta.1>
//!
//! ## What is Vapor Mode?
//!
//! Vapor Mode is a new compilation mode for Vue Single-File Components (SFC)
//! with the goal of reducing baseline bundle size and improved performance.
//! It achieves performance parity with Solid and Svelte 5.
//!
//! ## Opt-in Mechanism
//!
//! ```vue
//! <script setup vapor>
//! // component code
//! </script>
//! ```
//!
//! ## NOT Supported in Vapor Mode
//!
//! The following features are **intentionally excluded** from Vapor Mode:
//!
//! - **Options API** - Composition API only
//! - **`app.config.globalProperties`** - Not accessible
//! - **`getCurrentInstance()`** - Returns `null` in Vapor components
//! - **`@vue:xxx` per-element lifecycle events** - Use lifecycle hooks instead
//! - **Suspense** in Vapor-only mode (works when Vapor components render inside VDOM Suspense)
//!
//! ## Template Rules (this module)
//!
//! - `vapor/no-vue-lifecycle-events` - Disallow @vue:xxx per-element lifecycle events
//! - `vapor/no-suspense` - Warn about Suspense usage in Vapor-only apps
//! - `vapor/prefer-static-class` - Prefer static class for performance
//!
//! ## Script Rules (see `rules::script::vapor`)
//!
//! - `script/vapor/no-options-api` - Disallow Options API patterns
//! - `script/vapor/no-get-current-instance` - Disallow getCurrentInstance() calls

mod no_inline_template;
mod no_suspense;
mod no_vue_lifecycle_events;
mod prefer_static_class;
mod require_vapor_attribute;

pub use no_inline_template::NoInlineTemplate;
pub use no_suspense::NoSuspense;
pub use no_vue_lifecycle_events::NoVueLifecycleEvents;
pub use prefer_static_class::PreferStaticClass;
pub use require_vapor_attribute::RequireVaporAttribute;
