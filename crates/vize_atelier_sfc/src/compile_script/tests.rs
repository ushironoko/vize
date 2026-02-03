//! Tests for script compilation.

#[cfg(test)]
mod compile_script_tests {
    use crate::compile_script::compile_script;
    use crate::compile_script::function_mode::compile_script_setup;
    use crate::compile_script::props::{
        extract_prop_types_from_type, extract_with_defaults_defaults, is_valid_identifier,
    };
    use crate::compile_script::typescript::transform_typescript_to_js;
    use crate::types::SfcDescriptor;

    #[test]
    fn test_compile_empty_script() {
        let descriptor = SfcDescriptor::default();
        let result =
            compile_script(&descriptor, &Default::default(), "Test", false, false).unwrap();
        assert!(result.code.contains("__sfc__"));
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("foo"));
        assert!(is_valid_identifier("_bar"));
        assert!(is_valid_identifier("$baz"));
        assert!(is_valid_identifier("foo123"));
        assert!(!is_valid_identifier("123foo"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("foo-bar"));
    }

    #[test]
    fn test_extract_with_defaults_defaults() {
        // Test simple case
        let input = r#"withDefaults(defineProps<{ msg?: string }>(), { msg: "hello" })"#;
        let defaults = extract_with_defaults_defaults(input);
        eprintln!("Defaults: {:?}", defaults);
        assert_eq!(defaults.get("msg"), Some(&r#""hello""#.to_string()));

        // Test multiple defaults
        let input2 = r#"withDefaults(defineProps<{ msg?: string, count?: number }>(), { msg: "hello", count: 42 })"#;
        let defaults2 = extract_with_defaults_defaults(input2);
        assert_eq!(defaults2.get("msg"), Some(&r#""hello""#.to_string()));
        assert_eq!(defaults2.get("count"), Some(&"42".to_string()));

        // Test multiline input like AfCheckbox
        let input3 = r#"withDefaults(
  defineProps<{
    checked: boolean;
    label?: string;
    color?: "primary" | "secondary";
  }>(),
  {
    label: undefined,
    color: "primary",
  },
)"#;
        let defaults3 = extract_with_defaults_defaults(input3);
        eprintln!("Defaults3: {:?}", defaults3);
        assert_eq!(defaults3.get("label"), Some(&"undefined".to_string()));
        assert_eq!(defaults3.get("color"), Some(&r#""primary""#.to_string()));
    }

    #[test]
    fn test_compile_script_setup_with_define_props() {
        let content = r#"
import { ref } from 'vue'
const props = defineProps(['msg'])
const count = ref(0)
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        println!("Compiled output:\n{}", result.code);

        // Should have __sfc__
        assert!(
            result.code.contains("const __sfc__ ="),
            "Should have __sfc__"
        );
        // Should have __name (may use single or double quotes after OXC formatting)
        assert!(
            result.code.contains("__name:") && result.code.contains("Test"),
            "Should have __name. Got:\n{}",
            result.code
        );
        // Should have props definition (may use double quotes after OXC formatting)
        assert!(
            result.code.contains("props:") && result.code.contains("msg"),
            "Should have props definition. Got:\n{}",
            result.code
        );
        // Should have setup function with proper signature
        assert!(
            result
                .code
                .contains("setup(__props, { expose: __expose, emit: __emit })"),
            "Should have proper setup signature"
        );
        // __expose is only called if defineExpose is used (not in this test)
        // Should have __returned__
        assert!(
            result.code.contains("const __returned__ =")
                || result.code.contains("__returned__ = {"),
            "Should have __returned__"
        );
    }

    #[test]
    fn test_compile_script_setup_with_define_emits() {
        let content = r#"
const emit = defineEmits(['click', 'update'])
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        println!("Full output:\n{}", result.code);

        assert!(
            result.code.contains("emits:"),
            "Should contain emits definition"
        );
        assert!(
            result.code.contains("const emit = __emit"),
            "Should bind emit to __emit"
        );
        // emit should be in __returned__ as it's a runtime value used in templates
        assert!(
            result.code.contains("emit"),
            "emit should be accessible in template"
        );
        // defineEmits should NOT be in the setup function
        assert!(
            !result.code.contains("defineEmits"),
            "defineEmits should be removed from setup"
        );
    }

    #[test]
    fn test_compile_script_setup_with_define_emits_usage() {
        let content = r#"
import { ref } from 'vue'
const emit = defineEmits(['click', 'update'])
const count = ref(0)
function onClick() {
    emit('click', count.value)
}
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        println!("Compiled output:\n{}", result.code);

        // defineEmits call should NOT be in the setup function
        assert!(
            !result.code.contains("defineEmits"),
            "defineEmits call should be removed from setup"
        );
        // emit binding should be present
        assert!(
            result.code.contains("const emit = __emit"),
            "Should bind emit to __emit"
        );
        // onClick function should be in setup
        assert!(
            result.code.contains("function onClick()"),
            "onClick should be in setup"
        );
        // emits definition should be present (may be formatted differently by OXC)
        assert!(
            result.code.contains("emits:")
                && result.code.contains("click")
                && result.code.contains("update"),
            "Should have emits definition. Got:\n{}",
            result.code
        );
    }

    #[test]
    fn test_compile_script_setup_without_macros() {
        let content = r#"
import { ref } from 'vue'
const msg = ref('hello')
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        // Should have setup
        assert!(result.code.contains("setup(__props"), "Should have setup");
        // Should NOT have props or emits definitions
        assert!(
            !result.code.contains("  props:"),
            "Should not contain props"
        );
        assert!(!result.code.contains("emits:"), "Should not contain emits");
    }

    #[test]
    fn test_compile_script_setup_with_props_destructure() {
        let content = r#"
import { computed } from 'vue'
const { count } = defineProps({ count: Number })
const double = computed(() => count * 2)
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        println!("Compiled output:\n{}", result.code);

        // Should transform count to __props.count inside computed
        assert!(
            result.code.contains("__props.count"),
            "Should transform destructured prop to __props.count"
        );
        // The original `count` reference should be replaced
        assert!(
            result.code.contains("computed(() => __props.count * 2)"),
            "Should have transformed computed expression"
        );
        // Destructured props should NOT be in __returned__
        assert!(
            !result.code.contains("__returned__ = { computed, count,"),
            "Destructured props should not be in __returned__"
        );
        // Should have double and computed in __returned__
        assert!(
            result.code.contains("computed") && result.code.contains("double"),
            "Should have computed and double in __returned__"
        );
    }

    #[test]
    fn test_compiler_macros_not_in_returned() {
        let content = r#"
import { defineProps, ref } from 'vue'
const props = defineProps(['msg'])
const count = ref(0)
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        println!("Compiled output:\n{}", result.code);

        // Find the __returned__ block (may span multiple lines after OXC formatting)
        let code = &result.code;
        let returned_start = code.find("__returned__").expect("Should have __returned__");
        let returned_block = &code[returned_start..];
        let block_end = returned_block.find(';').unwrap_or(returned_block.len());
        let returned_content = &returned_block[..block_end];

        println!("__returned__ block: {}", returned_content);

        // Compiler macros should NOT be in __returned__
        assert!(
            !returned_content.contains("defineProps"),
            "Compiler macros should not be in __returned__"
        );
        // But regular imports should be
        assert!(
            returned_content.contains("ref"),
            "Regular imports should be in __returned__"
        );
    }

    #[test]
    fn test_props_destructure_with_defaults() {
        let content = r#"
import { computed, watch } from 'vue'

const {
  name,
  count = 0,
  disabled = false,
  items = () => []
} = defineProps<{
  name: string
  count?: number
  disabled?: boolean
  items?: string[]
}>()

const doubled = computed(() => count * 2)
const itemCount = computed(() => items.length)
"#;

        // First check context analysis
        let mut ctx = crate::script::ScriptCompileContext::new(content);
        ctx.analyze();

        println!("=== Context Analysis ===");
        println!("props_destructure: {:?}", ctx.macros.props_destructure);
        println!("bindings: {:?}", ctx.bindings.bindings);

        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        println!("\n=== Compiled output ===\n{}", result.code);

        // Should NOT contain the destructure statement
        assert!(
            !result.code.contains("const {"),
            "Should not contain destructure statement"
        );
        assert!(
            !result.code.contains("} = defineProps"),
            "Should not contain defineProps assignment"
        );

        // Should have props definition with defaults
        assert!(
            result.code.contains("props:"),
            "Should have props definition"
        );

        // Should transform props to __props
        assert!(
            result.code.contains("__props.count"),
            "Should transform count to __props.count"
        );
        assert!(
            result.code.contains("__props.items"),
            "Should transform items to __props.items"
        );

        // Should have the computed expressions transformed
        assert!(
            result.code.contains("computed(() => __props.count * 2)"),
            "Should transform count in computed. Got:\n{}",
            result.code
        );
    }

    #[test]
    fn test_extract_prop_types() {
        let type_args = r#"{
  name: string
  count?: number
  disabled?: boolean
  items?: string[]
}"#;
        let props = extract_prop_types_from_type(type_args);
        assert!(props.contains_key("name"), "Should extract name");
        assert!(props.contains_key("count"), "Should extract count");
        assert!(props.contains_key("disabled"), "Should extract disabled");
        assert!(props.contains_key("items"), "Should extract items");

        // Check types
        assert_eq!(props.get("name").unwrap().js_type, "String");
        assert_eq!(props.get("count").unwrap().js_type, "Number");
        assert_eq!(props.get("disabled").unwrap().js_type, "Boolean");
        assert_eq!(props.get("items").unwrap().js_type, "Array");

        // Check optionality
        assert!(!props.get("name").unwrap().optional);
        assert!(props.get("count").unwrap().optional);
        assert!(props.get("disabled").unwrap().optional);
        assert!(props.get("items").unwrap().optional);
    }

    #[test]
    fn test_compile_script_setup_with_multiline_define_emits() {
        let content = r#"
const emit = defineEmits<{
  (e: 'click', payload: MouseEvent): void
  (e: 'update', value: string): void
}>()

function handleClick(e: MouseEvent) {
    emit('click', e)
}
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        println!("Multi-line defineEmits output:\n{}", result.code);

        // defineEmits should NOT be in the setup function
        assert!(
            !result.code.contains("defineEmits"),
            "defineEmits should be removed from setup"
        );
        // emit binding should be present
        assert!(
            result.code.contains("const emit = __emit"),
            "Should bind emit to __emit"
        );
        // handleClick function should be in setup
        assert!(
            result.code.contains("function handleClick"),
            "handleClick should be in setup"
        );
        // emits definition should be present
        assert!(
            result.code.contains("emits:"),
            "Should have emits definition"
        );
    }

    #[test]
    fn test_compile_script_setup_with_typed_define_emits_single_line() {
        let content = r#"
const emit = defineEmits<(e: 'click') => void>()
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        println!("Typed defineEmits output:\n{}", result.code);

        // defineEmits should NOT be in the setup function
        assert!(
            !result.code.contains("defineEmits"),
            "defineEmits should be removed from setup"
        );
        // emit binding should be present
        assert!(
            result.code.contains("const emit = __emit"),
            "Should bind emit to __emit"
        );
    }

    #[test]
    fn test_compile_script_setup_with_define_expose() {
        let content = r#"
import { ref } from 'vue'
const count = ref(0)
const reset = () => count.value = 0
defineExpose({ count, reset })
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        println!("defineExpose output:\n{}", result.code);

        // defineExpose should be transformed to __expose(...)
        assert!(
            result.code.contains("__expose({"),
            "Should have __expose call with arguments"
        );
        assert!(
            result.code.contains("count"),
            "__expose should include count"
        );
        assert!(
            result.code.contains("reset"),
            "__expose should include reset"
        );
        // defineExpose should NOT be in the setup function
        assert!(
            !result.code.contains("defineExpose"),
            "defineExpose should be removed from setup"
        );
    }

    #[test]
    fn test_compile_script_setup_without_define_expose() {
        // Test that __expose() is always called, even without defineExpose.
        // This matches the official Vue compiler behavior, which is required for
        // proper component initialization with @vue/test-utils.
        let content = r#"
import { ref } from 'vue'
const count = ref(0)
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        println!("Output without defineExpose:\n{}", result.code);

        // __expose() should always be called for proper Vue runtime initialization
        assert!(
            result.code.contains("__expose()"),
            "Should have __expose() call even without defineExpose. Got:\n{}",
            result.code
        );
    }

    #[test]
    fn test_compile_script_setup_with_empty_define_expose() {
        // Test that defineExpose() (empty) is handled correctly
        let content = r#"
import { ref } from 'vue'
const count = ref(0)
defineExpose()
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();

        println!("Output with empty defineExpose:\n{}", result.code);

        // Should have __expose() call
        assert!(
            result.code.contains("__expose()"),
            "Should have __expose() call for empty defineExpose. Got:\n{}",
            result.code
        );

        // defineExpose should be removed
        assert!(
            !result.code.contains("defineExpose"),
            "defineExpose should be removed from setup"
        );
    }

    #[test]
    fn test_transform_typescript_to_js_strips_types() {
        let ts_code = r#"const getNumber = (x: number): string => {
    return x.toString();
}
const foo: string = "bar";"#;
        let result = transform_typescript_to_js(ts_code);
        eprintln!("TypeScript transform result:\n{}", result);

        // Should NOT contain type annotations
        assert!(
            !result.contains(": number"),
            "Should strip parameter type annotation. Got:\n{}",
            result
        );
        assert!(
            !result.contains(": string"),
            "Should strip return type and variable type annotations. Got:\n{}",
            result
        );
    }

    #[test]
    fn test_compile_script_setup_strips_typescript() {
        let content = r#"
const getNumberOfTeachers = (
  items: Item[]
): string => {
  return items.length.toString();
};
"#;
        // is_ts = false means we want JavaScript output (TypeScript should be stripped)
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();
        eprintln!("Compiled TypeScript output:\n{}", result.code);

        // Should NOT contain type annotations
        assert!(
            !result.code.contains(": Item[]"),
            "Should strip parameter type annotation. Got:\n{}",
            result.code
        );
        assert!(
            !result.code.contains("): string"),
            "Should strip return type annotation. Got:\n{}",
            result.code
        );
    }

    // =============================================================================
    // Patch Tests: Props destructure with type-based defineProps
    // =============================================================================

    #[test]
    fn test_props_destructure_type_based_with_defaults() {
        // Issue: _mergeDefaults first argument was empty for type-based props
        let content = r#"
const { color = "primary", size = "medium" } = defineProps<{
  color?: "primary" | "secondary";
  size?: "small" | "medium" | "large";
}>();
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();
        println!("Type-based props destructure output:\n{}", result.code);

        // Should have _mergeDefaults with proper first argument
        assert!(
            result.code.contains("_mergeDefaults("),
            "Should use _mergeDefaults"
        );
        // First argument should NOT be empty
        assert!(
            !result.code.contains("_mergeDefaults(, {"),
            "First argument should not be empty. Got:\n{}",
            result.code
        );
        // Should contain prop definitions
        assert!(
            result.code.contains("color") && result.code.contains("size"),
            "Should have prop definitions"
        );
    }

    #[test]
    fn test_props_destructure_type_based_generates_runtime_props() {
        let content = r#"
const { title, count = 0 } = defineProps<{
  title: string;
  count?: number;
}>();
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();
        println!("Runtime props generation:\n{}", result.code);

        // Should generate runtime props object with type and required fields
        assert!(
            result.code.contains("type:") || result.code.contains("String") || result.code.contains("Number"),
            "Should generate runtime type information"
        );
    }

    // =============================================================================
    // Patch Tests: Duplicate import filtering
    // =============================================================================

    #[test]
    fn test_duplicate_imports_filtered() {
        let content = r#"
import { ref } from 'vue'
import { ref } from 'vue'
import { computed } from 'vue'
const count = ref(0)
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();
        println!("Duplicate imports output:\n{}", result.code);

        // Count occurrences of "import { ref }"
        let ref_imports = result
            .code
            .matches("import { ref }")
            .count();
        assert!(
            ref_imports <= 1,
            "Should have at most 1 ref import, got {}. Output:\n{}",
            ref_imports,
            result.code
        );
    }

    // =============================================================================
    // Patch Tests: Async setup detection
    // =============================================================================

    #[test]
    fn test_top_level_await_generates_async_setup() {
        let content = r#"
const data = await fetch('/api').then(r => r.json())
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();
        println!("Async setup output:\n{}", result.code);

        assert!(
            result.code.contains("async setup("),
            "Should generate async setup for top-level await. Got:\n{}",
            result.code
        );
    }

    #[test]
    fn test_await_in_string_does_not_trigger_async() {
        let content = r#"
const message = "please await for response"
const code = 'await is a keyword'
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();
        println!("String await output:\n{}", result.code);

        assert!(
            !result.code.contains("async setup("),
            "Should NOT generate async setup for await in string. Got:\n{}",
            result.code
        );
    }

    // =============================================================================
    // Patch Tests: Type comparison detection
    // =============================================================================

    #[test]
    fn test_type_comparison_not_stripped() {
        let content = r#"
const props = defineProps(['type'])
const isButton = props.type === 'button'
const isLink = props.type !== 'link'
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();
        println!("Type comparison output:\n{}", result.code);

        // The comparison expressions should still be present
        assert!(
            result.code.contains("===") || result.code.contains("!=="),
            "Type comparison operators should be preserved. Got:\n{}",
            result.code
        );
        // The variable assignments should be present
        assert!(
            result.code.contains("isButton") && result.code.contains("isLink"),
            "Variable assignments should be preserved. Got:\n{}",
            result.code
        );
    }

    // =============================================================================
    // Patch Tests: TypeScript generic stripping
    // =============================================================================

    #[test]
    fn test_generic_function_call_stripped() {
        let ts_code = r#"const store = useStore<RootState>()"#;
        let result = transform_typescript_to_js(ts_code);
        println!("Generic stripped result: {}", result);

        assert!(
            !result.contains("<RootState>"),
            "Generic type argument should be stripped. Got: {}",
            result
        );
        assert!(
            result.contains("useStore()"),
            "Function call should remain. Got: {}",
            result
        );
    }

    #[test]
    fn test_ref_with_generic_stripped() {
        let ts_code = r#"const user = ref<User | null>(null)"#;
        let result = transform_typescript_to_js(ts_code);
        println!("Ref generic stripped result: {}", result);

        assert!(
            !result.contains("<User | null>"),
            "Generic type argument should be stripped. Got: {}",
            result
        );
        assert!(
            result.contains("ref(null)"),
            "Function call should remain. Got: {}",
            result
        );
    }

    // =============================================================================
    // Patch Tests: Arrow function type annotation stripping
    // =============================================================================

    #[test]
    fn test_arrow_function_param_types_stripped() {
        let ts_code = r#"items.filter((x: number) => x > 1)"#;
        let result = transform_typescript_to_js(ts_code);
        println!("Arrow param type stripped: {}", result);

        assert!(
            !result.contains(": number"),
            "Parameter type should be stripped. Got: {}",
            result
        );
    }

    // =============================================================================
    // Patch Tests: JavaScript reserved words
    // =============================================================================

    #[test]
    fn test_reserved_word_not_in_shorthand() {
        // This test verifies that reserved words like 'default' are not used as shorthand
        let content = r#"
import DefaultComponent from './Default.vue'
const result = { default: DefaultComponent }
"#;
        let result = compile_script_setup(content, "Test", false, false, None).unwrap();
        println!("Reserved word output:\n{}", result.code);

        // The code should compile without syntax errors
        // (If reserved words were used as shorthand, it would fail)
        assert!(
            result.code.contains("__returned__"),
            "Should successfully compile with reserved word prop"
        );
    }
}
