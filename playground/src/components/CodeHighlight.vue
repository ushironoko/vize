<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { createHighlighter, type Highlighter } from 'shiki';

const props = defineProps<{
  code: string;
  language: 'javascript' | 'json' | 'css' | 'html' | 'typescript';
  showLineNumbers?: boolean;
}>();

const highlightedCode = ref('');
let highlighter: Highlighter | null = null;

async function initHighlighter() {
  if (!highlighter) {
    highlighter = await createHighlighter({
      themes: ['night-owl'],
      langs: ['javascript', 'json', 'css', 'html', 'typescript'],
    });
  }
  return highlighter;
}

async function highlight() {
  const hl = await initHighlighter();
  const html = hl.codeToHtml(props.code, {
    lang: props.language,
    theme: 'night-owl',
  });
  highlightedCode.value = html;
}

onMounted(highlight);
watch(() => [props.code, props.language], highlight);
</script>

<template>
  <div class="code-highlight" :class="{ 'with-line-numbers': showLineNumbers }">
    <div v-html="highlightedCode"></div>
  </div>
</template>

<style scoped>
.code-highlight {
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  line-height: 1.6;
  border-radius: 12px;
  overflow: auto;
}

.code-highlight :deep(pre) {
  margin: 0;
  padding: 16px !important;
  background: #16213e !important;
  border-radius: 12px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-word;
}

.code-highlight :deep(code) {
  font-family: inherit;
}

.code-highlight.with-line-numbers :deep(.line) {
  display: flex;
}

.code-highlight.with-line-numbers :deep(.line::before) {
  content: counter(line);
  counter-increment: line;
  display: inline-block;
  width: 30px;
  color: #71717a;
  user-select: none;
  text-align: right;
  margin-right: 16px;
  font-size: 12px;
}

.code-highlight.with-line-numbers :deep(code) {
  counter-reset: line;
}
</style>
