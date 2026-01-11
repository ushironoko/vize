//! Type error diagnostics.

/// A type diagnostic from the type checker.
#[derive(Debug, Clone)]
pub struct TypeDiagnostic {
    /// Error code.
    pub code: TypeErrorCode,
    /// Error message.
    pub message: String,
    /// Severity level.
    pub severity: TypeSeverity,
    /// Start byte offset.
    pub start: u32,
    /// End byte offset.
    pub end: u32,
    /// Related information.
    pub related: Vec<RelatedInfo>,
}

impl TypeDiagnostic {
    /// Create a new error diagnostic.
    pub fn error(code: TypeErrorCode, message: impl Into<String>, start: u32, end: u32) -> Self {
        Self {
            code,
            message: message.into(),
            severity: TypeSeverity::Error,
            start,
            end,
            related: Vec::new(),
        }
    }

    /// Create a new warning diagnostic.
    pub fn warning(code: TypeErrorCode, message: impl Into<String>, start: u32, end: u32) -> Self {
        Self {
            code,
            message: message.into(),
            severity: TypeSeverity::Warning,
            start,
            end,
            related: Vec::new(),
        }
    }

    /// Add related information.
    pub fn with_related(mut self, info: RelatedInfo) -> Self {
        self.related.push(info);
        self
    }
}

/// Severity of a type diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeSeverity {
    /// Error - prevents compilation.
    Error,
    /// Warning - may indicate a problem.
    Warning,
}

/// Related information for a diagnostic.
#[derive(Debug, Clone)]
pub struct RelatedInfo {
    /// Message explaining the relation.
    pub message: String,
    /// Start byte offset.
    pub start: u32,
    /// End byte offset.
    pub end: u32,
}

impl RelatedInfo {
    /// Create new related info.
    pub fn new(message: impl Into<String>, start: u32, end: u32) -> Self {
        Self {
            message: message.into(),
            start,
            end,
        }
    }
}

/// Type error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TypeErrorCode {
    /// Unknown identifier.
    UnknownIdentifier = 2304,
    /// Property does not exist on type.
    PropertyNotFound = 2339,
    /// Argument type mismatch.
    ArgumentTypeMismatch = 2345,
    /// Type is not assignable.
    TypeNotAssignable = 2322,
    /// Type is not callable.
    NotCallable = 2349,
    /// Missing required property.
    MissingProperty = 2741,
    /// Implicit any type.
    ImplicitAny = 7006,
    /// Cannot find module.
    ModuleNotFound = 2307,
    /// Expected arguments.
    ExpectedArguments = 2554,
    /// Too many arguments.
    TooManyArguments = 2555,
    /// Type parameter constraint.
    TypeConstraint = 2344,
    /// Object is possibly undefined.
    PossiblyUndefined = 2532,
    /// Object is possibly null.
    PossiblyNull = 2531,
    /// Cannot use 'new' with this expression.
    NotConstructable = 2351,
    /// Duplicate identifier.
    DuplicateIdentifier = 2300,
    /// Cannot redeclare block-scoped variable.
    CannotRedeclare = 2451,
    /// Vue-specific: Invalid prop type.
    InvalidPropType = 9001,
    /// Vue-specific: Invalid emit.
    InvalidEmit = 9002,
    /// Vue-specific: Unknown component.
    UnknownComponent = 9003,
    /// Vue-specific: Invalid slot usage.
    InvalidSlot = 9004,
    /// Vue-specific: Invalid directive.
    InvalidDirective = 9005,
    /// Vue-specific: Reactivity issue.
    ReactivityIssue = 9006,
}

impl TypeErrorCode {
    /// Get the numeric code.
    pub fn code(&self) -> u32 {
        *self as u32
    }

    /// Get a human-readable name for the error code.
    pub fn name(&self) -> &'static str {
        match self {
            Self::UnknownIdentifier => "unknown-identifier",
            Self::PropertyNotFound => "property-not-found",
            Self::ArgumentTypeMismatch => "argument-type-mismatch",
            Self::TypeNotAssignable => "type-not-assignable",
            Self::NotCallable => "not-callable",
            Self::MissingProperty => "missing-property",
            Self::ImplicitAny => "implicit-any",
            Self::ModuleNotFound => "module-not-found",
            Self::ExpectedArguments => "expected-arguments",
            Self::TooManyArguments => "too-many-arguments",
            Self::TypeConstraint => "type-constraint",
            Self::PossiblyUndefined => "possibly-undefined",
            Self::PossiblyNull => "possibly-null",
            Self::NotConstructable => "not-constructable",
            Self::DuplicateIdentifier => "duplicate-identifier",
            Self::CannotRedeclare => "cannot-redeclare",
            Self::InvalidPropType => "invalid-prop-type",
            Self::InvalidEmit => "invalid-emit",
            Self::UnknownComponent => "unknown-component",
            Self::InvalidSlot => "invalid-slot",
            Self::InvalidDirective => "invalid-directive",
            Self::ReactivityIssue => "reactivity-issue",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_creation() {
        let diag = TypeDiagnostic::error(
            TypeErrorCode::UnknownIdentifier,
            "Cannot find name 'foo'",
            0,
            3,
        );
        assert_eq!(diag.severity, TypeSeverity::Error);
        assert_eq!(diag.code, TypeErrorCode::UnknownIdentifier);
    }

    #[test]
    fn test_error_code() {
        assert_eq!(TypeErrorCode::UnknownIdentifier.code(), 2304);
        assert_eq!(
            TypeErrorCode::UnknownIdentifier.name(),
            "unknown-identifier"
        );
    }
}
