//! Reactivity tracking for Vue templates.
//!
//! Tracks reactive sources (ref, reactive, computed) and their dependencies.

use vize_carton::{CompactString, FxHashMap};

/// Unique identifier for a reactive source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ReactiveId(u32);

impl ReactiveId {
    #[inline(always)]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Kind of reactive source
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ReactiveKind {
    /// ref()
    Ref = 0,
    /// shallowRef()
    ShallowRef = 1,
    /// reactive()
    Reactive = 2,
    /// shallowReactive()
    ShallowReactive = 3,
    /// computed()
    Computed = 4,
    /// readonly()
    Readonly = 5,
    /// shallowReadonly()
    ShallowReadonly = 6,
    /// toRef()
    ToRef = 7,
    /// toRefs()
    ToRefs = 8,
}

impl ReactiveKind {
    /// Check if this kind requires .value access
    #[inline]
    pub const fn needs_value_access(self) -> bool {
        matches!(
            self,
            Self::Ref | Self::ShallowRef | Self::Computed | Self::ToRef
        )
    }

    /// Get the kind from a function name
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "ref" => Some(Self::Ref),
            "shallowRef" => Some(Self::ShallowRef),
            "reactive" => Some(Self::Reactive),
            "shallowReactive" => Some(Self::ShallowReactive),
            "computed" => Some(Self::Computed),
            "readonly" => Some(Self::Readonly),
            "shallowReadonly" => Some(Self::ShallowReadonly),
            "toRef" => Some(Self::ToRef),
            "toRefs" => Some(Self::ToRefs),
            _ => None,
        }
    }

    /// Get display abbreviation for VIR output
    /// - st = state (ref)
    /// - ist = implicit state (reactive - no .value needed)
    /// - drv = derived (computed)
    #[inline]
    pub const fn to_display(self) -> &'static str {
        match self {
            Self::Ref | Self::ShallowRef | Self::ToRef | Self::ToRefs => "st",
            Self::Reactive | Self::ShallowReactive => "ist",
            Self::Computed => "drv",
            Self::Readonly | Self::ShallowReadonly => "ro",
        }
    }
}

/// A reactive source in the code
#[derive(Debug, Clone)]
pub struct ReactiveSource {
    pub id: ReactiveId,
    pub name: CompactString,
    pub kind: ReactiveKind,
    pub declaration_offset: u32,
}

/// Tracks reactive sources during analysis
#[derive(Debug, Default)]
pub struct ReactivityTracker {
    sources: Vec<ReactiveSource>,
    by_name: FxHashMap<CompactString, ReactiveId>,
    next_id: u32,
}

impl ReactivityTracker {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a reactive source
    pub fn register(
        &mut self,
        name: CompactString,
        kind: ReactiveKind,
        declaration_offset: u32,
    ) -> ReactiveId {
        let id = ReactiveId::new(self.next_id);
        self.next_id += 1;

        self.by_name.insert(name.clone(), id);
        self.sources.push(ReactiveSource {
            id,
            name,
            kind,
            declaration_offset,
        });

        id
    }

    /// Look up a reactive source by name
    #[inline]
    pub fn lookup(&self, name: &str) -> Option<&ReactiveSource> {
        self.by_name
            .get(name)
            .and_then(|id| self.sources.get(id.as_u32() as usize))
    }

    /// Check if a name is a reactive source
    #[inline]
    pub fn is_reactive(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }

    /// Check if a name needs .value access
    #[inline]
    pub fn needs_value_access(&self, name: &str) -> bool {
        self.lookup(name)
            .is_some_and(|s| s.kind.needs_value_access())
    }

    /// Get all reactive sources
    #[inline]
    pub fn sources(&self) -> &[ReactiveSource] {
        &self.sources
    }

    /// Get source count
    #[inline]
    pub fn count(&self) -> usize {
        self.sources.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reactivity_tracker() {
        let mut tracker = ReactivityTracker::new();

        tracker.register(CompactString::new("count"), ReactiveKind::Ref, 0);
        tracker.register(CompactString::new("state"), ReactiveKind::Reactive, 20);

        assert!(tracker.is_reactive("count"));
        assert!(tracker.needs_value_access("count"));

        assert!(tracker.is_reactive("state"));
        assert!(!tracker.needs_value_access("state"));

        assert!(!tracker.is_reactive("unknown"));
    }
}
