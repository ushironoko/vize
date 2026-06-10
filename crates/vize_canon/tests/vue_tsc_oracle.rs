//! Vue-tsc oracle — failing integration tests describing the Options API
//! type-checking gap in `BatchTypeChecker`.
//!
//! Each fixture below feeds a representative Options API SFC through the
//! full `BatchTypeChecker` (which runs `tsgo` over the generated virtual
//! TS). They currently fail because the virtual TS emits the user's
//! `export default { ... }` body verbatim — `this`-resolved identifiers
//! therefore lose their types and surface as:
//!
//!  - **TS2339** on every `this.<x>` access inside `methods` / `computed`,
//!  - **TS7023** on every method/computed return that depends on `this`,
//!  - **TS2304** on template identifiers (`{{ greeting }}`, etc.) that
//!    resolve at runtime through the component instance,
//!  - **TS2724** against `import { defineComponent } from 'vue'` once the
//!    generator no longer emits a `defineComponent({...})` call — tsgo
//!    fails to walk vue's `export *` re-export chain without it.
//!
//! All fixtures avoid embedding-project vocabulary: only generic names
//! (`useFakeStore`, `count`, `message`, `bump`, `greeting`, etc.) appear.
//!
//! ## Why `#[ignore]`?
//!
//! Each case spawns tsgo against a real project and links the
//! workspace's Vue / Pinia / tsgo install. CI environments without
//! `node_modules` and tsgo cannot run these. Invoke explicitly with:
//!
//!     cargo test -p vize_canon --test vue_tsc_oracle -- --ignored

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use vize_canon::batch::{BatchTypeChecker, TypeChecker};

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist");
    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    workspace_root
        .join("__agent_only")
        .join("tests")
        .join(format!("oracle-{name}-{}-{case_id}", std::process::id()))
}

fn write_project_tsconfig(project_root: &Path) {
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
}

/// Symlink the workspace `node_modules` so vue / pinia / tsgo resolve.
/// Returns `Err` when the workspace install is unavailable — the test
/// silently skips in that case, matching the existing integration-test
/// behavior.
fn link_workspace_node_modules(project_root: &Path) -> std::io::Result<()> {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| std::io::Error::other("workspace root not found"))?;
    let workspace_node_modules = workspace_root.join("node_modules");
    if !workspace_node_modules.exists() {
        return Err(std::io::Error::other("workspace node_modules missing"));
    }
    let target = project_root.join("node_modules");
    if target.exists() {
        std::fs::remove_dir_all(&target).ok();
    }
    std::os::unix::fs::symlink(&workspace_node_modules, &target)
}

fn create_project(name: &str, files: &[(&str, &str)]) -> Option<PathBuf> {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).ok()?;
    link_workspace_node_modules(&project_root).ok()?;
    write_project_tsconfig(&project_root);
    for (path, src) in files {
        let p = project_root.join(path);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).ok()?;
        }
        std::fs::write(p, src).ok()?;
    }
    Some(project_root)
}

fn ts2339_in(project_root: &Path) -> Vec<(String, String)> {
    diagnostics_in(project_root, &[2339])
}

/// Collect every diagnostic in `project_root` whose code is in `codes`.
/// Returns `(relative path, message)` pairs. Used for tests that need to
/// assert on multiple categories at once (e.g. "no TS2304 and no TS2724").
fn diagnostics_in(project_root: &Path, codes: &[u32]) -> Vec<(String, String)> {
    let mut checker = match BatchTypeChecker::new(project_root) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    checker.scan_project().ok();
    let result = match checker.check_project() {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    result
        .diagnostics
        .into_iter()
        .filter(|d| {
            d.code
                .map(|c| codes.contains(&(c as u32)))
                .unwrap_or(false)
        })
        .map(|d| {
            let path = d
                .file
                .strip_prefix(project_root)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| d.file.display().to_string());
            (path, d.message.to_string())
        })
        .collect()
}

/// Representative shape: Options API + `setup()` returning a store
/// handle + `data()` + `computed` cross-referencing the setup return +
/// `methods` cross-referencing `data` and `computed`. Every `this.<x>`
/// access here currently raises TS2339 / TS7023 because the virtual TS
/// re-emits the user's options object verbatim and so loses the
/// component's `this` shape.
#[test]
#[ignore = "runs tsgo against a real project; opt-in with -- --ignored"]
fn oracle_options_api_resolves_this_across_setup_data_computed_methods() {
    let Some(project_root) = create_project(
        "this-across-blocks",
        &[(
            "src/Component.vue",
            r#"<script lang="ts">
import { defineComponent } from 'vue'

function useFakeStore() {
    return {
        conditions: { ready: true },
        rects: [] as Array<{ x: number; y: number }>,
    }
}

export default defineComponent({
    setup() {
        const store = useFakeStore()
        return { store }
    },
    data() {
        return { localCount: 0, label: 'init' }
    },
    computed: {
        conditions() {
            return this.store.conditions
        },
        coords() {
            return `${this.label}-${this.localCount}`
        },
    },
    methods: {
        increment(step: number) {
            this.localCount = this.localCount + step
        },
        announce() {
            return this.coords + (this.conditions.ready ? '!' : '?')
        },
    },
})
</script>

<template>
  <div />
</template>
"#,
        )],
    ) else {
        // Workspace install unavailable — silently skip.
        return;
    };

    let ts2339 = ts2339_in(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);

    assert!(
        ts2339.is_empty(),
        "the Options API bridge regressed — vue-tsc would accept these `this.*` \
         references but vize reported TS2339:\n{ts2339:#?}"
    );
}

/// Representative shape: pinia helpers spread into `computed` / `methods`.
///
/// `...mapState(useStore, ['key1', 'key2'])` keys must be resolvable
/// from `ReturnType<typeof useStore>` and visible on `this`.
#[test]
#[ignore = "runs tsgo against a real project; opt-in with -- --ignored"]
fn oracle_options_api_pinia_spread_resolves_keys_on_this() {
    let Some(project_root) = create_project(
        "pinia-spread",
        &[(
            "src/Component.vue",
            r#"<script lang="ts">
import { defineComponent } from 'vue'

function useFakeStore() {
    return {
        rects: [] as Array<{ x: number; y: number }>,
        conditions: { ready: true },
        setConditions(_v: { ready: boolean }) {},
    }
}

function mapState<K extends keyof ReturnType<typeof useFakeStore>>(
    _store: typeof useFakeStore,
    _keys: K[],
): { [P in K]: ReturnType<typeof useFakeStore>[P] } {
    return {} as { [P in K]: ReturnType<typeof useFakeStore>[P] }
}
function mapActions<K extends keyof ReturnType<typeof useFakeStore>>(
    _store: typeof useFakeStore,
    _keys: K[],
): { [P in K]: ReturnType<typeof useFakeStore>[P] } {
    return {} as { [P in K]: ReturnType<typeof useFakeStore>[P] }
}

export default defineComponent({
    computed: {
        ...mapState(useFakeStore, ['rects', 'conditions']),
        summary() {
            return this.rects.length + (this.conditions.ready ? 1 : 0)
        },
    },
    methods: {
        ...mapActions(useFakeStore, ['setConditions']),
        reset() {
            this.setConditions({ ready: false })
        },
    },
})
</script>

<template>
  <div />
</template>
"#,
        )],
    ) else {
        return;
    };

    let ts2339 = ts2339_in(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);

    assert!(
        ts2339.is_empty(),
        "the pinia spread bridge regressed — TS2339 reported on keys that should \
         resolve via mapped types:\n{ts2339:#?}"
    );
}

/// End-to-end oracle: an Options API SFC with a `<template>` reading
/// `this`-resolved identifiers (`{{ greeting }}`, `{{ store.ready }}`).
///
/// On the current virtual TS this fixture produces **four distinct
/// error categories** under tsgo:
/// - TS2339 / TS7023 inside `methods` / `computed` bodies — `this.<x>`
///   resolves to `any` because the options object is emitted verbatim
///   and the component's `this` shape is not modelled.
/// - TS2304 inside the template — template identifiers are not in
///   scope for the template IIFE.
/// - TS2724 against `import { defineComponent } from 'vue'` — surfaces
///   once any future fix skips the `defineComponent({...})` call site,
///   because tsgo does not walk vue's `export * from
///   '@vue/runtime-dom'` re-export chain unless that call expression
///   itself triggers the lookup.
///
/// All names in the fixture are deliberately generic.
#[test]
#[ignore = "runs tsgo against a real project; opt-in with -- --ignored"]
fn oracle_options_api_with_template_resolves_this_and_template_bindings() {
    let Some(project_root) = create_project(
        "options-api-with-template",
        &[(
            "src/Component.vue",
            r#"<script lang="ts">
import { defineComponent } from 'vue'

function useFakeStore() {
    return {
        items: [] as Array<{ id: number; label: string }>,
        ready: false,
    }
}

export default defineComponent({
    setup() {
        const store = useFakeStore()
        return { store }
    },
    data() {
        return { count: 0, message: 'init' }
    },
    computed: {
        greeting() {
            return `${this.message}: ${this.count}`
        },
    },
    methods: {
        bump(step: number) {
            this.count = this.count + step
            return this.greeting
        },
    },
})
</script>

<template>
  <div>{{ greeting }} ({{ store.ready }})</div>
</template>
"#,
        )],
    ) else {
        return;
    };

    // Look at every diagnostic category produced by this gap:
    // 2339 (property does not exist), 7023 (implicit return any),
    // 2304 (name not found in template), 2724 (no exported member —
    // surfaces when the vue `export *` chain is not fully resolved).
    let diags = diagnostics_in(&project_root, &[2339, 7023, 2304, 2724]);
    let _ = std::fs::remove_dir_all(&project_root);

    assert!(
        diags.is_empty(),
        "Options API + template type checking is incomplete — at least one \
         of TS2339 / TS7023 / TS2304 / TS2724 fired on a fixture that \
         vue-tsc accepts. Diagnostics:\n{diags:#?}"
    );
}
