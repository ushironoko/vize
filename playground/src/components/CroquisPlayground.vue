<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue';
import MonacoEditor from './MonacoEditor.vue';
import type { WasmModule, AnalysisResult, BindingDisplay, BindingSource, ScopeDisplay } from '../wasm/index';

const props = defineProps<{
  compiler: WasmModule | null;
}>();

const ANALYSIS_PRESET = `<script setup lang="ts">
import { ref, computed, watch } from 'vue'

// Props with type annotation
const props = defineProps<{
  title: string
  count?: number
}>()

// Emits declaration
const emit = defineEmits<{
  update: [number]
  close: []
}>()

// Reactive refs
const counter = ref(0)
const doubled = computed(() => counter.value * 2)
const message = ref('Hello Vue!')

// Watchers
watch(counter, (newVal) => {
  console.log('Counter changed:', newVal)
})

// Methods
function increment() {
  counter.value++
  emit('update', counter.value)
}

function reset() {
  counter.value = 0
}
<\/script>

<template>
  <div class="container">
    <h1>{{ props.title }}</h1>
    <p class="message">{{ message }}</p>
    <div class="counter">
      <span>Count: {{ counter }}</span>
      <span>Doubled: {{ doubled }}</span>
    </div>
    <div class="actions">
      <button @click="increment">+1</button>
      <button @click="reset">Reset</button>
    </div>
  </div>
</template>

<style scoped>
.container {
  padding: 20px;
  font-family: system-ui, sans-serif;
}

.message {
  color: v-bind('counter > 5 ? "red" : "green"');
}

.counter {
  display: flex;
  gap: 16px;
  margin: 16px 0;
}

.actions {
  display: flex;
  gap: 8px;
}

button {
  padding: 8px 16px;
  border-radius: 4px;
  border: 1px solid #ccc;
  cursor: pointer;
}
</style>
`;

const source = ref(ANALYSIS_PRESET);
const analysisResult = ref<AnalysisResult | null>(null);
const error = ref<string | null>(null);
const activeTab = ref<'vir' | 'stats' | 'bindings' | 'scopes' | 'diagnostics'>('vir');
const analysisTime = ref<number | null>(null);

// Perform analysis
async function analyze() {
  if (!props.compiler) {
    error.value = 'Compiler not loaded';
    return;
  }

  error.value = null;
  const startTime = performance.now();

  try {
    const result = props.compiler.analyzeSfc(source.value, {
      filename: 'Component.vue',
    });
    analysisResult.value = result;
    analysisTime.value = performance.now() - startTime;
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
    analysisResult.value = null;
  }
}

// Watch for source changes and re-analyze
let analyzeTimer: ReturnType<typeof setTimeout> | null = null;
watch(source, () => {
  if (analyzeTimer) clearTimeout(analyzeTimer);
  analyzeTimer = setTimeout(analyze, 300);
}, { immediate: false });

// Watch for compiler changes
watch(() => props.compiler, () => {
  if (props.compiler) {
    analyze();
  }
});

// Analyze on mount
onMounted(() => {
  if (props.compiler) {
    analyze();
  }
});

// Computed values for display
const summary = computed(() => analysisResult.value?.summary);
const bindings = computed(() => summary.value?.bindings || []);
const macros = computed(() => summary.value?.macros || []);
const scopes = computed(() => summary.value?.scopes || []);
const css = computed(() => summary.value?.css);
const diagnostics = computed(() => analysisResult.value?.diagnostics || []);
const stats = computed(() => summary.value?.stats);

// Group bindings by source
const bindingsBySource = computed(() => {
  const groups: Record<string, BindingDisplay[]> = {};
  for (const binding of bindings.value) {
    const source = binding.source || 'unknown';
    if (!groups[source]) groups[source] = [];
    groups[source].push(binding);
  }
  return groups;
});

// VIR (Vize Intermediate Representation) text
const virText = computed(() => analysisResult.value?.vir || '');

// Token types for VIR syntax highlighting
type VirTokenType =
  | 'border'      // ╭╰│├└─┌┐ etc.
  | 'section'     // ■ section headers
  | 'section-name' // MACROS, BINDINGS, etc.
  | 'macro'       // @defineProps, @defineEmits
  | 'type'        // <TypeAnnotation>
  | 'binding'     // ▸ binding marker
  | 'identifier'  // variable names
  | 'tag'         // [SetupRef], [SetupConst]
  | 'source'      // ref, computed, props
  | 'arrow'       // →
  | 'number'      // numbers
  | 'diagnostic'  // ✗ ⚠ ℹ
  | 'keyword'     // type:, args:, etc.
  | 'colon'       // : separator
  | 'bracket'     // [], {}
  | 'plain';      // everything else

interface VirToken {
  type: VirTokenType;
  text: string;
}

interface VirLine {
  tokens: VirToken[];
  index: number;
  lineType: string;
}

// Tokenize a VIR line for fine-grained syntax highlighting
function tokenizeVirLine(line: string): VirToken[] {
  const tokens: VirToken[] = [];
  let remaining = line;

  while (remaining.length > 0) {
    let matched = false;

    // Border characters: ╭╰│├└─┌┐╮╯┤┬┴┼
    const borderMatch = remaining.match(/^[╭╰│├└─┌┐╮╯┤┬┴┼]+/);
    if (borderMatch) {
      tokens.push({ type: 'border', text: borderMatch[0] });
      remaining = remaining.slice(borderMatch[0].length);
      matched = true;
      continue;
    }

    // Section marker ■
    if (remaining.startsWith('■')) {
      tokens.push({ type: 'section', text: '■' });
      remaining = remaining.slice(1);
      matched = true;
      continue;
    }

    // Section names (all caps words)
    const sectionNameMatch = remaining.match(/^(MACROS|BINDINGS|SCOPES|PROPS|EMITS|CSS|DIAGNOSTICS|STATS|SUMMARY)/);
    if (sectionNameMatch) {
      tokens.push({ type: 'section-name', text: sectionNameMatch[0] });
      remaining = remaining.slice(sectionNameMatch[0].length);
      matched = true;
      continue;
    }

    // Macro names @defineProps, @defineEmits, etc.
    const macroMatch = remaining.match(/^@\w+/);
    if (macroMatch) {
      tokens.push({ type: 'macro', text: macroMatch[0] });
      remaining = remaining.slice(macroMatch[0].length);
      matched = true;
      continue;
    }

    // Type annotations <...>
    const typeMatch = remaining.match(/^<[^>]+>/);
    if (typeMatch) {
      tokens.push({ type: 'type', text: typeMatch[0] });
      remaining = remaining.slice(typeMatch[0].length);
      matched = true;
      continue;
    }

    // Binding marker ▸
    if (remaining.startsWith('▸')) {
      tokens.push({ type: 'binding', text: '▸' });
      remaining = remaining.slice(1);
      matched = true;
      continue;
    }

    // Arrow →
    if (remaining.startsWith('→')) {
      tokens.push({ type: 'arrow', text: '→' });
      remaining = remaining.slice(1);
      matched = true;
      continue;
    }

    // Diagnostic icons
    const diagMatch = remaining.match(/^[✗⚠ℹ✓]/);
    if (diagMatch) {
      tokens.push({ type: 'diagnostic', text: diagMatch[0] });
      remaining = remaining.slice(diagMatch[0].length);
      matched = true;
      continue;
    }

    // Tags in brackets [SetupRef], [Module], etc.
    const tagMatch = remaining.match(/^\[[A-Za-z][A-Za-z0-9_]*\]/);
    if (tagMatch) {
      tokens.push({ type: 'tag', text: tagMatch[0] });
      remaining = remaining.slice(tagMatch[0].length);
      matched = true;
      continue;
    }

    // Keywords like type:, args:, scoped:, etc.
    const keywordMatch = remaining.match(/^(type|args|scoped|selectors|v-bind|start|end|depth|parent|bindings|children):/);
    if (keywordMatch) {
      tokens.push({ type: 'keyword', text: keywordMatch[1] });
      tokens.push({ type: 'colon', text: ':' });
      remaining = remaining.slice(keywordMatch[0].length);
      matched = true;
      continue;
    }

    // Source types (ref, computed, props, etc.) - after keywords
    const sourceMatch = remaining.match(/^\b(ref|computed|reactive|props|emits|local|import|function|class|unknown)\b/);
    if (sourceMatch) {
      tokens.push({ type: 'source', text: sourceMatch[0] });
      remaining = remaining.slice(sourceMatch[0].length);
      matched = true;
      continue;
    }

    // Numbers including ranges like [0:100]
    const numberMatch = remaining.match(/^\d+/);
    if (numberMatch) {
      tokens.push({ type: 'number', text: numberMatch[0] });
      remaining = remaining.slice(numberMatch[0].length);
      matched = true;
      continue;
    }

    // Brackets and braces
    const bracketMatch = remaining.match(/^[\[\]{}()]/);
    if (bracketMatch) {
      tokens.push({ type: 'bracket', text: bracketMatch[0] });
      remaining = remaining.slice(bracketMatch[0].length);
      matched = true;
      continue;
    }

    // Colons (standalone)
    if (remaining.startsWith(':')) {
      tokens.push({ type: 'colon', text: ':' });
      remaining = remaining.slice(1);
      matched = true;
      continue;
    }

    // Identifiers (variable names, etc.)
    const identMatch = remaining.match(/^[a-zA-Z_][a-zA-Z0-9_]*/);
    if (identMatch) {
      tokens.push({ type: 'identifier', text: identMatch[0] });
      remaining = remaining.slice(identMatch[0].length);
      matched = true;
      continue;
    }

    // Whitespace
    const wsMatch = remaining.match(/^\s+/);
    if (wsMatch) {
      tokens.push({ type: 'plain', text: wsMatch[0] });
      remaining = remaining.slice(wsMatch[0].length);
      matched = true;
      continue;
    }

    // Any other character
    if (!matched) {
      tokens.push({ type: 'plain', text: remaining[0] });
      remaining = remaining.slice(1);
    }
  }

  return tokens;
}

// Determine line type for overall styling
function getVirLineType(line: string): string {
  if (line.startsWith('╭') || line.startsWith('│') || line.startsWith('╰')) return 'header';
  if (line.includes('■')) return 'section';
  if (line.includes('@define') || line.includes('┌─ @')) return 'macro';
  if (line.includes('▸')) return 'binding';
  if (line.includes('├─') || line.includes('└─')) return 'tree';
  if (line.includes('✗') || line.includes('⚠')) return 'diagnostic';
  return 'plain';
}

// Parse VIR text into tokenized lines
const virLines = computed((): VirLine[] => {
  if (!virText.value) return [];
  return virText.value.split('\n').map((line, index) => ({
    tokens: tokenizeVirLine(line),
    index,
    lineType: getVirLineType(line),
  }));
});

// Source labels
function getSourceLabel(source: BindingSource | string): string {
  const labels: Record<string, string> = {
    props: 'Props',
    emits: 'Emits',
    model: 'Models',
    slots: 'Slots',
    ref: 'Refs',
    reactive: 'Reactive',
    computed: 'Computed',
    import: 'Imports',
    local: 'Local',
    function: 'Functions',
    class: 'Classes',
    templateRef: 'Template Refs',
    unknown: 'Other',
  };
  return labels[source] || source;
}

// Source colors
function getSourceClass(source: BindingSource | string): string {
  const classes: Record<string, string> = {
    props: 'src-props',
    emits: 'src-emits',
    model: 'src-model',
    slots: 'src-slots',
    ref: 'src-ref',
    reactive: 'src-reactive',
    computed: 'src-computed',
    import: 'src-import',
    local: 'src-local',
    function: 'src-function',
    class: 'src-class',
  };
  return classes[source] || 'src-default';
}
</script>

<template>
  <div class="croquis-playground">
    <div class="panel input-panel">
      <div class="panel-header">
        <div class="header-title">
          <span class="icon">&#x2726;</span>
          <h2>Source</h2>
        </div>
        <div class="panel-actions">
          <button @click="source = ANALYSIS_PRESET" class="btn-ghost">Reset</button>
        </div>
      </div>
      <div class="editor-container">
        <MonacoEditor v-model="source" language="vue" />
      </div>
    </div>

    <div class="panel output-panel">
      <div class="panel-header">
        <div class="header-title">
          <span class="icon">&#x25C8;</span>
          <h2>Semantic Analysis</h2>
          <span v-if="analysisTime !== null" class="perf-badge">
            {{ analysisTime.toFixed(2) }}ms
          </span>
        </div>
        <div class="tabs">
          <button
            :class="['tab', { active: activeTab === 'vir' }]"
            @click="activeTab = 'vir'"
          >VIR</button>
          <button
            :class="['tab', { active: activeTab === 'stats' }]"
            @click="activeTab = 'stats'"
          >Stats</button>
          <button
            :class="['tab', { active: activeTab === 'bindings' }]"
            @click="activeTab = 'bindings'"
          >Bindings</button>
          <button
            :class="['tab', { active: activeTab === 'scopes' }]"
            @click="activeTab = 'scopes'"
          >Scopes</button>
          <button
            :class="['tab', { active: activeTab === 'diagnostics' }]"
            @click="activeTab = 'diagnostics'"
          >
            Diagnostics
            <span v-if="diagnostics.length > 0" class="tab-badge">{{ diagnostics.length }}</span>
          </button>
        </div>
      </div>

      <div class="output-content">
        <div v-if="error" class="error-panel">
          <div class="error-header">Analysis Error</div>
          <pre class="error-content">{{ error }}</pre>
        </div>

        <template v-else-if="analysisResult">
          <!-- VIR Tab (Primary) -->
          <div v-if="activeTab === 'vir'" class="vir-output">
            <div class="vir-header-bar">
              <span class="vir-title">VIR — Vize Intermediate Representation</span>
              <span class="vir-line-count">{{ virLines.length }} lines</span>
            </div>
            <div class="vir-content">
              <div class="vir-line-numbers">
                <span v-for="(_, i) in virLines" :key="i" class="vir-ln">{{ i + 1 }}</span>
              </div>
              <div class="vir-code">
                <div
                  v-for="line in virLines"
                  :key="line.index"
                  :class="['vir-line', `vir-line-${line.lineType}`]"
                ><template v-if="line.tokens.length > 0"><span
                    v-for="(token, ti) in line.tokens"
                    :key="ti"
                    :class="['vir-token', `vir-${token.type}`]"
                  >{{ token.text }}</span></template><template v-else>&#160;</template></div>
              </div>
            </div>
          </div>

          <!-- Stats Tab -->
          <div v-else-if="activeTab === 'stats'" class="stats-output">
            <div class="stats-grid">
              <div class="stat-box">
                <div class="stat-number">{{ stats?.binding_count || 0 }}</div>
                <div class="stat-label">Bindings</div>
              </div>
              <div class="stat-box">
                <div class="stat-number">{{ stats?.macro_count || 0 }}</div>
                <div class="stat-label">Macros</div>
              </div>
              <div class="stat-box">
                <div class="stat-number">{{ stats?.scope_count || 0 }}</div>
                <div class="stat-label">Scopes</div>
              </div>
              <div class="stat-box">
                <div class="stat-number">{{ css?.v_bind_count || 0 }}</div>
                <div class="stat-label">v-bind()</div>
              </div>
            </div>

            <div class="section">
              <h3 class="section-title">Compiler Macros</h3>
              <div v-if="macros.length === 0" class="empty-state">No macros detected</div>
              <div v-else class="macro-list">
                <div v-for="macro in macros" :key="`${macro.name}-${macro.start}`" class="macro-item">
                  <span class="macro-name">{{ macro.name }}</span>
                  <code v-if="macro.type_args" class="macro-type">{{ macro.type_args }}</code>
                  <span v-if="macro.binding" class="macro-binding">→ {{ macro.binding }}</span>
                </div>
              </div>
            </div>

            <div class="section" v-if="css">
              <h3 class="section-title">CSS Analysis</h3>
              <div class="css-info">
                <span class="css-stat">{{ css.selector_count }} selectors</span>
                <span v-if="css.is_scoped" class="css-badge scoped">scoped</span>
                <span v-if="css.v_bind_count > 0" class="css-badge vbind">{{ css.v_bind_count }} v-bind</span>
              </div>
            </div>
          </div>

          <!-- Bindings Tab -->
          <div v-else-if="activeTab === 'bindings'" class="bindings-output">
            <div v-if="bindings.length === 0" class="empty-state">No bindings detected</div>

            <template v-else>
              <div v-for="(group, source) in bindingsBySource" :key="source" class="source-group">
                <div class="source-header">
                  <span :class="['source-indicator', getSourceClass(String(source))]"></span>
                  <span class="source-name">{{ getSourceLabel(String(source)) }}</span>
                  <span class="source-count">{{ group.length }}</span>
                </div>
                <div class="binding-grid">
                  <div v-for="binding in group" :key="binding.name" class="binding-item">
                    <div class="binding-main">
                      <code class="binding-name">{{ binding.name }}</code>
                      <span v-if="binding.metadata?.needsValue" class="needs-value" title="Needs .value">.value</span>
                    </div>
                    <div class="binding-meta">
                      <span class="binding-kind">{{ binding.kind }}</span>
                      <span v-if="binding.typeAnnotation" class="binding-type">: {{ binding.typeAnnotation }}</span>
                    </div>
                    <div class="binding-flags">
                      <span :class="['flag', binding.isUsed ? 'active' : 'inactive']">used</span>
                      <span :class="['flag', binding.isMutated ? 'active' : 'inactive']">mutated</span>
                      <span class="refs">{{ binding.referenceCount }} refs</span>
                    </div>
                  </div>
                </div>
              </div>
            </template>
          </div>

          <!-- Scopes Tab -->
          <div v-else-if="activeTab === 'scopes'" class="scopes-output">
            <div v-if="scopes.length === 0" class="empty-state">No scopes detected</div>

            <div v-else class="scope-tree">
              <div v-for="scope in scopes" :key="scope.id" class="scope-node" :style="{ marginLeft: `${(scope.depth || 0) * 20}px` }">
                <div class="scope-header">
                  <span class="scope-icon">&#x25B8;</span>
                  <span class="scope-kind">{{ scope.kindStr || scope.kind }}</span>
                  <span class="scope-range">[{{ scope.start }}:{{ scope.end }}]</span>
                </div>
                <div v-if="scope.bindings.length > 0" class="scope-bindings">
                  <span v-for="name in scope.bindings" :key="name" class="scope-binding">{{ name }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Diagnostics Tab -->
          <div v-else-if="activeTab === 'diagnostics'" class="diagnostics-output">
            <div v-if="diagnostics.length === 0" class="success-state">
              <span class="success-icon">&#x2713;</span>
              <span>No issues found</span>
            </div>

            <div v-else class="diagnostic-list">
              <div v-for="(diag, i) in diagnostics" :key="i" :class="['diagnostic-item', `severity-${diag.severity}`]">
                <div class="diagnostic-header">
                  <span class="severity-icon">{{ diag.severity === 'error' ? '&#x2717;' : '&#x26A0;' }}</span>
                  <span class="diagnostic-message">{{ diag.message }}</span>
                </div>
                <div class="diagnostic-location">
                  <span class="location-range">{{ diag.start }}:{{ diag.end }}</span>
                  <span v-if="diag.code" class="diagnostic-code">[{{ diag.code }}]</span>
                </div>
              </div>
            </div>
          </div>
        </template>

        <div v-else class="loading-state">
          <span>Analyzing...</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.croquis-playground {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0;
  height: 100%;
  min-height: 0;
  grid-column: 1 / -1;
  background: var(--bg-primary);
}

.panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
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
}

.header-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.header-title .icon {
  font-size: 1rem;
  color: var(--accent-rust);
}

.header-title h2 {
  font-size: 0.875rem;
  font-weight: 600;
  margin: 0;
}

.perf-badge {
  font-size: 0.625rem;
  padding: 0.125rem 0.375rem;
  background: rgba(74, 222, 128, 0.15);
  color: #4ade80;
  border-radius: 3px;
  font-family: 'JetBrains Mono', monospace;
}

.panel-actions {
  display: flex;
  gap: 0.5rem;
}

.btn-ghost {
  padding: 0.25rem 0.5rem;
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

.tabs {
  display: flex;
  gap: 0.125rem;
}

.tab {
  padding: 0.375rem 0.625rem;
  font-size: 0.75rem;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s;
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.tab:hover {
  color: var(--text-secondary);
  background: var(--bg-tertiary);
}

.tab.active {
  color: var(--text-primary);
  background: var(--bg-tertiary);
  font-weight: 500;
}

.tab-badge {
  font-size: 0.625rem;
  padding: 0.0625rem 0.3125rem;
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
  border-radius: 8px;
  min-width: 1rem;
  text-align: center;
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

/* Error State */
.error-panel {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 6px;
  overflow: hidden;
}

.error-header {
  padding: 0.5rem 0.75rem;
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
  font-size: 0.75rem;
  font-weight: 600;
}

.error-content {
  padding: 0.75rem;
  font-size: 0.75rem;
  color: #fca5a5;
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}

/* Empty/Loading/Success States */
.empty-state, .loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  color: var(--text-muted);
  font-size: 0.875rem;
}

.success-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 2rem;
  color: #4ade80;
  font-size: 0.875rem;
}

.success-icon {
  font-size: 1.25rem;
}

/* Stats Tab */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 0.75rem;
  margin-bottom: 1.5rem;
}

.stat-box {
  padding: 1rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  text-align: center;
}

.stat-number {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--accent-rust);
  font-family: 'JetBrains Mono', monospace;
}

.stat-label {
  font-size: 0.75rem;
  color: var(--text-muted);
  margin-top: 0.25rem;
}

.section {
  margin-bottom: 1.5rem;
}

.section-title {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
  margin-bottom: 0.75rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--border-primary);
}

.macro-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.macro-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
}

.macro-name {
  font-weight: 600;
  font-family: 'JetBrains Mono', monospace;
  color: var(--accent-rust);
}

.macro-type {
  font-size: 0.75rem;
  color: #60a5fa;
  background: rgba(96, 165, 250, 0.1);
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
}

.macro-binding {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.css-info {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.css-stat {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.css-badge {
  font-size: 0.625rem;
  padding: 0.125rem 0.5rem;
  border-radius: 3px;
}

.css-badge.scoped {
  background: rgba(167, 139, 250, 0.2);
  color: #a78bfa;
}

.css-badge.vbind {
  background: rgba(45, 212, 191, 0.2);
  color: #2dd4bf;
}

/* Bindings Tab */
.source-group {
  margin-bottom: 1.25rem;
}

.source-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
  padding-bottom: 0.375rem;
  border-bottom: 1px solid var(--border-primary);
}

.source-indicator {
  width: 10px;
  height: 10px;
  border-radius: 2px;
}

.src-props { background: #a78bfa; }
.src-emits { background: #f472b6; }
.src-model { background: #fb923c; }
.src-slots { background: #34d399; }
.src-ref { background: #4ade80; }
.src-reactive { background: #f87171; }
.src-computed { background: #2dd4bf; }
.src-import { background: #60a5fa; }
.src-local { background: #94a3b8; }
.src-function { background: #fbbf24; }
.src-class { background: #818cf8; }
.src-default { background: #6b7280; }

.source-name {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-primary);
}

.source-count {
  font-size: 0.625rem;
  padding: 0.0625rem 0.375rem;
  background: var(--bg-tertiary);
  border-radius: 8px;
  color: var(--text-muted);
}

.binding-grid {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.binding-item {
  display: grid;
  grid-template-columns: 1fr auto auto;
  gap: 0.75rem;
  align-items: center;
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  font-size: 0.75rem;
}

.binding-main {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.binding-name {
  font-weight: 600;
  font-family: 'JetBrains Mono', monospace;
  color: var(--text-primary);
}

.needs-value {
  font-size: 0.625rem;
  color: #4ade80;
  opacity: 0.7;
}

.binding-meta {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  color: var(--text-muted);
}

.binding-kind {
  font-family: 'JetBrains Mono', monospace;
}

.binding-type {
  color: #60a5fa;
}

.binding-flags {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.flag {
  font-size: 0.5rem;
  text-transform: uppercase;
  padding: 0.0625rem 0.25rem;
  border-radius: 2px;
}

.flag.active {
  background: rgba(74, 222, 128, 0.15);
  color: #4ade80;
}

.flag.inactive {
  background: var(--bg-tertiary);
  color: var(--text-muted);
  opacity: 0.5;
}

.refs {
  font-size: 0.625rem;
  color: var(--text-muted);
  font-family: 'JetBrains Mono', monospace;
}

/* Scopes Tab */
.scope-tree {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.scope-node {
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
}

.scope-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.scope-icon {
  font-size: 0.625rem;
  color: var(--text-muted);
}

.scope-kind {
  font-size: 0.75rem;
  font-weight: 600;
  color: #a78bfa;
}

.scope-range {
  font-size: 0.625rem;
  font-family: 'JetBrains Mono', monospace;
  color: var(--text-muted);
}

.scope-bindings {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
  margin-top: 0.375rem;
  padding-left: 1rem;
}

.scope-binding {
  font-size: 0.625rem;
  padding: 0.0625rem 0.375rem;
  background: var(--bg-tertiary);
  border-radius: 3px;
  font-family: 'JetBrains Mono', monospace;
  color: var(--text-secondary);
}

/* VIR Tab */
.vir-output {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.vir-header-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  background: linear-gradient(135deg, rgba(168, 85, 247, 0.15), rgba(59, 130, 246, 0.15));
  border: 1px solid rgba(168, 85, 247, 0.3);
  border-radius: 4px 4px 0 0;
  border-bottom: none;
}

.vir-title {
  font-size: 0.75rem;
  font-weight: 600;
  color: #a78bfa;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.vir-line-count {
  font-size: 0.625rem;
  color: var(--text-muted);
  font-family: 'JetBrains Mono', monospace;
}

.vir-content {
  flex: 1;
  display: flex;
  margin: 0;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 0 0 4px 4px;
  font-size: 0.8125rem;
  font-family: 'JetBrains Mono', monospace;
  overflow: auto;
  line-height: 1.6;
}

.vir-line-numbers {
  display: flex;
  flex-direction: column;
  padding: 0.75rem 0;
  background: rgba(0, 0, 0, 0.15);
  border-right: 1px solid var(--border-primary);
  user-select: none;
  flex-shrink: 0;
}

.vir-ln {
  padding: 0 0.75rem;
  text-align: right;
  color: var(--text-muted);
  font-size: 0.6875rem;
  min-width: 2.5rem;
  opacity: 0.6;
}

.vir-code {
  flex: 1;
  padding: 0.75rem 1rem;
  overflow-x: auto;
}

.vir-line {
  white-space: pre;
  min-height: 1.3em;
}

/* VIR Token Types - Fine-grained syntax highlighting */
.vir-token {
  /* Default */
}

.vir-border {
  color: #6366f1;
  opacity: 0.7;
}

.vir-section {
  color: #f472b6;
  font-weight: 700;
}

.vir-section-name {
  color: #60a5fa;
  font-weight: 700;
  letter-spacing: 0.05em;
}

.vir-macro {
  color: #f472b6;
  font-weight: 600;
}

.vir-type {
  color: #2dd4bf;
  font-style: italic;
}

.vir-binding {
  color: #4ade80;
  font-weight: 600;
}

.vir-identifier {
  color: #fbbf24;
}

.vir-tag {
  color: #a78bfa;
  font-weight: 500;
}

.vir-source {
  color: #fb923c;
  font-style: italic;
}

.vir-arrow {
  color: #f472b6;
  font-weight: 600;
}

.vir-number {
  color: #4ade80;
}

.vir-diagnostic {
  font-weight: 600;
}

.vir-line-diagnostic .vir-diagnostic {
  color: #f87171;
}

.vir-keyword {
  color: #60a5fa;
}

.vir-colon {
  color: var(--text-muted);
}

.vir-bracket {
  color: #9ca3af;
}

.vir-plain {
  color: var(--text-secondary);
}

/* Line type background hints */
.vir-line-section {
  background: rgba(96, 165, 250, 0.05);
  margin: 0.25em 0;
  padding: 0.125em 0;
}

.vir-line-macro {
  background: rgba(244, 114, 182, 0.03);
}

.vir-line-binding {
  background: rgba(74, 222, 128, 0.03);
}

.vir-line-diagnostic {
  background: rgba(248, 113, 113, 0.08);
}

/* Diagnostics Tab */
.diagnostic-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.diagnostic-item {
  padding: 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  border-left: 3px solid;
}

.diagnostic-item.severity-error {
  border-left-color: #ef4444;
}

.diagnostic-item.severity-warning {
  border-left-color: #f59e0b;
}

.diagnostic-header {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
}

.severity-icon {
  font-size: 0.875rem;
}

.severity-error .severity-icon { color: #ef4444; }
.severity-warning .severity-icon { color: #f59e0b; }

.diagnostic-message {
  flex: 1;
  font-size: 0.875rem;
  color: var(--text-primary);
}

.diagnostic-location {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: 0.375rem;
  padding-left: 1.25rem;
}

.location-range {
  font-size: 0.75rem;
  font-family: 'JetBrains Mono', monospace;
  color: var(--text-muted);
}

.diagnostic-code {
  font-size: 0.625rem;
  color: var(--text-secondary);
}

/* Mobile responsive */
@media (max-width: 768px) {
  .croquis-playground {
    grid-template-columns: 1fr;
    grid-template-rows: minmax(300px, 1fr) minmax(300px, 1fr);
    height: auto;
    min-height: 100%;
  }

  .panel {
    min-height: 300px;
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

  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
