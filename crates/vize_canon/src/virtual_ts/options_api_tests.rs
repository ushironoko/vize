//! Bug report — failing tests describing the Options API type-checking
//! gap in `generate_virtual_ts_with_offsets`.
//!
//! Each test below currently fails on `main` because the virtual TS
//! generator passes the Options API `export default { ... }` block
//! through verbatim instead of producing a typed bridge for `this`-
//! resolved identifiers. Concretely, a single SFC of the shape used
//! in these fixtures produces (under `tsgo` via `vize check`):
//!
//!  - **TS2339** — `Property '<x>' does not exist on type '{ ... }'.`
//!    on every `this.<x>` reference inside `computed` / `methods`.
//!  - **TS7023** — `'<name>' implicitly has return type 'any' because it
//!    is referenced directly or indirectly in one of its return
//!    expressions.`
//!  - **TS2304** — `Cannot find name '<x>'.` inside `<template>` for
//!    identifiers that resolve via runtime `this`.
//!
//! All fixtures are deliberately domain-free (`useFakeStore`, `count`,
//! `message`, `bump`, `greeting`, etc.) so the contract reads as a pure
//! Options API + template specification.
//!
//! These tests are `#[ignore]` by default so they do not break CI; run
//! them explicitly with:
//!
//!     cargo test -p vize_canon --lib options_api_tests -- --ignored

use crate::virtual_ts::generate_virtual_ts_with_offsets;
use vize_croquis::{Analyzer, AnalyzerOptions};

const OPTIONS_API_FIXTURE: &str = r#"import { defineComponent } from 'vue'

function useFakeStore() {
    return {
        ready: false,
        items: [] as Array<{ id: number; label: string }>,
    }
}

export default defineComponent({
    setup() {
        const store = useFakeStore()
        return { store }
    },
    data() {
        return { count: 0 }
    },
    computed: {
        status() {
            return this.store.ready
        },
    },
    methods: {
        bump(step) {
            this.count = this.count + step
            return this.status
        },
    },
    props: {
        initial: { type: Number, default: 0 },
    },
})
"#;

fn analyze_options_api_script(script: &str) -> vize_croquis::Croquis {
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_plain(script);
    analyzer.finish()
}

/// The virtual TS output for an Options API component should declare a
/// typed `this` alias so the bodies of `computed` / `methods` can resolve
/// `this.<x>` against `setup()` returns, `data()` returns, and the rest
/// of the bucket shapes. Without such an alias every `this.<x>` access
/// in those bodies reports TS2339 / TS7023 when `vize check` runs.
#[test]
#[ignore = "documents the missing Options API typed-`this` bridge (run with `cargo test -- --ignored`)"]
fn options_api_virtual_ts_emits_this_bridge() {
    let summary = analyze_options_api_script(OPTIONS_API_FIXTURE);
    let output = generate_virtual_ts_with_offsets(
        &summary,
        Some(OPTIONS_API_FIXTURE),
        None,
        0,
        0,
        &Default::default(),
    );

    assert!(
        output.code.contains("type __VizeThis ="),
        "expected the virtual TS to declare a typed alias for `this` (e.g. \
         `type __VizeThis = ...`) so that `this.<x>` inside computed / \
         methods can resolve. Without it every `this.<x>` access reports \
         TS2339 / TS7023. Got:\n---\n{}\n---",
        output.code
    );
}

/// Pinia / Vuex spread helpers (`...mapState(useStore, ['a', 'b'])`)
/// inject keys onto `this`. The virtual TS needs to surface a precise
/// mapped type (`{ [K in 'a' | 'b']: ReturnType<typeof useStore>[K] }`)
/// so that `this.a` and `this.b` typecheck. A plain `Record<string, any>`
/// stub silently loses precision and lets unrelated typos pass.
#[test]
#[ignore = "documents the missing pinia/Vuex spread typed-shape emission (run with `cargo test -- --ignored`)"]
fn options_api_virtual_ts_emits_typed_shape_for_pinia_spread_helpers() {
    let script = r#"import { defineComponent } from 'vue'
import { mapState, mapActions } from 'pinia'

function useFakeStore() {
    return {
        ready: false,
        items: [] as Array<{ id: number; label: string }>,
        setReady(_v: boolean) {},
    }
}

export default defineComponent({
    computed: {
        ...mapState(useFakeStore, ['items', 'ready']),
        localComputed() { return 1 },
    },
    methods: {
        ...mapActions(useFakeStore, ['setReady']),
    },
})
"#;
    let summary = analyze_options_api_script(script);
    let output = generate_virtual_ts_with_offsets(
        &summary,
        Some(script),
        None,
        0,
        0,
        &Default::default(),
    );

    assert!(
        output.code.contains(
            "[K in 'items' | 'ready']: ReturnType<typeof useFakeStore>[K]"
        ),
        "expected a precise mapped type for the `mapState` spread keys. \
         A `Record<string, any>` stub would silently let typos and missing \
         keys pass. Got:\n{}",
        output.code
    );
    assert!(
        output.code.contains(
            "[K in 'setReady']: ReturnType<typeof useFakeStore>[K]"
        ),
        "expected a precise mapped type for the `mapActions` spread keys. \
         Got:\n{}",
        output.code
    );
}

/// Regression guard: this test should keep passing on `main`. It asserts
/// that the virtual TS for `<script setup>` SFCs does **not** contain
/// the typed-`this` alias the failing tests above ask for — `<script
/// setup>` has its own (working) code path and a future fix must not
/// leak the Options API helpers into it.
#[test]
fn script_setup_remains_unchanged_when_options_api_absent() {
    let script = r#"import { ref } from 'vue'
const count = ref(0)
"#;
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets(
        &summary,
        Some(script),
        None,
        0,
        0,
        &Default::default(),
    );

    assert!(
        !output.code.contains("__VizeThis"),
        "`<script setup>` output must not contain the Options API bridge \
         alias `__VizeThis`. Got:\n{}",
        output.code
    );
    assert!(
        !output.code.contains("__vize_setup_fn"),
        "`<script setup>` output must not emit the Options API setup \
         wrapper. Got:\n{}",
        output.code
    );
}

/// End-to-end shape check: Options API SFC **with a `<template>`** that
/// references identifiers resolved via `this` (`{{ status }}`,
/// `{{ store.ready }}`). The virtual TS needs to make those identifiers
/// available inside the template scope; otherwise tsgo reports TS2304
/// (`Cannot find name '<x>'`) for each interpolation.
#[test]
#[ignore = "documents the missing Options API + template scope bridge (run with `cargo test -- --ignored`)"]
fn options_api_with_template_contains_each_required_marker() {
    use vize_carton::Bump;
    let script = r#"import { defineComponent } from 'vue'
function useFakeStore() {
    return { ready: false }
}
export default defineComponent({
    setup() {
        const store = useFakeStore()
        return { store }
    },
    data() { return { count: 0 } },
    computed: {
        greeting() { return `count=${this.count}` },
    },
    methods: {
        bump(step: number) { this.count = this.count + step },
    },
})
"#;
    let template = r#"<div>{{ greeting }} {{ store.ready }} {{ count }}</div>"#;

    let allocator = Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &Default::default(),
    );
    let code = &output.code;

    // (1) Typed `this` alias — without it `this.count` / `this.store.ready`
    //     inside computed / methods bodies fall back to `any` and trigger
    //     TS2339 / TS7023.
    assert!(
        code.contains("type __VizeThis ="),
        "expected a typed `this` alias (e.g. `type __VizeThis = ...`) so \
         method/computed bodies can resolve `this.<x>`. Got:\n{}",
        code
    );

    // (2) Typed wrappers carrying the user's bodies — without them tsgo
    //     never sees the methods/computed with a `this` parameter and
    //     reports TS7023 on every return.
    assert!(
        code.contains("__vize_method_bump") || code.contains("__vize_method_"),
        "expected a typed method wrapper (e.g. `function \
         __vize_method_bump(this: __VizeThis ...)`) so the bodies are \
         actually type-checked against `this`. Got:\n{}",
        code
    );
    assert!(
        code.contains("__vize_computed_greeting") || code.contains("__vize_computed_"),
        "expected a typed computed wrapper. Got:\n{}",
        code
    );

    // (3) Template-scope bindings — for every statically-known `this.<x>`
    //     key, the template IIFE needs an in-scope binding (any shape
    //     that satisfies the type checker is fine; the existence is what
    //     matters). Without these, `{{ greeting }}` / `{{ store }}` /
    //     `{{ count }}` raise TS2304 inside the template.
    for name in ["greeting", "store", "count"] {
        let mentions_in_scope =
            code.contains(&format!(" {name}:")) || code.contains(&format!(" {name} "));
        assert!(
            mentions_in_scope,
            "expected the template scope to declare `{name}` as a typed \
             local pulled from the `this` alias (e.g. \
             `var {name}: __VizeThis[\"{name}\"] = ...;` or any \
             equivalent shape). Otherwise `{{{{ {name} }}}}` raises TS2304 \
             inside the template. Got:\n{code}",
        );
    }
}
