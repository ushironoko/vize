//! Rule trait and registry for lint rules.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use vize_relief::ast::{DirectiveNode, ElementNode, ForNode, IfNode, InterpolationNode, RootNode};

/// Rule category for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCategory {
    /// Essential rules (vue/essential) - prevent errors
    Essential,
    /// Strongly recommended rules (vue/strongly-recommended)
    StronglyRecommended,
    /// Recommended rules (vue/recommended)
    Recommended,
    /// Vapor mode specific rules
    Vapor,
    /// Musea (Art file / Storybook) specific rules
    Musea,
}

/// Rule metadata
pub struct RuleMeta {
    /// Rule name (e.g., "vue/require-v-for-key")
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Rule category
    pub category: RuleCategory,
    /// Whether rule is auto-fixable
    pub fixable: bool,
    /// Default severity
    pub default_severity: Severity,
}

/// Rule trait for implementing lint rules
///
/// Rules implement visitor-like methods that are called during AST traversal.
/// Each method receives a mutable reference to LintContext for reporting diagnostics.
pub trait Rule: Send + Sync {
    /// Get rule metadata
    fn meta(&self) -> &'static RuleMeta;

    /// Run on template root node (called once per template)
    #[allow(unused_variables)]
    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {}

    /// Called when entering an element node
    #[allow(unused_variables)]
    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {}

    /// Called when exiting an element node
    #[allow(unused_variables)]
    fn exit_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {}

    /// Called for each directive on an element
    #[allow(unused_variables)]
    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
    }

    /// Called for v-for nodes
    #[allow(unused_variables)]
    fn check_for<'a>(&self, ctx: &mut LintContext<'a>, for_node: &ForNode<'a>) {}

    /// Called for v-if nodes
    #[allow(unused_variables)]
    fn check_if<'a>(&self, ctx: &mut LintContext<'a>, if_node: &IfNode<'a>) {}

    /// Called for interpolation nodes {{ expr }}
    #[allow(unused_variables)]
    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
    }
}

/// Registry holding all enabled lint rules
pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Register a rule
    pub fn register(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    /// Get all registered rules
    pub fn rules(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }

    /// Create registry with all built-in rules enabled
    pub fn with_recommended() -> Self {
        let mut registry = Self::new();

        // Register Vue essential rules
        registry.register(Box::new(crate::rules::vue::RequireVForKey));
        registry.register(Box::new(crate::rules::vue::ValidVFor));
        registry.register(Box::new(crate::rules::vue::NoUseVIfWithVFor));
        registry.register(Box::new(crate::rules::vue::NoUnusedVars::default()));
        registry.register(Box::new(crate::rules::vue::NoDuplicateAttributes::default()));
        registry.register(Box::new(crate::rules::vue::NoTemplateKey));
        registry.register(Box::new(crate::rules::vue::NoTextareaMustache));
        registry.register(Box::new(crate::rules::vue::ValidVElse));
        registry.register(Box::new(crate::rules::vue::ValidVIf));
        registry.register(Box::new(crate::rules::vue::ValidVOn));
        registry.register(Box::new(crate::rules::vue::ValidVBind));
        registry.register(Box::new(crate::rules::vue::ValidVModel));
        registry.register(Box::new(crate::rules::vue::ValidVShow));
        registry.register(Box::new(crate::rules::vue::NoDupeVElseIf));
        registry.register(Box::new(
            crate::rules::vue::NoReservedComponentNames::default(),
        ));

        // Register Vue strongly recommended rules
        registry.register(Box::new(crate::rules::vue::NoTemplateShadow));
        registry.register(Box::new(crate::rules::vue::NoMultiSpaces::default()));
        registry.register(Box::new(crate::rules::vue::VBindStyle::default()));
        registry.register(Box::new(crate::rules::vue::VOnStyle::default()));

        // Register Vapor mode rules
        registry.register(Box::new(crate::rules::vapor::NoSuspense));
        registry.register(Box::new(crate::rules::vapor::NoInlineTemplate));
        registry.register(Box::new(crate::rules::vapor::NoVueLifecycleEvents));
        registry.register(Box::new(crate::rules::vapor::PreferStaticClass));
        registry.register(Box::new(
            crate::rules::vapor::RequireVaporAttribute::default(),
        ));

        registry
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::with_recommended()
    }
}
