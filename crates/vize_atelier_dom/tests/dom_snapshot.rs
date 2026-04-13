//! DOM compiler snapshot tests.
//!
//! These tests compare the DOM compiler output against expected snapshots.

use vize_atelier_dom::compile_template;
use vize_carton::Bump;

/// Helper to get the compiled code
fn get_compiled(src: &str) -> String {
    let allocator = Bump::new();
    let (_, errors, result) = compile_template(&allocator, src);

    if !errors.is_empty() {
        panic!("Compilation errors: {:?}", errors);
    }

    format!("{}\n{}", result.preamble, result.code)
}

// =============================================================================
// Static Element Tests
// =============================================================================

mod static_element {
    use super::get_compiled;

    #[test]
    fn simple_div() {
        insta::assert_snapshot!(get_compiled("<div></div>"));
    }

    #[test]
    fn div_with_text() {
        insta::assert_snapshot!(get_compiled("<div>hello</div>"));
    }

    #[test]
    fn nested_elements() {
        insta::assert_snapshot!(get_compiled("<div><span>hello</span></div>"));
    }
}

// =============================================================================
// Interpolation Tests
// =============================================================================

mod interpolation {
    use super::get_compiled;

    #[test]
    fn simple_interpolation() {
        insta::assert_snapshot!(get_compiled("{{ msg }}"));
    }

    #[test]
    fn interpolation_in_element() {
        insta::assert_snapshot!(get_compiled("<div>{{ msg }}</div>"));
    }
}

// =============================================================================
// v-if Tests
// =============================================================================

mod v_if {
    use super::get_compiled;

    #[test]
    fn simple_v_if() {
        insta::assert_snapshot!(get_compiled(r#"<div v-if="ok">hello</div>"#));
    }

    #[test]
    fn v_if_v_else() {
        insta::assert_snapshot!(get_compiled(
            r#"<div v-if="ok">yes</div><div v-else>no</div>"#
        ));
    }

    #[test]
    fn v_if_component_with_slot() {
        insta::assert_snapshot!(get_compiled(
            r#"<MyComponent v-if="ok"><span>slot content</span></MyComponent>"#
        ));
    }

    #[test]
    fn v_if_component_with_named_slot() {
        insta::assert_snapshot!(get_compiled(
            r#"<MyComponent v-if="ok"><template #header><h1>title</h1></template></MyComponent>"#
        ));
    }
}

// =============================================================================
// v-for Tests
// =============================================================================

mod v_for {
    use super::get_compiled;

    #[test]
    fn simple_v_for() {
        insta::assert_snapshot!(get_compiled(
            r#"<div v-for="item in items">{{ item }}</div>"#
        ));
    }
}

// =============================================================================
// v-bind Tests
// =============================================================================

mod v_bind {
    use super::get_compiled;

    #[test]
    fn dynamic_id() {
        insta::assert_snapshot!(get_compiled(r#"<div :id="foo"></div>"#));
    }

    #[test]
    fn dynamic_class() {
        insta::assert_snapshot!(get_compiled(r#"<div :class="cls"></div>"#));
    }

    #[test]
    fn merge_static_and_dynamic_class_with_vbind_object() {
        insta::assert_snapshot!(get_compiled(
            r#"<input v-bind="attrs" class="base" :class="stateClass" />"#
        ));
    }

    #[test]
    fn merge_static_and_dynamic_style_with_vbind_object() {
        insta::assert_snapshot!(get_compiled(
            r#"<input v-bind="attrs" style="color: red" :style="dynamicStyle" />"#
        ));
    }
}

// =============================================================================
// v-on Tests
// =============================================================================

mod v_on {
    use super::get_compiled;

    #[test]
    fn click_handler() {
        insta::assert_snapshot!(get_compiled(r#"<div @click="handler"></div>"#));
    }
}

// =============================================================================
// v-model Tests
// =============================================================================

mod v_model {
    use super::get_compiled;

    #[test]
    fn input_text() {
        insta::assert_snapshot!(get_compiled(r#"<input v-model="msg" />"#));
    }
}

// =============================================================================
// v-show Tests
// =============================================================================

mod v_show {
    use super::get_compiled;

    #[test]
    fn simple_v_show() {
        insta::assert_snapshot!(get_compiled(r#"<div v-show="visible">content</div>"#));
    }

    #[test]
    fn v_show_on_child_component() {
        insta::assert_snapshot!(get_compiled(
            r#"<div><MyComponent v-show="visible" /></div>"#
        ));
    }

    #[test]
    fn v_show_on_root_component() {
        insta::assert_snapshot!(get_compiled(r#"<MyComponent v-show="visible" />"#));
    }
}

// =============================================================================
// Component Tests
// =============================================================================

mod component {
    use super::get_compiled;

    #[test]
    fn simple_component() {
        insta::assert_snapshot!(get_compiled("<MyComponent></MyComponent>"));
    }

    #[test]
    fn dynamic_component_uses_block_in_non_block_context() {
        let code = get_compiled(r#"<div><component :is="current" /></div>"#);
        assert!(
            code.contains("(_openBlock(), _createBlock(_resolveDynamicComponent("),
            "dynamic component should use block form in nested context:\n{code}"
        );
    }

    #[test]
    fn forwarded_slot_flag_is_emitted() {
        let code = get_compiled(r#"<component :is="current"><slot /></component>"#);
        assert!(
            code.contains("_: 3 /* FORWARDED */"),
            "forwarded slot should use FORWARDED slot flag:\n{code}"
        );
    }

    #[test]
    fn dynamic_slot_still_uses_dynamic_flag_even_with_slot_forwarding() {
        let code = get_compiled(r#"<Comp><template #[name]><slot /></template></Comp>"#);
        assert!(
            code.contains("_: 2 /* DYNAMIC */"),
            "dynamic slots should keep DYNAMIC slot flag:\n{code}"
        );
    }

    #[test]
    fn sibling_v_if_groups_get_unique_auto_keys() {
        let code = get_compiled(r#"<div><span v-if="a">A</span><span v-if="b">B</span></div>"#);
        assert!(
            code.contains("{ key: 0 }"),
            "first v-if group should use key 0:\n{code}"
        );
        assert!(
            code.contains("{ key: 1 }"),
            "second v-if group should use key 1:\n{code}"
        );
    }

    #[test]
    fn slot_outlet_in_v_if_branch_uses_render_slot() {
        let code = get_compiled(
            r#"<button><slot v-if="ok" name="icon" /><slot v-else /></button>"#,
        );
        assert!(
            code.contains("_renderSlot(_ctx.$slots, \"icon\", { key: 0"),
            "slot v-if branch should compile to renderSlot:\n{code}"
        );
        assert!(
            code.contains("_renderSlot(_ctx.$slots, \"default\", { key: 1"),
            "slot v-else branch should compile to renderSlot:\n{code}"
        );
    }
}
