<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, shallowRef } from 'vue';
import * as monaco from 'monaco-editor';

export interface Diagnostic {
  message: string;
  startLine: number;
  startColumn: number;
  endLine?: number;
  endColumn?: number;
  severity: 'error' | 'warning' | 'info';
}

const props = defineProps<{
  modelValue: string;
  language: string;
  diagnostics?: Diagnostic[];
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
}>();

const containerRef = ref<HTMLDivElement | null>(null);
const editorInstance = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null);
let isConfigured = false;

function configureMonaco() {
  if (isConfigured) return;
  isConfigured = true;

  // Register Vue language
  monaco.languages.register({ id: 'vue', extensions: ['.vue'] });

  // Set monarch tokenizer for Vue (HTML-based with Vue extensions)
  monaco.languages.setMonarchTokensProvider('vue', {
    defaultToken: '',
    tokenPostfix: '.vue',
    keywords: ['v-if', 'v-else', 'v-else-if', 'v-for', 'v-show', 'v-model', 'v-bind', 'v-on', 'v-slot', 'v-pre', 'v-once', 'v-memo', 'v-cloak'],
    tokenizer: {
      root: [
        [/<!--/, { token: 'comment', next: '@htmlComment' }],
        [/<script\s+setup\s+vapor[^>]*>/, { token: 'tag', next: '@script' }],
        [/<script\s+setup[^>]*>/, { token: 'tag', next: '@script' }],
        [/<script[^>]*>/, { token: 'tag', next: '@script' }],
        [/<style[^>]*>/, { token: 'tag', next: '@style' }],
        [/<template[^>]*>/, { token: 'tag', next: '@template' }],
        [/<\/?[\w-]+/, { token: 'tag', next: '@tag' }],
        [/\{\{/, { token: 'delimiter.bracket', next: '@interpolation' }],
      ],
      tag: [
        [/\s+/, ''],
        [/(v-[\w-]+|@[\w.-]+|:[\w.-]+|#[\w.-]+)/, 'attribute.name.vue'],
        [/[\w-]+/, 'attribute.name'],
        [/=/, 'delimiter'],
        [/"[^"]*"/, 'attribute.value'],
        [/'[^']*'/, 'attribute.value'],
        [/>/, { token: 'tag', next: '@pop' }],
        [/\/>/, { token: 'tag', next: '@pop' }],
      ],
      template: [
        [/<\/template>/, { token: 'tag', next: '@pop' }],
        [/<!--/, { token: 'comment', next: '@htmlComment' }],
        [/\{\{/, { token: 'delimiter.bracket', next: '@interpolation' }],
        [/<\/?[\w-]+/, { token: 'tag', next: '@tag' }],
        [/./, ''],
      ],
      htmlComment: [
        [/-->/, { token: 'comment', next: '@pop' }],
        [/./, 'comment'],
      ],
      interpolation: [
        [/\}\}/, { token: 'delimiter.bracket', next: '@pop' }],
        [/[\w.]+/, 'variable'],
        [/./, ''],
      ],
      script: [
        [/<\/script>/, { token: 'tag', next: '@pop' }],
        [/(import|export|from|const|let|var|function|return|if|else|for|while|class|interface|type|extends|implements)(?=\s)/, 'keyword'],
        [/(defineProps|defineEmits|defineExpose|defineOptions|defineSlots|defineModel|withDefaults)/, 'keyword.control.vue'],
        [/(ref|reactive|computed|watch|watchEffect|onMounted|onUnmounted|toRef|toRefs)/, 'support.function.vue'],
        [/"[^"]*"/, 'string'],
        [/'[^']*'/, 'string'],
        [/`[^`]*`/, 'string'],
        [/\/\/.*$/, 'comment'],
        [/\/\*/, { token: 'comment', next: '@comment' }],
        [/[{}()[\]]/, 'delimiter.bracket'],
        [/[<>]=?|[!=]=?=?|&&|\|\|/, 'operator'],
        [/\d+/, 'number'],
        [/[\w$]+/, 'identifier'],
        [/./, ''],
      ],
      comment: [
        [/\*\//, { token: 'comment', next: '@pop' }],
        [/./, 'comment'],
      ],
      style: [
        [/<\/style>/, { token: 'tag', next: '@pop' }],
        [/\/\*/, { token: 'comment', next: '@cssComment' }],
        [/[\w-]+(?=\s*:)/, 'attribute.name'],
        [/:/, 'delimiter'],
        [/[{}]/, 'delimiter.bracket'],
        [/"[^"]*"/, 'string'],
        [/'[^']*'/, 'string'],
        [/#[\da-fA-F]+/, 'number.hex'],
        [/\d+[\w%]*/, 'number'],
        [/[\w-]+/, 'attribute.value'],
        [/./, ''],
      ],
      cssComment: [
        [/\*\//, { token: 'comment', next: '@pop' }],
        [/./, 'comment'],
      ],
    },
  });

  // Set Vue language configuration
  monaco.languages.setLanguageConfiguration('vue', {
    comments: {
      blockComment: ['<!--', '-->'],
    },
    brackets: [
      ['<!--', '-->'],
      ['<', '>'],
      ['{', '}'],
      ['[', ']'],
      ['(', ')'],
    ],
    autoClosingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
      { open: '`', close: '`' },
      { open: '<', close: '>' },
      { open: '<!--', close: '-->' },
    ],
    surroundingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
      { open: '<', close: '>' },
    ],
  });

  // Define custom theme matching project CSS (Rust/Metal theme)
  monaco.editor.defineTheme('vue-dark', {
    base: 'vs-dark',
    inherit: true,
    rules: [
      { token: 'keyword', foreground: 'e07048' },
      { token: 'keyword.control.vue', foreground: 'f08060', fontStyle: 'bold' },
      { token: 'support.function.vue', foreground: 'e07048' },
      { token: 'attribute.name.vue', foreground: 'e07048' },
      { token: 'variable', foreground: 'd0d4dc' },
      { token: 'tag', foreground: 'e07048' },
      { token: 'attribute.name', foreground: '9ca3b0' },
      { token: 'attribute.value', foreground: 'd0d4dc' },
      { token: 'string', foreground: 'd0d4dc' },
      { token: 'number', foreground: 'f08060' },
      { token: 'comment', foreground: '6b7280' },
      { token: 'delimiter.bracket', foreground: '9ca3b0' },
      { token: 'identifier', foreground: 'f0f2f5' },
    ],
    colors: {
      'editor.background': '#1a1b21',
      'editor.foreground': '#f0f2f5',
      'editor.lineHighlightBackground': '#252830',
      'editor.selectionBackground': '#e0704840',
      'editorCursor.foreground': '#e07048',
      'editorLineNumber.foreground': '#6b7280',
      'editorLineNumber.activeForeground': '#9ca3b0',
      'editorIndentGuide.background': '#252830',
      'editorIndentGuide.activeBackground': '#e0704840',
      'editor.inactiveSelectionBackground': '#e0704820',
    },
  });
}

onMounted(() => {
  if (!containerRef.value) return;

  configureMonaco();

  editorInstance.value = monaco.editor.create(containerRef.value, {
    value: props.modelValue,
    language: props.language,
    theme: 'vue-dark',
    fontSize: 14,
    fontFamily: "'JetBrains Mono', monospace",
    minimap: { enabled: false },
    lineNumbers: 'on',
    scrollBeyondLastLine: false,
    padding: { top: 16 },
    automaticLayout: true,
    quickSuggestions: true,
    suggestOnTriggerCharacters: true,
  });

  editorInstance.value.onDidChangeModelContent(() => {
    const value = editorInstance.value?.getValue() || '';
    emit('update:modelValue', value);
  });
});

onUnmounted(() => {
  editorInstance.value?.dispose();
});

watch(() => props.modelValue, (newValue) => {
  if (editorInstance.value && editorInstance.value.getValue() !== newValue) {
    editorInstance.value.setValue(newValue);
  }
});

watch(() => props.language, (newLanguage) => {
  if (editorInstance.value) {
    const model = editorInstance.value.getModel();
    if (model) {
      monaco.editor.setModelLanguage(model, newLanguage);
    }
  }
});

// Update diagnostics markers
watch(() => props.diagnostics, (diagnostics) => {
  if (!editorInstance.value) return;
  const model = editorInstance.value.getModel();
  if (!model) return;

  if (!diagnostics || diagnostics.length === 0) {
    monaco.editor.setModelMarkers(model, 'vize', []);
    return;
  }

  const markers: monaco.editor.IMarkerData[] = diagnostics.map(d => ({
    severity: d.severity === 'error'
      ? monaco.MarkerSeverity.Error
      : d.severity === 'warning'
        ? monaco.MarkerSeverity.Warning
        : monaco.MarkerSeverity.Info,
    message: d.message,
    startLineNumber: d.startLine,
    startColumn: d.startColumn,
    endLineNumber: d.endLine ?? d.startLine,
    endColumn: d.endColumn ?? d.startColumn + 1,
  }));

  monaco.editor.setModelMarkers(model, 'vize', markers);
}, { immediate: true });
</script>

<template>
  <div ref="containerRef" class="monaco-container"></div>
</template>

<style scoped>
.monaco-container {
  width: 100%;
  height: 100%;
}
</style>
