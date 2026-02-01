<script setup lang="ts">
import { ref, watch, onMounted, computed } from "vue";
import { createHighlighter, type Highlighter, type ThemeRegistration } from "shiki";

const props = defineProps<{
  code: string;
  language: "javascript" | "json" | "css" | "html" | "typescript";
  showLineNumbers?: boolean;
}>();

// Custom theme matching project's Rust/Metal design
const vizeTheme: ThemeRegistration = {
  name: "vize-dark",
  type: "dark",
  colors: {
    "editor.background": "#1a1b21",
    "editor.foreground": "#f0f2f5",
  },
  tokenColors: [
    { scope: ["keyword", "storage.type", "storage.modifier"], settings: { foreground: "#e07048" } },
    { scope: ["entity.name.function", "support.function"], settings: { foreground: "#f08060" } },
    {
      scope: ["entity.name.tag", "punctuation.definition.tag"],
      settings: { foreground: "#e07048" },
    },
    { scope: ["entity.other.attribute-name"], settings: { foreground: "#9ca3b0" } },
    { scope: ["string", "string.quoted"], settings: { foreground: "#d0d4dc" } },
    { scope: ["constant.numeric", "constant.language"], settings: { foreground: "#f08060" } },
    { scope: ["variable", "variable.other"], settings: { foreground: "#f0f2f5" } },
    { scope: ["comment", "punctuation.definition.comment"], settings: { foreground: "#4b5563" } },
    { scope: ["punctuation", "meta.brace"], settings: { foreground: "#9ca3b0" } },
    { scope: ["entity.name.type", "support.type"], settings: { foreground: "#d0d4dc" } },
    {
      scope: ["meta.property-name", "support.type.property-name"],
      settings: { foreground: "#e07048" },
    },
    {
      scope: ["meta.property-value", "support.constant.property-value"],
      settings: { foreground: "#d0d4dc" },
    },
  ],
};

const highlightedLines = ref<string[]>([]);
let highlighter: Highlighter | null = null;

async function initHighlighter() {
  if (!highlighter) {
    highlighter = await createHighlighter({
      themes: [vizeTheme],
      langs: ["javascript", "json", "css", "html", "typescript"],
    });
  }
  return highlighter;
}

async function highlight() {
  const hl = await initHighlighter();
  const tokens = hl.codeToTokens(props.code, {
    lang: props.language,
    theme: "vize-dark",
  });

  let lines = tokens.tokens;
  // Remove trailing empty line if present
  if (lines.length > 0 && lines[lines.length - 1].length === 0) {
    lines = lines.slice(0, -1);
  }

  // Build HTML for each line
  highlightedLines.value = lines.map((lineTokens) => {
    if (lineTokens.length === 0) {
      return "&nbsp;";
    }
    return lineTokens
      .map((token) => {
        const escaped = token.content
          .replace(/&/g, "&amp;")
          .replace(/</g, "&lt;")
          .replace(/>/g, "&gt;");
        return `<span style="color:${token.color}">${escaped}</span>`;
      })
      .join("");
  });
}

const lineCount = computed(() => highlightedLines.value.length);

onMounted(highlight);
watch(() => [props.code, props.language], highlight);
</script>

<template>
  <div class="code-highlight" :class="{ 'with-line-numbers': showLineNumbers }">
    <div v-if="showLineNumbers" class="line-numbers">
      <span v-for="i in lineCount" :key="i" class="line-number">{{ i }}</span>
    </div>
    <div class="code-content">
      <div
        v-for="(line, index) in highlightedLines"
        :key="index"
        class="code-line"
        v-html="line"
      ></div>
    </div>
  </div>
</template>

<style scoped>
.code-highlight {
  display: flex;
  font-family: "JetBrains Mono", monospace;
  font-size: 13px;
  line-height: 20px;
  border-radius: 4px;
  overflow: auto;
  background: #1a1b21;
}

.line-numbers {
  display: flex;
  flex-direction: column;
  padding-top: 12px;
  padding-bottom: 12px;
  background: rgba(0, 0, 0, 0.2);
  border-right: 1px solid rgba(255, 255, 255, 0.1);
  user-select: none;
  flex-shrink: 0;
  position: sticky;
  left: 0;
}

.line-number {
  display: block;
  padding: 0 12px;
  text-align: right;
  color: #71717a;
  line-height: 20px;
  height: 20px;
  box-sizing: border-box;
}

.code-content {
  flex: 1;
  padding-top: 12px;
  padding-bottom: 12px;
  padding-left: 16px;
  padding-right: 16px;
  overflow-x: auto;
}

.code-line {
  white-space: pre;
  line-height: 20px;
  height: 20px;
  box-sizing: border-box;
}

.code-line :deep(span) {
  line-height: inherit;
}
</style>
