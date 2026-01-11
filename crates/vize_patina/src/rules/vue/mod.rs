//! Vue-specific lint rules.
//!
//! These rules are compatible with eslint-plugin-vue's essential and
//! strongly-recommended rule sets.

// Essential rules
mod no_dupe_v_else_if;
mod no_duplicate_attributes;
mod no_reserved_component_names;
mod no_template_key;
mod no_textarea_mustache;
mod no_unused_vars;
mod no_use_v_if_with_v_for;
mod require_v_for_key;
mod valid_v_bind;
mod valid_v_else;
mod valid_v_for;
mod valid_v_if;
mod valid_v_model;
mod valid_v_on;
mod valid_v_show;

// Strongly recommended rules
mod no_multi_spaces;
mod no_template_shadow;
mod v_bind_style;
mod v_on_style;

// Essential rules exports
pub use no_dupe_v_else_if::NoDupeVElseIf;
pub use no_duplicate_attributes::NoDuplicateAttributes;
pub use no_reserved_component_names::NoReservedComponentNames;
pub use no_template_key::NoTemplateKey;
pub use no_textarea_mustache::NoTextareaMustache;
pub use no_unused_vars::NoUnusedVars;
pub use no_use_v_if_with_v_for::NoUseVIfWithVFor;
pub use require_v_for_key::RequireVForKey;
pub use valid_v_bind::ValidVBind;
pub use valid_v_else::ValidVElse;
pub use valid_v_for::ValidVFor;
pub use valid_v_if::ValidVIf;
pub use valid_v_model::ValidVModel;
pub use valid_v_on::ValidVOn;
pub use valid_v_show::ValidVShow;

// Strongly recommended rules exports
pub use no_multi_spaces::NoMultiSpaces;
pub use no_template_shadow::NoTemplateShadow;
pub use v_bind_style::{VBindStyle, VBindStyleOption};
pub use v_on_style::{VOnStyle, VOnStyleOption};
