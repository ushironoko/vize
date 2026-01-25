/**
 * TextInput Component - Text input with builtin cursor management and IME support
 */

import { defineComponent, h, ref, watch, computed, type PropType } from '@vue/runtime-core';
import { useInput } from '../composables/useInput.js';

export interface TextInputProps {
  /** Input value (v-model) */
  modelValue?: string;
  /** Placeholder text */
  placeholder?: string;
  /** Whether input is focused */
  focus?: boolean;
  /** Password mode (mask input) */
  mask?: boolean;
  /** Mask character */
  maskChar?: string;
  /** Input width */
  width?: number | string;
  /** Foreground color */
  fg?: string;
  /** Background color */
  bg?: string;
  /** Called when value changes */
  'onUpdate:modelValue'?: (value: string) => void;
  /** Called when submitted (Enter) */
  onSubmit?: (value: string) => void;
  /** Called when escape is pressed */
  onCancel?: () => void;
}

export const TextInput = defineComponent({
  name: 'TextInput',
  props: {
    modelValue: {
      type: String,
      default: '',
    },
    placeholder: {
      type: String,
      default: '',
    },
    focus: {
      type: Boolean,
      default: false,
    },
    mask: {
      type: Boolean,
      default: false,
    },
    maskChar: {
      type: String,
      default: '*',
    },
    width: [Number, String] as PropType<number | string>,
    fg: String,
    bg: String,
  },
  emits: ['update:modelValue', 'submit', 'cancel'],
  setup(props, { emit }) {
    const internalValue = ref(props.modelValue);
    const cursorPos = ref(props.modelValue.length);

    // Sync with v-model
    watch(
      () => props.modelValue,
      (newValue) => {
        internalValue.value = newValue;
        // Keep cursor at end if value changes externally
        if (cursorPos.value > newValue.length) {
          cursorPos.value = newValue.length;
        }
      }
    );

    // Update value and emit
    const updateValue = (value: string) => {
      internalValue.value = value;
      emit('update:modelValue', value);
    };

    // Insert text at cursor position
    const insertText = (text: string) => {
      const before = internalValue.value.slice(0, cursorPos.value);
      const after = internalValue.value.slice(cursorPos.value);
      updateValue(before + text + after);
      cursorPos.value += text.length;
    };

    // Delete character before cursor (Backspace)
    const deleteBack = () => {
      if (cursorPos.value > 0) {
        const before = internalValue.value.slice(0, cursorPos.value - 1);
        const after = internalValue.value.slice(cursorPos.value);
        updateValue(before + after);
        cursorPos.value--;
      }
    };

    // Delete character at cursor (Delete)
    const deleteForward = () => {
      if (cursorPos.value < internalValue.value.length) {
        const before = internalValue.value.slice(0, cursorPos.value);
        const after = internalValue.value.slice(cursorPos.value + 1);
        updateValue(before + after);
      }
    };

    // Move cursor left
    const moveLeft = () => {
      if (cursorPos.value > 0) {
        cursorPos.value--;
      }
    };

    // Move cursor right
    const moveRight = () => {
      if (cursorPos.value < internalValue.value.length) {
        cursorPos.value++;
      }
    };

    // Move cursor to start
    const moveToStart = () => {
      cursorPos.value = 0;
    };

    // Move cursor to end
    const moveToEnd = () => {
      cursorPos.value = internalValue.value.length;
    };

    // Use focus prop to control input handling
    const isActive = computed(() => props.focus);

    // Handle keyboard input when focused
    useInput({
      isActive,
      onChar: (char) => {
        insertText(char);
      },
      onArrow: (direction) => {
        if (direction === 'left') moveLeft();
        if (direction === 'right') moveRight();
      },
      onKey: (key, modifiers) => {
        if (key === 'backspace') {
          deleteBack();
        } else if (key === 'delete') {
          deleteForward();
        } else if (key === 'home') {
          moveToStart();
        } else if (key === 'end') {
          moveToEnd();
        } else if (key === 'a' && modifiers.ctrl) {
          // Ctrl+A - select all (move to end for now)
          moveToEnd();
        }
      },
      onSubmit: () => {
        emit('submit', internalValue.value);
      },
      onEscape: () => {
        emit('cancel');
      },
    });

    return () => {
      const style: Record<string, unknown> = {};
      if (props.width !== undefined) {
        style.width = String(props.width);
      }

      return h('input', {
        value: internalValue.value,
        placeholder: props.placeholder,
        focused: props.focus,
        cursor: cursorPos.value,
        mask: props.mask,
        'mask-char': props.maskChar,
        style,
        fg: props.fg,
        bg: props.bg,
      });
    };
  },
});

/**
 * Password input variant
 */
export const PasswordInput = defineComponent({
  name: 'PasswordInput',
  props: {
    modelValue: {
      type: String,
      default: '',
    },
    placeholder: {
      type: String,
      default: 'Enter password...',
    },
    focus: Boolean,
    width: [Number, String] as PropType<number | string>,
    fg: String,
    bg: String,
  },
  emits: ['update:modelValue', 'submit', 'cancel'],
  setup(props, { emit }) {
    return () =>
      h(TextInput, {
        ...props,
        mask: true,
        'onUpdate:modelValue': (v: string) => emit('update:modelValue', v),
        onSubmit: (v: string) => emit('submit', v),
        onCancel: () => emit('cancel'),
      });
  },
});
