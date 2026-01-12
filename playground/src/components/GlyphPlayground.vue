<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import MonacoEditor from './MonacoEditor.vue';
import CodeHighlight from './CodeHighlight.vue';
import type { WasmModule, FormatOptions, FormatResult } from '../wasm/index';

const props = defineProps<{
  compiler: WasmModule | null;
}>();

const GLYPH_PRESET = `<template>
<div class="container">
<h1>{{ count }}</h1>
<p>Doubled: {{ doubled }}</p>
<div class="buttons">
<button @click="decrement">-1</button>
<button @click="increment">+1</button>
</div>
</div>
</template>

<script setup lang="ts">
import {ref,computed,watch} from 'vue'

const count=ref(0)
const doubled=computed(()=>count.value*2)

function increment(){count.value++}
function decrement(){count.value--}

watch(count,(newVal,oldVal)=>{
console.log(\`Count changed from \${oldVal} to \${newVal}\`)
})
<\/script>

<style scoped>
.container{padding:20px;background:#f0f0f0}
h1{color:#333;font-size:2rem}
.buttons{display:flex;gap:10px}
button{padding:8px 16px;cursor:pointer}
</style>
`;

const source = ref(GLYPH_PRESET);
const formatResult = ref<FormatResult | null>(null);
const error = ref<string | null>(null);
const formatTime = ref<number | null>(null);
const activeTab = ref<'formatted' | 'diff'>('formatted');

// Format options
const options = ref<FormatOptions>({
  printWidth: 100,
  tabWidth: 2,
  useTabs: false,
  semi: true,
  singleQuote: false,
  bracketSpacing: true,
  bracketSameLine: false,
  singleAttributePerLine: false,
});

const diffLines = computed(() => {
  if (!formatResult.value) return [];

  const original = source.value.split('\n');
  const formatted = formatResult.value.code.split('\n');
  const diff: Array<{ type: 'same' | 'removed' | 'added'; content: string; lineNum: number }> = [];

  // Simple diff - just show removed and added lines
  const maxLen = Math.max(original.length, formatted.length);
  let origIdx = 0;
  let fmtIdx = 0;

  while (origIdx < original.length || fmtIdx < formatted.length) {
    const origLine = original[origIdx];
    const fmtLine = formatted[fmtIdx];

    if (origLine === fmtLine) {
      diff.push({ type: 'same', content: origLine || '', lineNum: origIdx + 1 });
      origIdx++;
      fmtIdx++;
    } else if (origLine !== undefined && fmtLine !== undefined) {
      diff.push({ type: 'removed', content: origLine, lineNum: origIdx + 1 });
      diff.push({ type: 'added', content: fmtLine, lineNum: fmtIdx + 1 });
      origIdx++;
      fmtIdx++;
    } else if (origLine !== undefined) {
      diff.push({ type: 'removed', content: origLine, lineNum: origIdx + 1 });
      origIdx++;
    } else if (fmtLine !== undefined) {
      diff.push({ type: 'added', content: fmtLine, lineNum: fmtIdx + 1 });
      fmtIdx++;
    }
  }

  return diff;
});

async function format() {
  if (!props.compiler) return;

  const startTime = performance.now();
  error.value = null;

  try {
    const result = props.compiler.formatSfc(source.value, options.value);
    formatResult.value = result;
    formatTime.value = performance.now() - startTime;
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
    formatResult.value = null;
  }
}

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);
}

function applyFormatted() {
  if (formatResult.value) {
    source.value = formatResult.value.code;
  }
}

let formatTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  [source, options],
  () => {
    if (formatTimer) clearTimeout(formatTimer);
    formatTimer = setTimeout(format, 300);
  },
  { immediate: true, deep: true }
);

watch(
  () => props.compiler,
  () => {
    if (props.compiler) {
      format();
    }
  }
);
</script>

<template>
  <div class="glyph-playground">
    <div class="panel input-panel">
      <div class="panel-header">
        <h2>Vue SFC (.vue)</h2>
        <div class="panel-actions">
          <button @click="source = GLYPH_PRESET" class="btn-ghost">Reset</button>
          <button @click="copyToClipboard(source)" class="btn-ghost">Copy</button>
        </div>
      </div>
      <div class="editor-container">
        <MonacoEditor v-model="source" language="vue" />
      </div>
    </div>

    <div class="panel output-panel">
      <div class="panel-header">
        <h2>
          Formatted Output
          <span v-if="formatTime !== null" class="format-time">
            {{ formatTime.toFixed(4) }}ms
          </span>
          <span v-if="formatResult" :class="['status-badge', formatResult.changed ? 'changed' : 'unchanged']">
            {{ formatResult.changed ? 'Changed' : 'Unchanged' }}
          </span>
        </h2>
        <div class="panel-actions">
          <button v-if="formatResult?.changed" @click="applyFormatted" class="btn-primary">Apply</button>
          <button @click="copyToClipboard(formatResult?.code || '')" class="btn-ghost">Copy</button>
        </div>
        <div class="tabs">
          <button
            :class="['tab', { active: activeTab === 'formatted' }]"
            @click="activeTab = 'formatted'"
          >
            Formatted
          </button>
          <button
            :class="['tab', { active: activeTab === 'diff' }]"
            @click="activeTab = 'diff'"
          >
            Diff
          </button>
        </div>
      </div>

      <div class="output-content">
        <div v-if="error" class="error">
          <h3>Format Error</h3>
          <pre>{{ error }}</pre>
        </div>

        <template v-else-if="formatResult">
          <!-- Formatted Tab -->
          <div v-if="activeTab === 'formatted'" class="formatted-output">
            <CodeHighlight :code="formatResult.code" language="vue" show-line-numbers />
          </div>

          <!-- Diff Tab -->
          <div v-else-if="activeTab === 'diff'" class="diff-output">
            <div v-if="!formatResult.changed" class="no-changes">
              No changes needed
            </div>
            <div v-else class="diff-view">
              <div
                v-for="(line, i) in diffLines"
                :key="i"
                :class="['diff-line', line.type]"
              >
                <span class="line-prefix">{{ line.type === 'removed' ? '-' : line.type === 'added' ? '+' : ' ' }}</span>
                <span class="line-content">{{ line.content }}</span>
              </div>
            </div>
          </div>
        </template>

        <div v-else class="loading">
          <span>Enter Vue code to format</span>
        </div>
      </div>

      <!-- Format Options -->
      <div class="options-panel">
        <h3>Format Options</h3>
        <div class="options-grid">
          <label class="option">
            <span>Print Width</span>
            <input type="number" v-model.number="options.printWidth" min="40" max="200" />
          </label>
          <label class="option">
            <span>Tab Width</span>
            <input type="number" v-model.number="options.tabWidth" min="1" max="8" />
          </label>
          <label class="option checkbox">
            <input type="checkbox" v-model="options.useTabs" />
            <span>Use Tabs</span>
          </label>
          <label class="option checkbox">
            <input type="checkbox" v-model="options.semi" />
            <span>Semicolons</span>
          </label>
          <label class="option checkbox">
            <input type="checkbox" v-model="options.singleQuote" />
            <span>Single Quotes</span>
          </label>
          <label class="option checkbox">
            <input type="checkbox" v-model="options.bracketSpacing" />
            <span>Bracket Spacing</span>
          </label>
          <label class="option checkbox">
            <input type="checkbox" v-model="options.bracketSameLine" />
            <span>Bracket Same Line</span>
          </label>
          <label class="option checkbox">
            <input type="checkbox" v-model="options.singleAttributePerLine" />
            <span>Single Attr Per Line</span>
          </label>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.glyph-playground {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0;
  height: 100%;
  grid-column: 1 / -1; /* Span full width of parent grid */
}

.panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.input-panel {
  border-right: 1px solid var(--border-primary);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-primary);
  flex-shrink: 0;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.panel-header h2 {
  font-size: 0.875rem;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.format-time {
  font-size: 0.75rem;
  font-weight: 400;
  color: var(--text-muted);
}

.status-badge {
  font-size: 0.625rem;
  padding: 0.125rem 0.5rem;
  border-radius: 3px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.status-badge.changed {
  background: rgba(245, 158, 11, 0.2);
  color: #f59e0b;
}

.status-badge.unchanged {
  background: rgba(74, 222, 128, 0.2);
  color: #4ade80;
}

.panel-actions {
  display: flex;
  gap: 0.5rem;
}

.btn-ghost {
  padding: 0.25rem 0.75rem;
  font-size: 0.75rem;
  background: transparent;
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.btn-ghost:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.btn-primary {
  padding: 0.25rem 0.75rem;
  font-size: 0.75rem;
  background: var(--accent-rust);
  border: none;
  border-radius: 4px;
  color: white;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-primary:hover {
  background: #8b3720;
}

.tabs {
  display: flex;
  gap: 0.25rem;
}

.tab {
  padding: 0.375rem 0.75rem;
  font-size: 0.75rem;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s;
}

.tab:hover {
  color: var(--text-secondary);
  background: var(--bg-tertiary);
}

.tab.active {
  color: var(--accent-rust);
  background: rgba(163, 72, 40, 0.15);
}

.editor-container {
  flex: 1;
  overflow: hidden;
}

.output-content {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
}

.error {
  padding: 1rem;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 6px;
}

.error h3 {
  color: #ef4444;
  margin-bottom: 0.5rem;
  font-size: 0.875rem;
}

.error pre {
  font-size: 0.75rem;
  color: #fca5a5;
  white-space: pre-wrap;
  word-break: break-word;
}

.formatted-output {
  height: 100%;
}

.no-changes {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: #4ade80;
  font-size: 1rem;
}

.diff-view {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.8rem;
  overflow-x: auto;
}

.diff-line {
  display: flex;
  padding: 0.125rem 0.5rem;
  white-space: pre;
}

.diff-line.same {
  color: var(--text-secondary);
}

.diff-line.removed {
  background: rgba(239, 68, 68, 0.15);
  color: #fca5a5;
}

.diff-line.added {
  background: rgba(74, 222, 128, 0.15);
  color: #86efac;
}

.line-prefix {
  width: 1.5rem;
  flex-shrink: 0;
  user-select: none;
}

.line-content {
  flex: 1;
}

.options-panel {
  padding: 0.75rem 1rem;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-primary);
  flex-shrink: 0;
}

.options-panel h3 {
  font-size: 0.75rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.options-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
}

.option {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.option input[type="number"] {
  width: 60px;
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  background: var(--bg-primary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  color: var(--text-primary);
}

.option.checkbox {
  cursor: pointer;
}

.option.checkbox input[type="checkbox"] {
  width: 14px;
  height: 14px;
  accent-color: var(--accent-rust);
}

.loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--text-muted);
}

/* Mobile responsive */
@media (max-width: 768px) {
  .glyph-playground {
    grid-template-columns: 1fr;
    grid-template-rows: 1fr 1fr;
  }

  .input-panel {
    border-right: none;
    border-bottom: 1px solid var(--border-primary);
  }

  .panel-header {
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .tabs {
    flex-wrap: wrap;
    width: 100%;
  }

  .options-grid {
    flex-direction: column;
    gap: 0.5rem;
  }

  .option {
    justify-content: space-between;
  }
}
</style>
