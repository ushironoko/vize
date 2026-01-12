// WASM module loader for vize

export interface CompilerOptions {
  mode?: 'function' | 'module';
  ssr?: boolean;
  scopeId?: string | null;
  filename?: string;
  // Internal mock-only property for vapor mode detection
  outputMode?: 'vdom' | 'vapor';
  // Script extension: 'preserve' keeps TypeScript, 'downcompile' (default) transpiles to JS
  scriptExt?: 'preserve' | 'downcompile';
}

export interface CompileResult {
  code: string;
  preamble: string;
  ast: object;
  map?: object | null;
  helpers: string[];
  templates?: string[];
}

export interface SfcBlock {
  content: string;
  loc: { start: number; end: number };
  lang?: string;
  src?: string;
  attrs: Record<string, string>;
}

export interface SfcScriptBlock extends SfcBlock {
  setup: boolean;
}

export interface SfcStyleBlock extends SfcBlock {
  scoped: boolean;
  module?: string;
}

export interface SfcDescriptor {
  filename: string;
  source: string;
  template?: SfcBlock;
  script?: SfcScriptBlock;
  scriptSetup?: SfcScriptBlock;
  styles: SfcStyleBlock[];
  customBlocks: Array<{ type: string; content: string; attrs: Record<string, string> }>;
}

export interface SfcCompileResult {
  descriptor: SfcDescriptor;
  template?: CompileResult;
  script?: {
    code: string;
    bindings?: object;
  };
  css?: string;
  errors?: string[];
  warnings?: string[];
  bindingMetadata?: object;
}

export interface CssCompileOptions {
  scopeId?: string;
  scoped?: boolean;
  minify?: boolean;
  sourceMap?: boolean;
  filename?: string;
  targets?: {
    chrome?: number;
    firefox?: number;
    safari?: number;
    edge?: number;
    ios?: number;
    android?: number;
  };
}

export interface CssCompileResult {
  code: string;
  map?: string;
  cssVars: string[];
  errors: string[];
  warnings: string[];
}

// Musea types
export interface ArtParseOptions {
  filename?: string;
}

export interface ArtMetadata {
  title: string;
  description?: string;
  component?: string;
  category?: string;
  tags: string[];
  status: 'draft' | 'ready' | 'deprecated';
  order?: number;
}

export interface ArtVariant {
  name: string;
  template: string;
  isDefault: boolean;
  skipVrt: boolean;
  args?: Record<string, unknown>;
}

export interface ArtStyleBlock {
  content: string;
  scoped: boolean;
}

export interface ArtDescriptor {
  filename: string;
  metadata: ArtMetadata;
  variants: ArtVariant[];
  hasScriptSetup: boolean;
  hasScript: boolean;
  styleCount: number;
  styles: ArtStyleBlock[];
}

export interface CsfOutput {
  code: string;
  filename: string;
}

// Patina (Linter) types
export interface LintOptions {
  filename?: string;
  /** Rules to enable (if not set, all rules are enabled) */
  enabledRules?: string[];
  /** Override severity for specific rules */
  severityOverrides?: Record<string, 'error' | 'warning' | 'off'>;
  /** Locale for i18n messages (default: 'en') */
  locale?: 'en' | 'ja' | 'zh';
}

export interface LocaleInfo {
  code: string;
  name: string;
}

export interface LintDiagnostic {
  rule: string;
  severity: 'error' | 'warning';
  message: string;
  location: {
    start: { line: number; column: number; offset: number };
    end: { line: number; column: number; offset: number };
  };
  help?: string;
}

export interface LintResult {
  filename: string;
  errorCount: number;
  warningCount: number;
  diagnostics: LintDiagnostic[];
}

export interface LintRule {
  name: string;
  description: string;
  category: string;
  fixable: boolean;
  defaultSeverity: 'error' | 'warning';
}

// Glyph (Formatter) types
export interface FormatOptions {
  printWidth?: number;
  tabWidth?: number;
  useTabs?: boolean;
  semi?: boolean;
  singleQuote?: boolean;
  bracketSpacing?: boolean;
  bracketSameLine?: boolean;
  singleAttributePerLine?: boolean;
}

export interface FormatResult {
  code: string;
  changed: boolean;
}

// Analysis (Croquis) types
export interface AnalysisOptions {
  filename?: string;
}

// Binding source (where it comes from)
export type BindingSource =
  | 'props'
  | 'emits'
  | 'model'
  | 'slots'
  | 'ref'
  | 'reactive'
  | 'computed'
  | 'import'
  | 'local'
  | 'function'
  | 'class'
  | 'templateRef'
  | 'unknown';

// Binding metadata
export interface BindingMetadata {
  fromMacro?: string;
  isExported: boolean;
  isImported: boolean;
  isComponent: boolean;
  isDirective: boolean;
  needsValue: boolean;
  usedInTemplate: boolean;
  usedInScript: boolean;
  scopeDepth: number;
}

export interface BindingDisplay {
  name: string;
  kind: string;
  source: BindingSource;
  metadata: BindingMetadata;
  typeAnnotation?: string;
  start: number;
  end: number;
  isUsed: boolean;
  isMutated: boolean;
  referenceCount: number;
}

// Scope kind
export type ScopeKind =
  | 'module'
  | 'function'
  | 'arrowFunction'
  | 'block'
  | 'vFor'
  | 'vSlot'
  | 'class'
  | 'staticBlock'
  | 'catch'
  | 'setup';

export interface ScopeDisplay {
  id: number;
  parentId?: number;
  kind: ScopeKind;
  kindStr: string;
  start: number;
  end: number;
  bindings: string[];
  children: number[];
  depth: number;
}

export interface MacroDisplay {
  name: string;
  start: number;
  end: number;
  type_args?: string;
  args?: string;
  binding?: string;
}

export interface PropDisplay {
  name: string;
  type_annotation?: string;
  required: boolean;
  has_default: boolean;
}

export interface EmitDisplay {
  name: string;
  payload_type?: string;
}

export interface CssDisplay {
  selector_count: number;
  unused_selectors: Array<{ text: string; start: number; end: number }>;
  v_bind_count: number;
  is_scoped: boolean;
}

export interface AnalysisStats {
  binding_count: number;
  unused_binding_count: number;
  scope_count: number;
  macro_count: number;
  error_count: number;
  warning_count: number;
}

export interface AnalysisDiagnostic {
  severity: 'error' | 'warning' | 'info' | 'hint';
  message: string;
  start: number;
  end: number;
  code?: string;
  related: Array<{ message: string; start: number; end: number }>;
}

export interface AnalysisSummary {
  component_name?: string;
  is_setup: boolean;
  bindings: BindingDisplay[];
  scopes: ScopeDisplay[];
  macros: MacroDisplay[];
  props: PropDisplay[];
  emits: EmitDisplay[];
  css?: CssDisplay;
  diagnostics: AnalysisDiagnostic[];
  stats: AnalysisStats;
}

export interface AnalysisResult {
  summary: AnalysisSummary;
  diagnostics: AnalysisDiagnostic[];
  /** VIR (Vize Intermediate Representation) text format */
  vir?: string;
}

export interface WasmModule {
  compile: (template: string, options: CompilerOptions) => CompileResult;
  compileVapor: (template: string, options: CompilerOptions) => CompileResult;
  compileCss: (css: string, options: CssCompileOptions) => CssCompileResult;
  parseTemplate: (template: string, options: CompilerOptions) => object;
  parseSfc: (source: string, options: CompilerOptions) => SfcDescriptor;
  compileSfc: (source: string, options: CompilerOptions) => SfcCompileResult;
  // Analysis functions
  analyzeSfc: (source: string, options: AnalysisOptions) => AnalysisResult;
  // Musea functions
  parseArt: (source: string, options: ArtParseOptions) => ArtDescriptor;
  artToCsf: (source: string, options: ArtParseOptions) => CsfOutput;
  // Patina (Linter) functions
  lintTemplate: (source: string, options: LintOptions) => LintResult;
  lintSfc: (source: string, options: LintOptions) => LintResult;
  getLintRules: () => LintRule[];
  getLocales: () => LocaleInfo[];
  // Glyph (Formatter) functions
  formatSfc: (source: string, options: FormatOptions) => FormatResult;
  formatTemplate: (source: string, options: FormatOptions) => FormatResult;
  formatScript: (source: string, options: FormatOptions) => FormatResult;
  Compiler: new () => {
    compile: (template: string, options: CompilerOptions) => CompileResult;
    compileVapor: (template: string, options: CompilerOptions) => CompileResult;
    compileCss: (css: string, options: CssCompileOptions) => CssCompileResult;
    parse: (template: string, options: CompilerOptions) => object;
    parseSfc: (source: string, options: CompilerOptions) => SfcDescriptor;
    compileSfc: (source: string, options: CompilerOptions) => SfcCompileResult;
    analyzeSfc: (source: string, options: AnalysisOptions) => AnalysisResult;
  };
}

let wasmModule: WasmModule | null = null;
let loadPromise: Promise<WasmModule> | null = null;
let usingMock = false;

export async function loadWasm(): Promise<WasmModule> {
  if (wasmModule) {
    return wasmModule;
  }

  if (loadPromise) {
    return loadPromise;
  }

  loadPromise = (async () => {
    try {
      // Try to load the actual WASM module
      const wasm = await import('./vize_bindings.js');
      await wasm.default();

      // Get mock module to fill in any missing functions
      const mock = createMockModule();

      // Merge WASM module with mock fallbacks for missing functions
      wasmModule = {
        compile: wasm.compile || mock.compile,
        compileVapor: wasm.compileVapor || mock.compileVapor,
        compileCss: wasm.compileCss || mock.compileCss,
        parseTemplate: wasm.parseTemplate || mock.parseTemplate,
        parseSfc: wasm.parseSfc || mock.parseSfc,
        compileSfc: wasm.compileSfc || mock.compileSfc,
        // Analysis functions
        // Use mock analyzeSfc for enhanced scope detection (WASM version has limited scope support)
        analyzeSfc: mock.analyzeSfc,
        // Musea functions
        parseArt: wasm.parseArt || mock.parseArt,
        artToCsf: wasm.artToCsf || mock.artToCsf,
        // Patina (Linter) functions - may not be in WASM yet
        lintTemplate: wasm.lintTemplate || mock.lintTemplate,
        lintSfc: wasm.lintSfc || mock.lintSfc,
        getLintRules: wasm.getLintRules || mock.getLintRules,
        getLocales: wasm.getLocales || mock.getLocales,
        // Glyph (Formatter) functions - may not be in WASM yet
        formatSfc: wasm.formatSfc || mock.formatSfc,
        formatTemplate: wasm.formatTemplate || mock.formatTemplate,
        formatScript: wasm.formatScript || mock.formatScript,
        Compiler: wasm.Compiler || mock.Compiler,
      };
      usingMock = false;
      return wasmModule;
    } catch (e) {
      console.warn('WASM module not found, using mock compiler:', e);
      // Return mock module if WASM is not available
      wasmModule = createMockModule();
      usingMock = true;
      return wasmModule;
    }
  })();

  return loadPromise;
}

// Mock module for development when WASM is not built
function createMockModule(): WasmModule {
  const mockCompile = (template: string, options: CompilerOptions): CompileResult => {
    const isVapor = options.outputMode === 'vapor';
    const hasInterpolation = template.includes('{{');
    const hasElement = template.includes('<');
    const hasVIf = template.includes('v-if');
    const hasVFor = template.includes('v-for');
    const hasVOn = template.includes('@') || template.includes('v-on');

    let code = '';
    const helpers: string[] = [];

    if (isVapor) {
      // Vapor mode output
      code = `// Vapor Mode Output\n`;
      code += `import { template, createTextNode, setText, renderEffect } from 'vue/vapor';\n\n`;
      code += `const t0 = template(\`${template.replace(/\{\{[^}]+\}\}/g, '<!>')}\`);\n\n`;
      code += `export function render(_ctx) {\n`;
      code += `  const n0 = t0();\n`;

      if (hasInterpolation) {
        code += `  const x0 = createTextNode();\n`;
        code += `  renderEffect(() => setText(x0, _ctx.msg));\n`;
        helpers.push('template', 'createTextNode', 'setText', 'renderEffect');
      }

      code += `  return n0;\n`;
      code += `}`;
    } else {
      // VDom mode output
      helpers.push('createElementVNode');
      if (hasInterpolation) helpers.push('toDisplayString');
      if (hasVIf) helpers.push('createCommentVNode', 'Fragment', 'openBlock', 'createElementBlock');
      if (hasVFor) helpers.push('renderList', 'Fragment', 'openBlock', 'createElementBlock');
      if (hasVOn) helpers.push('withModifiers');

      const isModule = options.mode === 'module';

      if (isModule) {
        code = `import { ${helpers.join(', ')} } from "vue"\n\n`;
        code += `export function render(_ctx, _cache, $props, $setup, $data, $options) {\n`;
      } else {
        code = `function render(_ctx, _cache) {\n`;
      }

      code += `  return `;

      if (hasElement) {
        const tagMatch = template.match(/<(\w+)/);
        const tag = tagMatch ? tagMatch[1] : 'div';

        if (hasVIf) {
          code += `(_ctx.show)\n`;
          code += `    ? (_openBlock(), _createElementBlock("${tag}", { key: 0 }))\n`;
          code += `    : _createCommentVNode("v-if", true)`;
        } else if (hasVFor) {
          code += `(_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.items, (item) => {\n`;
          code += `    return (_openBlock(), _createElementBlock("${tag}", { key: item.id }))\n`;
          code += `  }), 128))`;
        } else if (hasInterpolation) {
          const expr = template.match(/\{\{\s*([^}]+)\s*\}\}/)?.[1] || 'msg';
          code += `_createElementVNode("${tag}", null, _toDisplayString(_ctx.${expr.trim()}), 1 /* TEXT */)`;
        } else {
          code += `_createElementVNode("${tag}", null, null, -1 /* HOISTED */)`;
        }
      } else if (hasInterpolation) {
        const expr = template.match(/\{\{\s*([^}]+)\s*\}\}/)?.[1] || '';
        code += `_toDisplayString(_ctx.${expr.trim()})`;
      } else {
        code += `"${template}"`;
      }

      code += `\n}`;
    }

    // Build AST representation
    const ast = buildMockAst(template);

    return {
      code,
      preamble: options.mode === 'function' ? `const { ${helpers.join(', ')} } = Vue\n` : '',
      ast,
      helpers,
    };
  };

  const mockParseSfc = (source: string, _options: CompilerOptions): SfcDescriptor => {
    const templateMatch = source.match(/<template>([\s\S]*?)<\/template>/);
    const scriptMatch = source.match(/<script>([\s\S]*?)<\/script>/);
    const scriptSetupMatch = source.match(/<script\s+setup>([\s\S]*?)<\/script>/);
    const styleMatches = [...source.matchAll(/<style(\s+scoped)?>([\s\S]*?)<\/style>/g)];

    return {
      filename: 'anonymous.vue',
      source,
      template: templateMatch ? {
        content: templateMatch[1],
        loc: { start: 0, end: 0 },
        attrs: {},
      } : undefined,
      script: scriptMatch ? {
        content: scriptMatch[1],
        loc: { start: 0, end: 0 },
        attrs: {},
        setup: false,
      } : undefined,
      scriptSetup: scriptSetupMatch ? {
        content: scriptSetupMatch[1],
        loc: { start: 0, end: 0 },
        attrs: {},
        setup: true,
      } : undefined,
      styles: styleMatches.map(m => ({
        content: m[2],
        loc: { start: 0, end: 0 },
        attrs: {},
        scoped: !!m[1],
      })),
      customBlocks: [],
    };
  };

  const mockCompileSfc = (source: string, options: CompilerOptions): SfcCompileResult => {
    const descriptor = mockParseSfc(source, options);
    const templateResult = descriptor.template
      ? mockCompile(descriptor.template.content, options)
      : undefined;

    // Generate mock script code and extract bindings
    let scriptCode = '';
    const bindings: Record<string, string> = {};

    if (descriptor.scriptSetup) {
      scriptCode = `// Mock compiled script setup\n`;
      scriptCode += `import { ref, computed } from 'vue'\n\n`;
      scriptCode += `export default {\n`;
      scriptCode += `  setup() {\n`;
      scriptCode += `    ${descriptor.scriptSetup.content.split('\n').filter(l => !l.trim().startsWith('import')).join('\n    ')}\n`;
      scriptCode += `    return { /* bindings */ }\n`;
      scriptCode += `  }\n`;
      scriptCode += `}\n`;
      if (templateResult) {
        scriptCode += `\n${templateResult.code}`;
      }

      // Extract mock bindings from script setup content
      const lines = descriptor.scriptSetup.content.split('\n');
      for (const line of lines) {
        // Match: const x = ref(...), const x = computed(...), or defineProps destructure
        const constMatch = line.match(/const\s+(\w+)\s*=/);
        if (constMatch) {
          const varName = constMatch[1];
          if (line.includes('ref(')) {
            bindings[varName] = 'setup-ref';
          } else if (line.includes('computed(')) {
            bindings[varName] = 'setup-computed';
          } else {
            bindings[varName] = 'setup-const';
          }
        }
      }
    } else if (descriptor.script) {
      scriptCode = descriptor.script.content;
    } else {
      scriptCode = 'export default {}';
    }

    return {
      descriptor,
      template: templateResult,
      script: { code: scriptCode },
      css: descriptor.styles.map(s => s.content).join('\n') || undefined,
      bindingMetadata: Object.keys(bindings).length > 0 ? bindings : undefined,
    };
  };

  const mockCompileCss = (css: string, options: CssCompileOptions): CssCompileResult => {
    let code = css;

    // Apply scoping if requested
    if (options.scoped && options.scopeId) {
      // Simple mock scoping - just append attribute selector
      code = css.replace(/([.#]?\w+)(\s*\{)/g, `$1[${options.scopeId}]$2`);
    }

    // Minify if requested
    if (options.minify) {
      code = code
        .replace(/\s+/g, ' ')
        .replace(/\s*{\s*/g, '{')
        .replace(/\s*}\s*/g, '}')
        .replace(/\s*:\s*/g, ':')
        .replace(/\s*;\s*/g, ';')
        .trim();
    }

    // Extract v-bind() expressions
    const cssVars: string[] = [];
    const vBindRegex = /v-bind\(([^)]+)\)/g;
    let match;
    while ((match = vBindRegex.exec(css)) !== null) {
      const expr = match[1].trim().replace(/['"]/g, '');
      cssVars.push(expr);
    }

    return {
      code,
      cssVars,
      errors: [],
      warnings: [],
    };
  };

  // Mock analyze function - defined inline to avoid hoisting issues
  const inlineMockAnalyzeSfc = (source: string, _options: AnalysisOptions): AnalysisResult => {
    // Parse the SFC to extract information
    const hasScriptSetup = source.includes('<script setup');
    const hasDefineProps = source.includes('defineProps');
    const hasDefineEmits = source.includes('defineEmits');
    const hasScoped = source.includes('<style scoped');

    const bindings: BindingDisplay[] = [];
    const macros: MacroDisplay[] = [];
    const props: PropDisplay[] = [];
    const emits: EmitDisplay[] = [];

    // Extract ref bindings
    const refMatches = source.matchAll(/const\s+(\w+)\s*=\s*ref\(/g);
    for (const match of refMatches) {
      bindings.push({
        name: match[1],
        kind: 'SetupRef',
        source: 'ref' as BindingSource,
        metadata: {
          isExported: false,
          isImported: false,
          isComponent: false,
          isDirective: false,
          needsValue: true,
          usedInTemplate: true,
          usedInScript: true,
          scopeDepth: 0,
        },
        start: match.index || 0,
        end: (match.index || 0) + match[0].length,
        isUsed: true,
        isMutated: true,
        referenceCount: 1,
      });
    }

    // Extract computed bindings
    const computedMatches = source.matchAll(/const\s+(\w+)\s*=\s*computed\(/g);
    for (const match of computedMatches) {
      bindings.push({
        name: match[1],
        kind: 'SetupComputed',
        source: 'computed' as BindingSource,
        metadata: {
          isExported: false,
          isImported: false,
          isComponent: false,
          isDirective: false,
          needsValue: true,
          usedInTemplate: true,
          usedInScript: true,
          scopeDepth: 0,
        },
        start: match.index || 0,
        end: (match.index || 0) + match[0].length,
        isUsed: true,
        isMutated: false,
        referenceCount: 1,
      });
    }

    // Extract function bindings
    const functionMatches = source.matchAll(/function\s+(\w+)\s*\(/g);
    for (const match of functionMatches) {
      bindings.push({
        name: match[1],
        kind: 'SetupConst',
        source: 'function' as BindingSource,
        metadata: {
          isExported: false,
          isImported: false,
          isComponent: false,
          isDirective: false,
          needsValue: false,
          usedInTemplate: true,
          usedInScript: true,
          scopeDepth: 0,
        },
        start: match.index || 0,
        end: (match.index || 0) + match[0].length,
        isUsed: true,
        isMutated: false,
        referenceCount: 1,
      });
    }

    // Extract defineProps
    if (hasDefineProps) {
      const propsMatch = source.match(/defineProps<\{([^}]+)\}>/);
      if (propsMatch) {
        macros.push({
          name: 'defineProps',
          start: propsMatch.index || 0,
          end: (propsMatch.index || 0) + propsMatch[0].length,
          type_args: propsMatch[1],
        });
        // Extract prop names from type
        const propNameMatches = propsMatch[1].matchAll(/(\w+)(\?)?:/g);
        for (const propMatch of propNameMatches) {
          props.push({
            name: propMatch[1],
            required: !propMatch[2],
            has_default: false,
          });
        }
      }
    }

    // Extract defineEmits
    if (hasDefineEmits) {
      const emitsMatch = source.match(/defineEmits<\{([^}]+)\}>/);
      if (emitsMatch) {
        macros.push({
          name: 'defineEmits',
          start: emitsMatch.index || 0,
          end: (emitsMatch.index || 0) + emitsMatch[0].length,
          type_args: emitsMatch[1],
        });
        // Extract emit names from type
        const emitNameMatches = emitsMatch[1].matchAll(/(\w+):/g);
        for (const emitMatch of emitNameMatches) {
          emits.push({
            name: emitMatch[1],
          });
        }
      }
    }

    // Generate VIR text (formal TOML-like format)
    let vir = '';
    vir += '# VIR v0.1\n';
    vir += '\n';

    // Stats section
    vir += '[stats]\n';
    vir += `bindings = ${bindings.length}\n`;
    vir += `macros = ${macros.length}\n`;

    // Macros section
    if (macros.length > 0) {
      vir += '\n[macros]\n';
      for (const macro of macros) {
        vir += `@${macro.name}`;
        if (macro.type_args) {
          vir += `<{\n${macro.type_args.trim()}\n}>`;
        }
        if (macro.identifier) {
          vir += ` -> ${macro.identifier}`;
        }
        vir += ` # ${macro.start}:${macro.end}\n`;
      }
    }

    // Bindings section
    if (bindings.length > 0) {
      vir += '\n[bindings]\n';
      for (const binding of bindings) {
        const flags: string[] = [];
        if (binding.kind === 'SetupRef') flags.push('ref');
        if (binding.metadata?.isUsed !== false) flags.push('used');
        if (binding.metadata?.isMutated) flags.push('mut');
        vir += `${binding.name}: ${binding.source} @0:0`;
        if (flags.length > 0) {
          vir += ` [${flags.join(', ')}]`;
        }
        vir += '\n';
      }
    }

    // CSS section
    const selectorCount = (source.match(/[.#\w][\w-]*\s*\{/g) || []).length;
    const vBindCount = (source.match(/v-bind\(/g) || []).length;
    if (hasScoped) {
      vir += '\n[css]\n';
      vir += `scoped = true\n`;
      vir += `selectors = ${selectorCount}\n`;
      vir += `v_bind = ${vBindCount}\n`;
    }

    // Props section
    if (props.length > 0) {
      vir += '\n[props]\n';
      for (const prop of props) {
        vir += `${prop.name}: ${prop.type || 'any'}\n`;
      }
    }

    // Emits section
    if (emits.length > 0) {
      vir += '\n[emits]\n';
      for (const emit of emits) {
        vir += `${emit.name}\n`;
      }
    }

    // Generate scopes from source analysis
    const scopes: ScopeDisplay[] = [];
    let scopeId = 0;

    // Module scope (root)
    const moduleScope: ScopeDisplay = {
      id: scopeId++,
      kind: 'module',
      kindStr: 'Module',
      start: 0,
      end: source.length,
      bindings: bindings.map(b => b.name),
      children: [],
      depth: 0,
    };
    scopes.push(moduleScope);

    // Detect setup scope if script setup exists
    if (hasScriptSetup) {
      const scriptSetupMatch = source.match(/<script[^>]*setup[^>]*>([\s\S]*?)<\/script>/);
      if (scriptSetupMatch) {
        const setupStart = source.indexOf(scriptSetupMatch[0]);
        const setupEnd = setupStart + scriptSetupMatch[0].length;
        const setupScope: ScopeDisplay = {
          id: scopeId++,
          parentId: 0,
          kind: 'setup',
          kindStr: 'Setup',
          start: setupStart,
          end: setupEnd,
          bindings: bindings.map(b => b.name),
          children: [],
          depth: 1,
        };
        moduleScope.children.push(setupScope.id);
        scopes.push(setupScope);

        // Detect function scopes inside setup
        const functionRegex = /function\s+(\w+)\s*\([^)]*\)\s*\{/g;
        let funcMatch;
        while ((funcMatch = functionRegex.exec(scriptSetupMatch[1])) !== null) {
          const funcScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'function',
            kindStr: `Function (${funcMatch[1]})`,
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 50,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(funcScope.id);
          scopes.push(funcScope);
        }

        // Detect arrow function scopes
        const arrowRegex = /const\s+(\w+)\s*=\s*\([^)]*\)\s*=>/g;
        while ((funcMatch = arrowRegex.exec(scriptSetupMatch[1])) !== null) {
          const arrowScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: `Arrow (${funcMatch[1]})`,
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 50,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(arrowScope.id);
          scopes.push(arrowScope);
        }

        // Detect watch callbacks
        const watchRegex = /watch\s*\([^,]+,\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = watchRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const watchScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'watch',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 30,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(watchScope.id);
          scopes.push(watchScope);
        }

        // Detect watchEffect callbacks
        const watchEffectRegex = /watchEffect\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = watchEffectRegex.exec(scriptSetupMatch[1])) !== null) {
          const watchEffectScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'watchEffect',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 30,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(watchEffectScope.id);
          scopes.push(watchEffectScope);
        }

        // Detect computed callbacks
        const computedRegex = /(?:const|let)\s+(\w+)\s*=\s*computed\s*\(\s*(?:\([^)]*\)\s*)?=>/g;
        while ((funcMatch = computedRegex.exec(scriptSetupMatch[1])) !== null) {
          const computedScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: `computed (${funcMatch[1]})`,
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 30,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(computedScope.id);
          scopes.push(computedScope);
        }

        // Detect computed with getter/setter
        const computedGetSetRegex = /(?:const|let)\s+(\w+)\s*=\s*computed\s*\(\s*\{/g;
        while ((funcMatch = computedGetSetRegex.exec(scriptSetupMatch[1])) !== null) {
          const computedScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'function',
            kindStr: `computed (${funcMatch[1]}) [get/set]`,
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 50,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(computedScope.id);
          scopes.push(computedScope);
        }

        // Detect lifecycle hooks
        const lifecycleHooks = ['onMounted', 'onUnmounted', 'onBeforeMount', 'onBeforeUnmount', 'onUpdated', 'onBeforeUpdate', 'onActivated', 'onDeactivated', 'onErrorCaptured', 'onRenderTracked', 'onRenderTriggered', 'onServerPrefetch'];
        for (const hook of lifecycleHooks) {
          const hookRegex = new RegExp(`${hook}\\s*\\(\\s*(?:async\\s*)?\\(?([^)]*)\\)?\\s*=>`, 'g');
          while ((funcMatch = hookRegex.exec(scriptSetupMatch[1])) !== null) {
            const hookScope: ScopeDisplay = {
              id: scopeId++,
              parentId: setupScope.id,
              kind: 'arrowFunction',
              kindStr: hook,
              start: setupStart + funcMatch.index,
              end: setupStart + funcMatch.index + funcMatch[0].length + 30,
              bindings: [],
              children: [],
              depth: 2,
            };
            setupScope.children.push(hookScope.id);
            scopes.push(hookScope);
          }
        }

        // Detect provide with factory function
        const provideRegex = /provide\s*\(\s*['"][^'"]+['"]\s*,\s*\(\)\s*=>/g;
        while ((funcMatch = provideRegex.exec(scriptSetupMatch[1])) !== null) {
          const provideScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'provide factory',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(provideScope.id);
          scopes.push(provideScope);
        }

        // Detect inject with default factory
        const injectRegex = /inject\s*\(\s*['"][^'"]+['"]\s*,\s*\(\)\s*=>/g;
        while ((funcMatch = injectRegex.exec(scriptSetupMatch[1])) !== null) {
          const injectScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'inject default',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(injectScope.id);
          scopes.push(injectScope);
        }

        // Detect try-catch blocks
        const tryCatchRegex = /try\s*\{/g;
        while ((funcMatch = tryCatchRegex.exec(scriptSetupMatch[1])) !== null) {
          const tryScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'block',
            kindStr: 'try',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 30,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(tryScope.id);
          scopes.push(tryScope);
        }

        const catchRegex = /catch\s*\(\s*(\w+)\s*\)\s*\{/g;
        while ((funcMatch = catchRegex.exec(scriptSetupMatch[1])) !== null) {
          const catchScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'block',
            kindStr: 'catch',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 30,
            bindings: [funcMatch[1]],
            children: [],
            depth: 2,
          };
          setupScope.children.push(catchScope.id);
          scopes.push(catchScope);
        }

        // Detect for loops
        const forLoopRegex = /for\s*\(\s*(?:const|let|var)\s+(\w+)/g;
        while ((funcMatch = forLoopRegex.exec(scriptSetupMatch[1])) !== null) {
          const forScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'block',
            kindStr: 'for',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 30,
            bindings: [funcMatch[1]],
            children: [],
            depth: 2,
          };
          setupScope.children.push(forScope.id);
          scopes.push(forScope);
        }

        // Detect for...of / for...in loops
        const forOfInRegex = /for\s*\(\s*(?:const|let|var)\s+(\w+)\s+(?:of|in)\s+/g;
        while ((funcMatch = forOfInRegex.exec(scriptSetupMatch[1])) !== null) {
          const forOfScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'block',
            kindStr: 'for..of/in',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 30,
            bindings: [funcMatch[1]],
            children: [],
            depth: 2,
          };
          setupScope.children.push(forOfScope.id);
          scopes.push(forOfScope);
        }

        // Detect if blocks with block-scoped variables
        const ifLetRegex = /if\s*\([^)]+\)\s*\{[^}]*(?:const|let)\s+(\w+)/g;
        while ((funcMatch = ifLetRegex.exec(scriptSetupMatch[1])) !== null) {
          const ifScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'block',
            kindStr: 'if',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: [funcMatch[1]],
            children: [],
            depth: 2,
          };
          setupScope.children.push(ifScope.id);
          scopes.push(ifScope);
        }

        // Detect Array.forEach callbacks
        const forEachRegex = /\.forEach\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = forEachRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const forEachScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'forEach',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(forEachScope.id);
          scopes.push(forEachScope);
        }

        // Detect Array.map callbacks
        const mapRegex = /\.map\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = mapRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const mapScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'map',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(mapScope.id);
          scopes.push(mapScope);
        }

        // Detect Array.filter callbacks
        const filterRegex = /\.filter\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = filterRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const filterScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'filter',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(filterScope.id);
          scopes.push(filterScope);
        }

        // Detect Array.reduce callbacks
        const reduceRegex = /\.reduce\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = reduceRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const reduceScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'reduce',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(reduceScope.id);
          scopes.push(reduceScope);
        }

        // Detect Array.find/findIndex callbacks
        const findRegex = /\.find(?:Index)?\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = findRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const findScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'find',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(findScope.id);
          scopes.push(findScope);
        }

        // Detect Array.some/every callbacks
        const someEveryRegex = /\.(?:some|every)\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = someEveryRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const someEveryScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'some/every',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(someEveryScope.id);
          scopes.push(someEveryScope);
        }

        // Detect Promise.then callbacks
        const thenRegex = /\.then\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = thenRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const thenScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: '.then',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(thenScope.id);
          scopes.push(thenScope);
        }

        // Detect Promise.catch callbacks
        const promiseCatchRegex = /\.catch\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = promiseCatchRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const promiseCatchScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: '.catch',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(promiseCatchScope.id);
          scopes.push(promiseCatchScope);
        }

        // Detect Promise.finally callbacks
        const finallyRegex = /\.finally\s*\(\s*\(\)\s*=>/g;
        while ((funcMatch = finallyRegex.exec(scriptSetupMatch[1])) !== null) {
          const finallyScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: '.finally',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(finallyScope.id);
          scopes.push(finallyScope);
        }

        // Detect setTimeout/setInterval callbacks
        const timerRegex = /set(?:Timeout|Interval)\s*\(\s*\(\)\s*=>/g;
        while ((funcMatch = timerRegex.exec(scriptSetupMatch[1])) !== null) {
          const timerScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'timer',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(timerScope.id);
          scopes.push(timerScope);
        }

        // Detect nextTick callbacks
        const nextTickRegex = /nextTick\s*\(\s*\(\)\s*=>/g;
        while ((funcMatch = nextTickRegex.exec(scriptSetupMatch[1])) !== null) {
          const nextTickScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'nextTick',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(nextTickScope.id);
          scopes.push(nextTickScope);
        }

        // Detect async IIFE
        const asyncIifeRegex = /\(\s*async\s*\(\)\s*=>\s*\{/g;
        while ((funcMatch = asyncIifeRegex.exec(scriptSetupMatch[1])) !== null) {
          const asyncIifeScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'async IIFE',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 30,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(asyncIifeScope.id);
          scopes.push(asyncIifeScope);
        }
      }
    }

    // Detect v-for scopes in template
    const templateMatch = source.match(/<template>([\s\S]*?)<\/template>/);
    if (templateMatch) {
      const templateStart = source.indexOf(templateMatch[0]);
      const vForRegex = /v-for="([^"]+)"/g;
      let vForMatch;
      while ((vForMatch = vForRegex.exec(templateMatch[1])) !== null) {
        const expr = vForMatch[1];
        const inMatch = expr.match(/\(?([^)]+)\)?\s+(?:in|of)\s+/);
        const vForBindings: string[] = [];
        if (inMatch) {
          const aliases = inMatch[1].split(',').map(s => s.trim());
          vForBindings.push(...aliases);
        }

        const vForScope: ScopeDisplay = {
          id: scopeId++,
          parentId: 0,
          kind: 'vFor',
          kindStr: `v-for`,
          start: templateStart + vForMatch.index,
          end: templateStart + vForMatch.index + vForMatch[0].length,
          bindings: vForBindings,
          children: [],
          depth: 1,
        };
        moduleScope.children.push(vForScope.id);
        scopes.push(vForScope);
      }

      // Detect v-slot scopes
      const vSlotRegex = /v-slot(?::(\w+))?="([^"]+)"/g;
      let vSlotMatch;
      while ((vSlotMatch = vSlotRegex.exec(templateMatch[1])) !== null) {
        const slotName = vSlotMatch[1] || 'default';
        const slotParams = vSlotMatch[2]?.match(/\{?\s*([^}]+)\s*\}?/)?.[1]?.split(',').map(s => s.trim()) || [];
        const vSlotScope: ScopeDisplay = {
          id: scopeId++,
          parentId: 0,
          kind: 'vSlot',
          kindStr: `v-slot:${slotName}`,
          start: templateStart + vSlotMatch.index,
          end: templateStart + vSlotMatch.index + vSlotMatch[0].length,
          bindings: slotParams,
          children: [],
          depth: 1,
        };
        moduleScope.children.push(vSlotScope.id);
        scopes.push(vSlotScope);
      }

      // Detect inline event handler scopes
      const eventRegex = /@(\w+)="([^"]+)"/g;
      let eventMatch;
      while ((eventMatch = eventRegex.exec(templateMatch[1])) !== null) {
        const handler = eventMatch[2];
        if (handler.includes('=>') || handler.includes('$event')) {
          const eventScope: ScopeDisplay = {
            id: scopeId++,
            parentId: 0,
            kind: 'arrowFunction',
            kindStr: `@${eventMatch[1]} handler`,
            start: templateStart + eventMatch.index,
            end: templateStart + eventMatch.index + eventMatch[0].length,
            bindings: ['$event'],
            children: [],
            depth: 1,
          };
          moduleScope.children.push(eventScope.id);
          scopes.push(eventScope);
        }
      }
    }

    // Add scopes to VIR
    if (scopes.length > 0) {
      vir += '\n[scopes]\n';
      for (const scope of scopes) {
        vir += `#${scope.id} ${scope.kindStr.toLowerCase()} @${scope.start}:${scope.end}`;
        if (scope.bindings.length > 0) {
          vir += ` {${scope.bindings.join(', ')}}`;
        }
        vir += '\n';
      }
    }

    // Update stats with scope count
    vir = vir.replace('[stats]\n', `[stats]\nscopes = ${scopes.length}\n`);

    const summary: AnalysisSummary = {
      is_setup: hasScriptSetup,
      bindings,
      scopes,
      macros,
      props,
      emits,
      css: hasScoped ? {
        selector_count: (source.match(/[.#\w][\w-]*\s*\{/g) || []).length,
        unused_selectors: [],
        v_bind_count: (source.match(/v-bind\(/g) || []).length,
        is_scoped: true,
      } : undefined,
      diagnostics: [],
      stats: {
        binding_count: bindings.length,
        unused_binding_count: 0,
        scope_count: scopes.length,
        macro_count: macros.length,
        error_count: 0,
        warning_count: 0,
      },
    };

    return {
      summary,
      diagnostics: [],
      vir,
    };
  };

  class MockCompiler {
    compile(template: string, options: CompilerOptions): CompileResult {
      return mockCompile(template, options);
    }
    compileVapor(template: string, options: CompilerOptions): CompileResult {
      return mockCompile(template, { ...options, outputMode: 'vapor' });
    }
    compileCss(css: string, options: CssCompileOptions): CssCompileResult {
      return mockCompileCss(css, options);
    }
    parse(template: string, _options: CompilerOptions): object {
      return buildMockAst(template);
    }
    parseSfc(source: string, options: CompilerOptions): SfcDescriptor {
      return mockParseSfc(source, options);
    }
    compileSfc(source: string, options: CompilerOptions): SfcCompileResult {
      return mockCompileSfc(source, options);
    }
    analyzeSfc(source: string, options: AnalysisOptions): AnalysisResult {
      return inlineMockAnalyzeSfc(source, options);
    }
  }

  const mockParseArt = (source: string, options: ArtParseOptions): ArtDescriptor => {
    // Simple mock parsing for Art files
    const titleMatch = source.match(/title="([^"]+)"/);
    const descMatch = source.match(/description="([^"]+)"/);
    const componentMatch = source.match(/component="([^"]+)"/);
    const categoryMatch = source.match(/category="([^"]+)"/);

    // Extract variants
    const variantRegex = /<variant\s+name="([^"]+)"([^>]*)>([\s\S]*?)<\/variant>/g;
    const variants: ArtVariant[] = [];
    let match;
    while ((match = variantRegex.exec(source)) !== null) {
      const [, name, attrs, template] = match;
      variants.push({
        name,
        template: template.trim(),
        isDefault: attrs.includes('default'),
        skipVrt: attrs.includes('skip-vrt'),
      });
    }

    // Extract style blocks
    const styleRegex = /<style(\s+scoped)?>([\s\S]*?)<\/style>/g;
    const styles: ArtStyleBlock[] = [];
    let styleMatch;
    while ((styleMatch = styleRegex.exec(source)) !== null) {
      styles.push({
        content: styleMatch[2],
        scoped: !!styleMatch[1],
      });
    }

    return {
      filename: options.filename || 'anonymous.art.vue',
      metadata: {
        title: titleMatch?.[1] || 'Untitled',
        description: descMatch?.[1],
        component: componentMatch?.[1],
        category: categoryMatch?.[1],
        tags: [],
        status: 'ready',
      },
      variants,
      hasScriptSetup: source.includes('\x3Cscript setup'),
      hasScript: source.includes('\x3Cscript') && !source.includes('\x3Cscript setup'),
      styleCount: styles.length,
      styles,
    };
  };

  const mockArtToCsf = (source: string, options: ArtParseOptions): CsfOutput => {
    const art = mockParseArt(source, options);
    const componentPath = art.metadata.component || './Component.vue';

    let code = `import type { Meta, StoryObj } from '@storybook/vue3';\n`;
    code += `import Component from '${componentPath}';\n\n`;
    code += `const meta: Meta<typeof Component> = {\n`;
    code += `  title: '${art.metadata.title}',\n`;
    code += `  component: Component,\n`;
    code += `  tags: ['autodocs'],\n`;
    code += `};\n\n`;
    code += `export default meta;\n`;
    code += `type Story = StoryObj<typeof meta>;\n\n`;

    for (const variant of art.variants) {
      const exportName = variant.name.replace(/\s+/g, '');
      code += `export const ${exportName}: Story = {\n`;
      code += `  name: '${variant.name}',\n`;
      code += `  render: (args) => ({\n`;
      code += `    components: { Component },\n`;
      code += `    setup() { return { args }; },\n`;
      code += `    template: \`${variant.template.replace(/`/g, '\\`')}\`,\n`;
      code += `  }),\n`;
      code += `};\n\n`;
    }

    const baseName = (options.filename || 'Component').replace('.art.vue', '');
    return {
      code,
      filename: `${baseName}.stories.ts`,
    };
  };

  // Helper to calculate line/column from character offset
  function getPositionFromOffset(source: string, offset: number): { line: number; column: number } {
    const lines = source.substring(0, offset).split('\n');
    return {
      line: lines.length,
      column: lines[lines.length - 1].length + 1, // 1-indexed column
    };
  }

  // Mock lint functions
  const mockLintTemplate = (source: string, options: LintOptions): LintResult => {
    const allDiagnostics: LintDiagnostic[] = [];
    const filename = options.filename || 'anonymous.vue';

    // Helper to check if a rule is enabled
    const isRuleEnabled = (ruleName: string): boolean => {
      if (!options.enabledRules || options.enabledRules.length === 0) {
        return true; // All rules enabled by default
      }
      return options.enabledRules.includes(ruleName);
    };

    // Helper to get severity for a rule
    const getSeverity = (ruleName: string, defaultSeverity: 'error' | 'warning'): 'error' | 'warning' | 'off' => {
      if (options.severityOverrides && ruleName in options.severityOverrides) {
        return options.severityOverrides[ruleName];
      }
      return defaultSeverity;
    };

    // Simple mock lint rules
    // Check for v-for without :key - find elements with v-for but no :key on same element
    const vForRegex = /<(\w+)[^>]*v-for="[^"]+"/g;
    let vForMatch;
    while ((vForMatch = vForRegex.exec(source)) !== null) {
      const ruleName = 'vue/require-v-for-key';
      if (!isRuleEnabled(ruleName)) continue;
      const severity = getSeverity(ruleName, 'error');
      if (severity === 'off') continue;

      // Check if this element has a :key
      const elementEnd = source.indexOf('>', vForMatch.index);
      const elementStr = source.substring(vForMatch.index, elementEnd + 1);
      if (!elementStr.includes(':key=')) {
        const startPos = getPositionFromOffset(source, vForMatch.index);
        const endOffset = vForMatch.index + vForMatch[0].length;
        const endPos = getPositionFromOffset(source, endOffset);
        allDiagnostics.push({
          rule: ruleName,
          severity,
          message: `Elements in iteration expect to have 'v-bind:key' directives. Element: <${vForMatch[1]}>`,
          location: {
            start: { line: startPos.line, column: startPos.column, offset: vForMatch.index },
            end: { line: endPos.line, column: endPos.column, offset: endOffset },
          },
          help: 'Add a `:key` attribute with a unique identifier for each item',
        });
      }
    }

    // Check for v-if with v-for on same element
    const vIfWithVForRuleName = 'vue/no-use-v-if-with-v-for';
    if (isRuleEnabled(vIfWithVForRuleName)) {
      const vIfWithVForSeverity = getSeverity(vIfWithVForRuleName, 'warning');
      if (vIfWithVForSeverity !== 'off') {
        const vIfWithVForRegex = /<(\w+)[^>]*v-for="[^"]*"[^>]*v-if="[^"]*"/g;
        let vIfWithVFor;
        while ((vIfWithVFor = vIfWithVForRegex.exec(source)) !== null) {
          const startPos = getPositionFromOffset(source, vIfWithVFor.index);
          const endOffset = vIfWithVFor.index + vIfWithVFor[0].length;
          const endPos = getPositionFromOffset(source, endOffset);
          allDiagnostics.push({
            rule: vIfWithVForRuleName,
            severity: vIfWithVForSeverity,
            message: 'Avoid using `v-if` with `v-for` on the same element. Use a computed property to filter the list instead.',
            location: {
              start: { line: startPos.line, column: startPos.column, offset: vIfWithVFor.index },
              end: { line: endPos.line, column: endPos.column, offset: endOffset },
            },
            help: 'Use a computed property to pre-filter the list, e.g., `computed: { activeItems() { return items.filter(i => i.active) } }`',
          });
        }
      }
    }

    // Check for :key on <template> elements
    const templateKeyRuleName = 'vue/no-template-key';
    if (isRuleEnabled(templateKeyRuleName)) {
      const templateKeySeverity = getSeverity(templateKeyRuleName, 'error');
      if (templateKeySeverity !== 'off') {
        const templateKeyRegex = /<template[^>]*:key="[^"]*"/g;
        let templateKey;
        while ((templateKey = templateKeyRegex.exec(source)) !== null) {
          const startPos = getPositionFromOffset(source, templateKey.index);
          const endOffset = templateKey.index + templateKey[0].length;
          const endPos = getPositionFromOffset(source, endOffset);
          allDiagnostics.push({
            rule: templateKeyRuleName,
            severity: templateKeySeverity,
            message: '`<template>` cannot have a `:key` attribute',
            location: {
              start: { line: startPos.line, column: startPos.column, offset: templateKey.index },
              end: { line: endPos.line, column: endPos.column, offset: endOffset },
            },
            help: 'Move the `:key` attribute to a real element inside the template',
          });
        }
      }
    }

    return {
      filename,
      errorCount: allDiagnostics.filter(d => d.severity === 'error').length,
      warningCount: allDiagnostics.filter(d => d.severity === 'warning').length,
      diagnostics: allDiagnostics,
    };
  };

  const mockLintSfc = (source: string, options: LintOptions): LintResult => {
    // Extract template from SFC and lint it
    const templateMatch = source.match(/<template>([\s\S]*?)<\/template>/);
    if (templateMatch) {
      return mockLintTemplate(templateMatch[1], options);
    }
    return {
      filename: options.filename || 'anonymous.vue',
      errorCount: 0,
      warningCount: 0,
      diagnostics: [],
    };
  };

  const mockGetLintRules = (): LintRule[] => {
    return [
      // Essential rules - Error prevention
      {
        name: 'vue/require-v-for-key',
        description: 'Require v-bind:key with v-for directives. Keys help Vue identify which items have changed, been added, or removed for efficient DOM updates.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/valid-v-for',
        description: 'Enforce valid v-for directives. The v-for directive requires a specific syntax: "item in items" or "(item, index) in items".',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/valid-v-if',
        description: 'Enforce valid v-if directives. The directive must have a value that evaluates to a boolean.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/valid-v-else',
        description: 'Enforce valid v-else directives. The v-else directive must be preceded by v-if or v-else-if on a sibling element.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/valid-v-else-if',
        description: 'Enforce valid v-else-if directives. Must be preceded by v-if or v-else-if and must have a value.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/valid-v-model',
        description: 'Enforce valid v-model directives. The v-model directive requires a value that is a valid left-hand side expression.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/valid-v-on',
        description: 'Enforce valid v-on directives. Event handlers must have valid syntax and reference existing methods or expressions.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/valid-v-bind',
        description: 'Enforce valid v-bind directives. Dynamic bindings must have valid JavaScript expressions as values.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/valid-v-slot',
        description: 'Enforce valid v-slot directives. Slot directives must be used on template elements or component elements.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/no-dupe-keys',
        description: 'Disallow duplicate keys in v-for. Duplicate keys can cause rendering bugs and performance issues.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/no-dupe-v-else-if',
        description: 'Disallow duplicate conditions in v-if / v-else-if chains. Duplicate conditions are likely a copy-paste error.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/no-duplicate-attributes',
        description: 'Disallow duplicate attributes on elements. Duplicate attributes override each other unexpectedly.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/no-parsing-error',
        description: 'Disallow parsing errors in <template>. Template syntax errors prevent the component from rendering.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/no-reserved-keys',
        description: 'Disallow overwriting reserved keys like $data, $props, $el, $options, $parent, $root, $children, $slots, $refs, $attrs, $listeners, $watch.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/no-template-key',
        description: 'Disallow key attribute on <template>. The key attribute should be placed on real elements inside the template.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/no-textarea-mustache',
        description: 'Disallow mustache interpolations inside <textarea>. Use v-model instead for two-way binding.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/require-component-is',
        description: 'Require v-bind:is on <component> elements. Dynamic components need the is attribute to specify which component to render.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/require-valid-default-prop',
        description: 'Enforce props with default values to have valid types. Object and Array defaults must be returned from a factory function.',
        category: 'Essential',
        fixable: false,
        defaultSeverity: 'error',
      },

      // Strongly Recommended rules - Improving readability
      {
        name: 'vue/no-use-v-if-with-v-for',
        description: 'Disallow using v-if on the same element as v-for. When used together, v-for has higher priority, causing v-if to run on each iteration. Use computed properties to filter instead.',
        category: 'Strongly Recommended',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/no-unused-vars',
        description: 'Disallow unused variable definitions in v-for directives. Unused iteration variables indicate dead code or incomplete implementation.',
        category: 'Strongly Recommended',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/no-unused-components',
        description: 'Disallow registering components that are not used in the template. Unused component registrations increase bundle size.',
        category: 'Strongly Recommended',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/attribute-hyphenation',
        description: 'Enforce attribute naming style: always use hyphenated names (kebab-case) in templates for props.',
        category: 'Strongly Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/component-definition-name-casing',
        description: 'Enforce specific casing for component definition names. Consistent naming improves searchability.',
        category: 'Strongly Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/html-closing-bracket-newline',
        description: 'Require or disallow a line break before tag\'s closing bracket. Improves readability for multi-line elements.',
        category: 'Strongly Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/html-closing-bracket-spacing',
        description: 'Enforce consistent spacing before tag\'s closing bracket. No space for normal tags, one space for self-closing.',
        category: 'Strongly Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/html-indent',
        description: 'Enforce consistent indentation in <template>. Default is 2 spaces.',
        category: 'Strongly Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/html-quotes',
        description: 'Enforce quotes style of HTML attributes. Default is double quotes.',
        category: 'Strongly Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/html-self-closing',
        description: 'Enforce self-closing style for elements without content.',
        category: 'Strongly Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/max-attributes-per-line',
        description: 'Enforce the maximum number of attributes per line. Improves readability of complex elements.',
        category: 'Strongly Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/multiline-html-element-content-newline',
        description: 'Require a line break before and after the contents of a multiline element.',
        category: 'Strongly Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/mustache-interpolation-spacing',
        description: 'Enforce unified spacing in mustache interpolations. Default is one space: {{ value }}.',
        category: 'Strongly Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/no-multi-spaces',
        description: 'Disallow multiple spaces between attributes. Use single spaces for consistency.',
        category: 'Strongly Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/no-spaces-around-equal-signs-in-attribute',
        description: 'Disallow spaces around equal signs in attributes. Write attr="value" not attr = "value".',
        category: 'Strongly Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/prop-name-casing',
        description: 'Enforce camelCase for prop names in JavaScript and kebab-case in templates.',
        category: 'Strongly Recommended',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/require-default-prop',
        description: 'Require default value for non-required props. Makes component behavior predictable.',
        category: 'Strongly Recommended',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/require-prop-types',
        description: 'Require type definitions in props. Type definitions improve documentation and runtime validation.',
        category: 'Strongly Recommended',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/singleline-html-element-content-newline',
        description: 'Require a line break before and after single-line element contents when the element has attributes.',
        category: 'Strongly Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/v-bind-style',
        description: 'Enforce v-bind directive style: shorthand (:attr) or longhand (v-bind:attr).',
        category: 'Strongly Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/v-on-style',
        description: 'Enforce v-on directive style: shorthand (@event) or longhand (v-on:event).',
        category: 'Strongly Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/v-slot-style',
        description: 'Enforce v-slot directive style: shorthand (#slot) or longhand (v-slot:slot).',
        category: 'Strongly Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },

      // Recommended rules - Minimizing arbitrary choices
      {
        name: 'vue/attributes-order',
        description: 'Enforce order of attributes: DEFINITION, LIST_RENDERING, CONDITIONALS, RENDER_MODIFIERS, GLOBAL, UNIQUE, TWO_WAY_BINDING, OTHER_DIRECTIVES, OTHER_ATTR, EVENTS, CONTENT.',
        category: 'Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/component-tags-order',
        description: 'Enforce order of component top-level elements: <script>, <template>, <style> or <template>, <script>, <style>.',
        category: 'Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/no-lone-template',
        description: 'Disallow unnecessary <template> elements. <template> without directives adds no value.',
        category: 'Recommended',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/no-v-html',
        description: 'Disallow use of v-html to prevent XSS attacks. Use alternatives like sanitization libraries.',
        category: 'Recommended',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/order-in-components',
        description: 'Enforce order of properties in components: name, components, props, data, computed, watch, lifecycle hooks, methods.',
        category: 'Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/this-in-template',
        description: 'Disallow usage of "this" in template. Template expressions have implicit access to component instance.',
        category: 'Recommended',
        fixable: true,
        defaultSeverity: 'warning',
      },

      // Vue 3 specific rules
      {
        name: 'vue/no-deprecated-v-on-native-modifier',
        description: 'Disallow using deprecated .native modifier (Vue 3). Use emits option instead.',
        category: 'Essential (Vue 3)',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/no-deprecated-slot-attribute',
        description: 'Disallow deprecated slot attribute (Vue 3). Use v-slot directive instead.',
        category: 'Essential (Vue 3)',
        fixable: true,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/no-deprecated-slot-scope-attribute',
        description: 'Disallow deprecated slot-scope attribute (Vue 3). Use v-slot directive instead.',
        category: 'Essential (Vue 3)',
        fixable: true,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/require-slots-as-functions',
        description: 'Enforce using $slots as functions in Vue 3. $slots.default() instead of $slots.default.',
        category: 'Essential (Vue 3)',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/no-deprecated-v-bind-sync',
        description: 'Disallow deprecated .sync modifier (Vue 3). Use v-model with argument instead.',
        category: 'Essential (Vue 3)',
        fixable: true,
        defaultSeverity: 'error',
      },

      // Script setup specific rules
      {
        name: 'vue/define-macros-order',
        description: 'Enforce order of defineEmits and defineProps compiler macros in <script setup>.',
        category: 'Script Setup',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/define-props-declaration',
        description: 'Enforce declaration style of defineProps: runtime or type-based.',
        category: 'Script Setup',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/define-emits-declaration',
        description: 'Enforce declaration style of defineEmits: runtime or type-based.',
        category: 'Script Setup',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'vue/no-ref-as-operand',
        description: 'Disallow use of ref objects as operands. Use .value to access reactive value.',
        category: 'Script Setup',
        fixable: true,
        defaultSeverity: 'error',
      },
      {
        name: 'vue/no-setup-props-destructure',
        description: 'Disallow destructuring props in setup (loses reactivity). Use toRefs() instead.',
        category: 'Script Setup',
        fixable: false,
        defaultSeverity: 'warning',
      },

      // Vapor mode rules
      {
        name: 'vapor/no-vue-lifecycle-events',
        description: 'Disallow @vue:xxx per-element lifecycle events in Vapor mode. Vapor components should use lifecycle hooks (onMounted, onUnmounted, etc.) instead.',
        category: 'Vapor',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vapor/no-suspense',
        description: 'Warn about <Suspense> usage in Vapor-only applications. Suspense only works when Vapor components render inside VDOM Suspense.',
        category: 'Vapor',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'vapor/prefer-static-class',
        description: 'Prefer static class over dynamic class binding for better performance in Vapor mode. Use :class only when necessary.',
        category: 'Vapor',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vapor/no-inline-template',
        description: 'Disallow inline-template attribute in Vapor mode. Use slot or component composition instead.',
        category: 'Vapor',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vapor/require-vapor-attribute',
        description: 'Suggest adding the vapor attribute to <script setup> blocks for Vapor mode compilation.',
        category: 'Vapor',
        fixable: true,
        defaultSeverity: 'warning',
      },
      {
        name: 'vapor/no-options-api',
        description: 'Disallow Options API patterns in Vapor components. Vapor only supports Composition API.',
        category: 'Vapor',
        fixable: false,
        defaultSeverity: 'error',
      },
      {
        name: 'vapor/no-get-current-instance',
        description: 'Disallow getCurrentInstance() calls in Vapor components. getCurrentInstance() returns null in Vapor mode.',
        category: 'Vapor',
        fixable: false,
        defaultSeverity: 'error',
      },

      // Accessibility rules
      {
        name: 'a11y/img-alt',
        description: 'Require alt attribute on images. Alt text is essential for screen reader users.',
        category: 'Accessibility',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'a11y/anchor-has-content',
        description: 'Require anchor elements to have accessible content. Empty links are not accessible.',
        category: 'Accessibility',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'a11y/heading-has-content',
        description: 'Require heading elements (h1-h6) to have accessible content.',
        category: 'Accessibility',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'a11y/iframe-has-title',
        description: 'Require iframe elements to have a title attribute for screen readers.',
        category: 'Accessibility',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'a11y/no-distracting-elements',
        description: 'Disallow distracting elements like <marquee> and <blink>.',
        category: 'Accessibility',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'a11y/tabindex-no-positive',
        description: 'Disallow positive tabindex values. Positive values disrupt natural tab order.',
        category: 'Accessibility',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'a11y/click-events-have-key-events',
        description: 'Require keyboard event handlers with click events on non-interactive elements.',
        category: 'Accessibility',
        fixable: false,
        defaultSeverity: 'warning',
      },
      {
        name: 'a11y/form-control-has-label',
        description: 'Require form controls to have associated labels for accessibility.',
        category: 'Accessibility',
        fixable: false,
        defaultSeverity: 'warning',
      },
    ];
  };

  // Mock format functions
  const mockFormatSfc = (source: string, options: FormatOptions): FormatResult => {
    const tabWidth = options.tabWidth || 2;
    const useTabs = options.useTabs || false;
    const indent = useTabs ? '\t' : ' '.repeat(tabWidth);
    const semi = options.semi !== false;
    const singleQuote = options.singleQuote || false;
    const bracketSpacing = options.bracketSpacing !== false;
    const singleAttributePerLine = options.singleAttributePerLine || false;

    let formatted = source.replace(/\r\n/g, '\n');

    // Split into SFC blocks for individual formatting
    const templateMatch = formatted.match(/(<template[^>]*>)([\s\S]*?)(<\/template>)/);
    const scriptMatch = formatted.match(/(<script[^>]*>)([\s\S]*?)(<\/script>)/);
    const styleMatch = formatted.match(/(<style[^>]*>)([\s\S]*?)(<\/style>)/);

    // Format template block
    if (templateMatch) {
      const [fullMatch, openTag, content, closeTag] = templateMatch;
      const formattedTemplate = formatTemplateContent(content, indent, singleAttributePerLine);
      formatted = formatted.replace(fullMatch, `${openTag}\n${formattedTemplate}\n${closeTag}`);
    }

    // Format script block
    if (scriptMatch) {
      const [fullMatch, openTag, content, closeTag] = scriptMatch;
      const formattedScript = formatScriptContent(content, indent, semi, singleQuote, bracketSpacing);
      formatted = formatted.replace(fullMatch, `${openTag}\n${formattedScript}\n${closeTag}`);
    }

    // Format style block
    if (styleMatch) {
      const [fullMatch, openTag, content, closeTag] = styleMatch;
      const formattedStyle = formatStyleContent(content, indent);
      formatted = formatted.replace(fullMatch, `${openTag}\n${formattedStyle}\n${closeTag}`);
    }

    // Ensure consistent newlines between blocks
    formatted = formatted.replace(/(<\/(?:template|script|style)>)\n*(<(?:template|script|style))/g, '$1\n\n$2');

    return {
      code: formatted.trim() + '\n',
      changed: formatted.trim() + '\n' !== source,
    };
  };

  // Helper to format template content
  function formatTemplateContent(content: string, indent: string, singleAttributePerLine: boolean): string {
    const lines = content.split('\n');
    const result: string[] = [];
    let depth = 0;

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) continue;

      // Decrease indent for closing tags
      if (trimmed.startsWith('</') || trimmed.startsWith('/>')) {
        depth = Math.max(0, depth - 1);
      }

      // Handle self-closing decrements
      if (trimmed.match(/^<[^/][^>]*\/>$/)) {
        result.push(indent.repeat(depth) + trimmed);
        continue;
      }

      // Handle multiple attributes per line when singleAttributePerLine is true
      if (singleAttributePerLine && trimmed.match(/<\w+\s+[^>]*>/)) {
        const tagMatch = trimmed.match(/^(<\w+)(\s+[^>]+)(\/?>)$/);
        if (tagMatch) {
          const [, tagStart, attrs, tagEnd] = tagMatch;
          const attrList = attrs.trim().split(/\s+(?=[@:a-zA-Z])/);
          if (attrList.length > 1) {
            result.push(indent.repeat(depth) + tagStart);
            for (const attr of attrList) {
              result.push(indent.repeat(depth + 1) + attr.trim());
            }
            result.push(indent.repeat(depth) + tagEnd);
            if (!trimmed.endsWith('/>') && !trimmed.startsWith('</')) {
              depth++;
            }
            continue;
          }
        }
      }

      result.push(indent.repeat(depth) + trimmed);

      // Increase indent after opening tags (not self-closing)
      if (trimmed.match(/^<[^/][^>]*[^/]>$/) && !trimmed.startsWith('<!')) {
        const tagMatch = trimmed.match(/<(\w+)/);
        if (tagMatch) {
          const tag = tagMatch[1].toLowerCase();
          const voidTags = ['br', 'hr', 'img', 'input', 'meta', 'link', 'area', 'base', 'col', 'embed'];
          if (!voidTags.includes(tag)) {
            depth++;
          }
        }
      }
    }

    return result.join('\n');
  }

  // Helper to format script content
  function formatScriptContent(content: string, indent: string, semi: boolean, singleQuote: boolean, bracketSpacing: boolean): string {
    let formatted = content;

    // Normalize indentation
    const lines = formatted.split('\n');
    const result: string[] = [];
    let depth = 0;

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) {
        result.push('');
        continue;
      }

      // Track braces for indentation
      const openBraces = (trimmed.match(/[{([\[]/g) || []).length;
      const closeBraces = (trimmed.match(/[})\]]/g) || []).length;

      // Decrease indent for closing braces at start
      if (trimmed.match(/^[}\])]/) || trimmed.match(/^<\/|^\)/)) {
        depth = Math.max(0, depth - 1);
      }

      result.push(indent.repeat(depth) + trimmed);

      // Adjust depth based on braces
      depth += openBraces - closeBraces;
      depth = Math.max(0, depth);
    }

    formatted = result.join('\n');

    // Handle semicolons
    if (!semi) {
      formatted = formatted.replace(/;(\s*\n)/g, '$1');
    }

    // Handle quotes
    if (singleQuote) {
      formatted = formatted.replace(/"([^"\\]*(?:\\.[^"\\]*)*)"/g, "'$1'");
    }

    // Handle bracket spacing in objects
    if (bracketSpacing) {
      formatted = formatted.replace(/\{(\S)/g, '{ $1');
      formatted = formatted.replace(/(\S)\}/g, '$1 }');
    } else {
      formatted = formatted.replace(/\{\s+/g, '{');
      formatted = formatted.replace(/\s+\}/g, '}');
    }

    return formatted;
  }

  // Helper to format style content
  function formatStyleContent(content: string, indent: string): string {
    const lines = content.split('\n');
    const result: string[] = [];
    let depth = 0;

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) continue;

      // Decrease indent for closing braces
      if (trimmed.startsWith('}')) {
        depth = Math.max(0, depth - 1);
      }

      result.push(indent.repeat(depth) + trimmed);

      // Increase indent after opening braces
      if (trimmed.endsWith('{')) {
        depth++;
      }
    }

    return result.join('\n');
  }

  const mockFormatTemplate = (source: string, options: FormatOptions): FormatResult => {
    const tabWidth = options.tabWidth || 2;
    const useTabs = options.useTabs || false;
    const indent = useTabs ? '\t' : ' '.repeat(tabWidth);

    // Basic template formatting
    let formatted = source.replace(/\r\n/g, '\n');

    // Normalize indentation
    const lines = formatted.split('\n');
    const formattedLines: string[] = [];
    let depth = 0;

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) {
        formattedLines.push('');
        continue;
      }

      // Decrease indent for closing tags
      if (trimmed.startsWith('</')) {
        depth = Math.max(0, depth - 1);
      }

      formattedLines.push(indent.repeat(depth) + trimmed);

      // Increase indent after opening tags (not self-closing)
      if (trimmed.startsWith('<') && !trimmed.startsWith('</') && !trimmed.endsWith('/>') && !trimmed.startsWith('<!')) {
        const tagMatch = trimmed.match(/<(\w+)/);
        if (tagMatch) {
          const tag = tagMatch[1].toLowerCase();
          // Skip void elements
          const voidTags = ['br', 'hr', 'img', 'input', 'meta', 'link', 'area', 'base', 'col', 'embed', 'param', 'source', 'track', 'wbr'];
          if (!voidTags.includes(tag)) {
            depth++;
          }
        }
      }
    }

    formatted = formattedLines.join('\n');

    return {
      code: formatted,
      changed: formatted !== source,
    };
  };

  const mockFormatScript = (source: string, options: FormatOptions): FormatResult => {
    const tabWidth = options.tabWidth || 2;
    const useTabs = options.useTabs || false;
    const indent = useTabs ? '\t' : ' '.repeat(tabWidth);
    const semi = options.semi !== false;
    const singleQuote = options.singleQuote || false;

    let formatted = source.replace(/\r\n/g, '\n');

    // Basic formatting
    if (singleQuote) {
      // Convert double quotes to single quotes (simplified)
      formatted = formatted.replace(/"([^"\\]*(?:\\.[^"\\]*)*)"/g, "'$1'");
    }

    // Handle semicolons
    if (!semi) {
      formatted = formatted.replace(/;(\s*\n)/g, '$1');
    }

    // Normalize indentation
    formatted = formatted.replace(/\t/g, indent);

    return {
      code: formatted,
      changed: formatted !== source,
    };
  };

  const mockGetLocales = (): LocaleInfo[] => {
    return [
      { code: 'en', name: 'English' },
      { code: 'ja', name: '日本語' },
      { code: 'zh', name: '中文' },
    ];
  };

  const mockAnalyzeSfc = (source: string, _options: AnalysisOptions): AnalysisResult => {
    // Parse the SFC to extract information
    const hasScriptSetup = source.includes('<script setup');
    const hasDefineProps = source.includes('defineProps');
    const hasDefineEmits = source.includes('defineEmits');
    const hasScoped = source.includes('<style scoped');

    const bindings: BindingDisplay[] = [];
    const macros: MacroDisplay[] = [];
    const props: PropDisplay[] = [];
    const emits: EmitDisplay[] = [];

    // Extract ref bindings
    const refMatches2 = source.matchAll(/const\s+(\w+)\s*=\s*ref\(/g);
    for (const match of refMatches2) {
      bindings.push({
        name: match[1],
        kind: 'SetupRef',
        source: 'ref' as BindingSource,
        metadata: {
          isExported: false,
          isImported: false,
          isComponent: false,
          isDirective: false,
          needsValue: true,
          usedInTemplate: true,
          usedInScript: true,
          scopeDepth: 0,
        },
        start: match.index || 0,
        end: (match.index || 0) + match[0].length,
        isUsed: true,
        isMutated: true,
        referenceCount: 1,
      });
    }

    // Extract computed bindings
    const computedMatches = source.matchAll(/const\s+(\w+)\s*=\s*computed\(/g);
    for (const match of computedMatches) {
      bindings.push({
        name: match[1],
        kind: 'SetupComputed',
        source: 'computed' as BindingSource,
        metadata: {
          isExported: false,
          isImported: false,
          isComponent: false,
          isDirective: false,
          needsValue: true,
          usedInTemplate: true,
          usedInScript: true,
          scopeDepth: 0,
        },
        start: match.index || 0,
        end: (match.index || 0) + match[0].length,
        isUsed: true,
        isMutated: false,
        referenceCount: 1,
      });
    }

    // Extract function bindings
    const functionMatches = source.matchAll(/function\s+(\w+)\s*\(/g);
    for (const match of functionMatches) {
      bindings.push({
        name: match[1],
        kind: 'SetupConst',
        source: 'function' as BindingSource,
        metadata: {
          isExported: false,
          isImported: false,
          isComponent: false,
          isDirective: false,
          needsValue: false,
          usedInTemplate: true,
          usedInScript: true,
          scopeDepth: 0,
        },
        start: match.index || 0,
        end: (match.index || 0) + match[0].length,
        isUsed: true,
        isMutated: false,
        referenceCount: 1,
      });
    }

    // Extract defineProps
    if (hasDefineProps) {
      const propsMatch = source.match(/defineProps<\{([^}]+)\}>/);
      if (propsMatch) {
        macros.push({
          name: 'defineProps',
          start: propsMatch.index || 0,
          end: (propsMatch.index || 0) + propsMatch[0].length,
          type_args: propsMatch[1],
        });
        // Extract prop names from type
        const propNameMatches = propsMatch[1].matchAll(/(\w+)(\?)?:/g);
        for (const propMatch of propNameMatches) {
          props.push({
            name: propMatch[1],
            required: !propMatch[2],
            has_default: false,
          });
        }
      }
    }

    // Extract defineEmits
    if (hasDefineEmits) {
      const emitsMatch = source.match(/defineEmits<\{([^}]+)\}>/);
      if (emitsMatch) {
        macros.push({
          name: 'defineEmits',
          start: emitsMatch.index || 0,
          end: (emitsMatch.index || 0) + emitsMatch[0].length,
          type_args: emitsMatch[1],
        });
        // Extract emit names from type
        const emitNameMatches = emitsMatch[1].matchAll(/(\w+):/g);
        for (const emitMatch of emitNameMatches) {
          emits.push({
            name: emitMatch[1],
          });
        }
      }
    }

    // Generate VIR text (formal TOML-like format)
    let vir = '';
    vir += '# VIR v0.1\n';
    vir += '\n';

    // Stats section
    vir += '[stats]\n';
    vir += `bindings = ${bindings.length}\n`;
    vir += `macros = ${macros.length}\n`;

    // Macros section
    if (macros.length > 0) {
      vir += '\n[macros]\n';
      for (const macro of macros) {
        vir += `@${macro.name}`;
        if (macro.type_args) {
          vir += `<{\n${macro.type_args.trim()}\n}>`;
        }
        if (macro.identifier) {
          vir += ` -> ${macro.identifier}`;
        }
        vir += ` # ${macro.start}:${macro.end}\n`;
      }
    }

    // Bindings section
    if (bindings.length > 0) {
      vir += '\n[bindings]\n';
      for (const binding of bindings) {
        const flags: string[] = [];
        if (binding.kind === 'SetupRef') flags.push('ref');
        if (binding.metadata?.isUsed !== false) flags.push('used');
        if (binding.metadata?.isMutated) flags.push('mut');
        vir += `${binding.name}: ${binding.source} @0:0`;
        if (flags.length > 0) {
          vir += ` [${flags.join(', ')}]`;
        }
        vir += '\n';
      }
    }

    // CSS section
    const selectorCount = (source.match(/[.#\w][\w-]*\s*\{/g) || []).length;
    const vBindCount = (source.match(/v-bind\(/g) || []).length;
    if (hasScoped) {
      vir += '\n[css]\n';
      vir += `scoped = true\n`;
      vir += `selectors = ${selectorCount}\n`;
      vir += `v_bind = ${vBindCount}\n`;
    }

    // Props section
    if (props.length > 0) {
      vir += '\n[props]\n';
      for (const prop of props) {
        vir += `${prop.name}: ${prop.type || 'any'}\n`;
      }
    }

    // Emits section
    if (emits.length > 0) {
      vir += '\n[emits]\n';
      for (const emit of emits) {
        vir += `${emit.name}\n`;
      }
    }

    // Generate scopes from source analysis
    const scopes: ScopeDisplay[] = [];
    let scopeId = 0;

    // Module scope (root)
    const moduleScope: ScopeDisplay = {
      id: scopeId++,
      kind: 'module',
      kindStr: 'Module',
      start: 0,
      end: source.length,
      bindings: bindings.map(b => b.name),
      children: [],
      depth: 0,
    };
    scopes.push(moduleScope);

    // Detect setup scope if script setup exists
    if (hasScriptSetup) {
      const scriptSetupMatch = source.match(/<script[^>]*setup[^>]*>([\s\S]*?)<\/script>/);
      if (scriptSetupMatch) {
        const setupStart = source.indexOf(scriptSetupMatch[0]);
        const setupEnd = setupStart + scriptSetupMatch[0].length;
        const setupScope: ScopeDisplay = {
          id: scopeId++,
          parentId: 0,
          kind: 'setup',
          kindStr: 'Setup',
          start: setupStart,
          end: setupEnd,
          bindings: bindings.map(b => b.name),
          children: [],
          depth: 1,
        };
        moduleScope.children.push(setupScope.id);
        scopes.push(setupScope);

        // Detect function scopes inside setup
        const functionRegex = /function\s+(\w+)\s*\([^)]*\)\s*\{/g;
        let funcMatch;
        while ((funcMatch = functionRegex.exec(scriptSetupMatch[1])) !== null) {
          const funcScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'function',
            kindStr: `Function (${funcMatch[1]})`,
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 50,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(funcScope.id);
          scopes.push(funcScope);
        }

        // Detect arrow function scopes
        const arrowRegex = /const\s+(\w+)\s*=\s*\([^)]*\)\s*=>/g;
        while ((funcMatch = arrowRegex.exec(scriptSetupMatch[1])) !== null) {
          const arrowScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: `Arrow (${funcMatch[1]})`,
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 50,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(arrowScope.id);
          scopes.push(arrowScope);
        }

        // Detect watch callbacks
        const watchRegex = /watch\s*\([^,]+,\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = watchRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const watchScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'watch',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 30,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(watchScope.id);
          scopes.push(watchScope);
        }

        // Detect watchEffect callbacks
        const watchEffectRegex = /watchEffect\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = watchEffectRegex.exec(scriptSetupMatch[1])) !== null) {
          const watchEffectScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'watchEffect',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 30,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(watchEffectScope.id);
          scopes.push(watchEffectScope);
        }

        // Detect computed callbacks
        const computedRegex = /(?:const|let)\s+(\w+)\s*=\s*computed\s*\(\s*(?:\([^)]*\)\s*)?=>/g;
        while ((funcMatch = computedRegex.exec(scriptSetupMatch[1])) !== null) {
          const computedScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: `computed (${funcMatch[1]})`,
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 30,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(computedScope.id);
          scopes.push(computedScope);
        }

        // Detect computed with getter/setter
        const computedGetSetRegex = /(?:const|let)\s+(\w+)\s*=\s*computed\s*\(\s*\{/g;
        while ((funcMatch = computedGetSetRegex.exec(scriptSetupMatch[1])) !== null) {
          const computedScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'function',
            kindStr: `computed (${funcMatch[1]}) [get/set]`,
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 50,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(computedScope.id);
          scopes.push(computedScope);
        }

        // Detect lifecycle hooks
        const lifecycleHooks = ['onMounted', 'onUnmounted', 'onBeforeMount', 'onBeforeUnmount', 'onUpdated', 'onBeforeUpdate', 'onActivated', 'onDeactivated', 'onErrorCaptured', 'onRenderTracked', 'onRenderTriggered', 'onServerPrefetch'];
        for (const hook of lifecycleHooks) {
          const hookRegex = new RegExp(`${hook}\\s*\\(\\s*(?:async\\s*)?\\(?([^)]*)\\)?\\s*=>`, 'g');
          while ((funcMatch = hookRegex.exec(scriptSetupMatch[1])) !== null) {
            const hookScope: ScopeDisplay = {
              id: scopeId++,
              parentId: setupScope.id,
              kind: 'arrowFunction',
              kindStr: hook,
              start: setupStart + funcMatch.index,
              end: setupStart + funcMatch.index + funcMatch[0].length + 30,
              bindings: [],
              children: [],
              depth: 2,
            };
            setupScope.children.push(hookScope.id);
            scopes.push(hookScope);
          }
        }

        // Detect provide with factory function
        const provideRegex = /provide\s*\(\s*['"][^'"]+['"]\s*,\s*\(\)\s*=>/g;
        while ((funcMatch = provideRegex.exec(scriptSetupMatch[1])) !== null) {
          const provideScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'provide factory',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(provideScope.id);
          scopes.push(provideScope);
        }

        // Detect inject with default factory
        const injectRegex = /inject\s*\(\s*['"][^'"]+['"]\s*,\s*\(\)\s*=>/g;
        while ((funcMatch = injectRegex.exec(scriptSetupMatch[1])) !== null) {
          const injectScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'inject default',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(injectScope.id);
          scopes.push(injectScope);
        }

        // Detect try-catch blocks
        const tryCatchRegex = /try\s*\{/g;
        while ((funcMatch = tryCatchRegex.exec(scriptSetupMatch[1])) !== null) {
          const tryScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'block',
            kindStr: 'try',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 30,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(tryScope.id);
          scopes.push(tryScope);
        }

        const catchRegex = /catch\s*\(\s*(\w+)\s*\)\s*\{/g;
        while ((funcMatch = catchRegex.exec(scriptSetupMatch[1])) !== null) {
          const catchScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'block',
            kindStr: 'catch',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 30,
            bindings: [funcMatch[1]],
            children: [],
            depth: 2,
          };
          setupScope.children.push(catchScope.id);
          scopes.push(catchScope);
        }

        // Detect for loops
        const forLoopRegex = /for\s*\(\s*(?:const|let|var)\s+(\w+)/g;
        while ((funcMatch = forLoopRegex.exec(scriptSetupMatch[1])) !== null) {
          const forScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'block',
            kindStr: 'for',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 30,
            bindings: [funcMatch[1]],
            children: [],
            depth: 2,
          };
          setupScope.children.push(forScope.id);
          scopes.push(forScope);
        }

        // Detect for...of / for...in loops
        const forOfInRegex = /for\s*\(\s*(?:const|let|var)\s+(\w+)\s+(?:of|in)\s+/g;
        while ((funcMatch = forOfInRegex.exec(scriptSetupMatch[1])) !== null) {
          const forOfScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'block',
            kindStr: 'for..of/in',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 30,
            bindings: [funcMatch[1]],
            children: [],
            depth: 2,
          };
          setupScope.children.push(forOfScope.id);
          scopes.push(forOfScope);
        }

        // Detect if blocks with block-scoped variables
        const ifLetRegex = /if\s*\([^)]+\)\s*\{[^}]*(?:const|let)\s+(\w+)/g;
        while ((funcMatch = ifLetRegex.exec(scriptSetupMatch[1])) !== null) {
          const ifScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'block',
            kindStr: 'if',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: [funcMatch[1]],
            children: [],
            depth: 2,
          };
          setupScope.children.push(ifScope.id);
          scopes.push(ifScope);
        }

        // Detect Array.forEach callbacks
        const forEachRegex = /\.forEach\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = forEachRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const forEachScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'forEach',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(forEachScope.id);
          scopes.push(forEachScope);
        }

        // Detect Array.map callbacks
        const mapRegex = /\.map\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = mapRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const mapScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'map',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(mapScope.id);
          scopes.push(mapScope);
        }

        // Detect Array.filter callbacks
        const filterRegex = /\.filter\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = filterRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const filterScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'filter',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(filterScope.id);
          scopes.push(filterScope);
        }

        // Detect Array.reduce callbacks
        const reduceRegex = /\.reduce\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = reduceRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const reduceScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'reduce',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(reduceScope.id);
          scopes.push(reduceScope);
        }

        // Detect Array.find/findIndex callbacks
        const findRegex = /\.find(?:Index)?\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = findRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const findScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'find',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(findScope.id);
          scopes.push(findScope);
        }

        // Detect Array.some/every callbacks
        const someEveryRegex = /\.(?:some|every)\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = someEveryRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const someEveryScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'some/every',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(someEveryScope.id);
          scopes.push(someEveryScope);
        }

        // Detect Promise.then callbacks
        const thenRegex = /\.then\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = thenRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const thenScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: '.then',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(thenScope.id);
          scopes.push(thenScope);
        }

        // Detect Promise.catch callbacks
        const promiseCatchRegex = /\.catch\s*\(\s*\(?([^)]*)\)?\s*=>/g;
        while ((funcMatch = promiseCatchRegex.exec(scriptSetupMatch[1])) !== null) {
          const params = funcMatch[1]?.split(',').map(p => p.trim()).filter(Boolean) || [];
          const promiseCatchScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: '.catch',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: params,
            children: [],
            depth: 2,
          };
          setupScope.children.push(promiseCatchScope.id);
          scopes.push(promiseCatchScope);
        }

        // Detect Promise.finally callbacks
        const finallyRegex = /\.finally\s*\(\s*\(\)\s*=>/g;
        while ((funcMatch = finallyRegex.exec(scriptSetupMatch[1])) !== null) {
          const finallyScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: '.finally',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(finallyScope.id);
          scopes.push(finallyScope);
        }

        // Detect setTimeout/setInterval callbacks
        const timerRegex = /set(?:Timeout|Interval)\s*\(\s*\(\)\s*=>/g;
        while ((funcMatch = timerRegex.exec(scriptSetupMatch[1])) !== null) {
          const timerScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'timer',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(timerScope.id);
          scopes.push(timerScope);
        }

        // Detect nextTick callbacks
        const nextTickRegex = /nextTick\s*\(\s*\(\)\s*=>/g;
        while ((funcMatch = nextTickRegex.exec(scriptSetupMatch[1])) !== null) {
          const nextTickScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'nextTick',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 20,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(nextTickScope.id);
          scopes.push(nextTickScope);
        }

        // Detect async IIFE
        const asyncIifeRegex = /\(\s*async\s*\(\)\s*=>\s*\{/g;
        while ((funcMatch = asyncIifeRegex.exec(scriptSetupMatch[1])) !== null) {
          const asyncIifeScope: ScopeDisplay = {
            id: scopeId++,
            parentId: setupScope.id,
            kind: 'arrowFunction',
            kindStr: 'async IIFE',
            start: setupStart + funcMatch.index,
            end: setupStart + funcMatch.index + funcMatch[0].length + 30,
            bindings: [],
            children: [],
            depth: 2,
          };
          setupScope.children.push(asyncIifeScope.id);
          scopes.push(asyncIifeScope);
        }
      }
    }

    // Detect v-for scopes in template
    const templateMatch = source.match(/<template>([\s\S]*?)<\/template>/);
    if (templateMatch) {
      const templateStart = source.indexOf(templateMatch[0]);
      const vForRegex = /v-for="([^"]+)"/g;
      let vForMatch;
      while ((vForMatch = vForRegex.exec(templateMatch[1])) !== null) {
        const expr = vForMatch[1];
        const inMatch = expr.match(/\(?([^)]+)\)?\s+(?:in|of)\s+/);
        const vForBindings: string[] = [];
        if (inMatch) {
          const aliases = inMatch[1].split(',').map(s => s.trim());
          vForBindings.push(...aliases);
        }

        const vForScope: ScopeDisplay = {
          id: scopeId++,
          parentId: 0,
          kind: 'vFor',
          kindStr: `v-for`,
          start: templateStart + vForMatch.index,
          end: templateStart + vForMatch.index + vForMatch[0].length,
          bindings: vForBindings,
          children: [],
          depth: 1,
        };
        moduleScope.children.push(vForScope.id);
        scopes.push(vForScope);
      }

      // Detect v-slot scopes
      const vSlotRegex = /v-slot(?::(\w+))?="([^"]+)"/g;
      let vSlotMatch;
      while ((vSlotMatch = vSlotRegex.exec(templateMatch[1])) !== null) {
        const slotName = vSlotMatch[1] || 'default';
        const slotParams = vSlotMatch[2]?.match(/\{?\s*([^}]+)\s*\}?/)?.[1]?.split(',').map(s => s.trim()) || [];
        const vSlotScope: ScopeDisplay = {
          id: scopeId++,
          parentId: 0,
          kind: 'vSlot',
          kindStr: `v-slot:${slotName}`,
          start: templateStart + vSlotMatch.index,
          end: templateStart + vSlotMatch.index + vSlotMatch[0].length,
          bindings: slotParams,
          children: [],
          depth: 1,
        };
        moduleScope.children.push(vSlotScope.id);
        scopes.push(vSlotScope);
      }

      // Detect inline event handler scopes
      const eventRegex = /@(\w+)="([^"]+)"/g;
      let eventMatch;
      while ((eventMatch = eventRegex.exec(templateMatch[1])) !== null) {
        const handler = eventMatch[2];
        if (handler.includes('=>') || handler.includes('$event')) {
          const eventScope: ScopeDisplay = {
            id: scopeId++,
            parentId: 0,
            kind: 'arrowFunction',
            kindStr: `@${eventMatch[1]} handler`,
            start: templateStart + eventMatch.index,
            end: templateStart + eventMatch.index + eventMatch[0].length,
            bindings: ['$event'],
            children: [],
            depth: 1,
          };
          moduleScope.children.push(eventScope.id);
          scopes.push(eventScope);
        }
      }
    }

    // Add scopes to VIR
    if (scopes.length > 0) {
      vir += '\n[scopes]\n';
      for (const scope of scopes) {
        vir += `#${scope.id} ${scope.kindStr.toLowerCase()} @${scope.start}:${scope.end}`;
        if (scope.bindings.length > 0) {
          vir += ` {${scope.bindings.join(', ')}}`;
        }
        vir += '\n';
      }
    }

    // Update stats with scope count
    vir = vir.replace('[stats]\n', `[stats]\nscopes = ${scopes.length}\n`);

    const summary: AnalysisSummary = {
      is_setup: hasScriptSetup,
      bindings,
      scopes,
      macros,
      props,
      emits,
      css: hasScoped ? {
        selector_count: (source.match(/[.#\w][\w-]*\s*\{/g) || []).length,
        unused_selectors: [],
        v_bind_count: (source.match(/v-bind\(/g) || []).length,
        is_scoped: true,
      } : undefined,
      diagnostics: [],
      stats: {
        binding_count: bindings.length,
        unused_binding_count: 0,
        scope_count: scopes.length,
        macro_count: macros.length,
        error_count: 0,
        warning_count: 0,
      },
    };

    return {
      summary,
      diagnostics: [],
      vir,
    };
  };

  return {
    compile: mockCompile,
    compileVapor: (template: string, options: CompilerOptions) =>
      mockCompile(template, { ...options, outputMode: 'vapor' }),
    compileCss: mockCompileCss,
    parseTemplate: (template: string) => buildMockAst(template),
    parseSfc: mockParseSfc,
    compileSfc: mockCompileSfc,
    analyzeSfc: mockAnalyzeSfc,
    parseArt: mockParseArt,
    artToCsf: mockArtToCsf,
    lintTemplate: mockLintTemplate,
    lintSfc: mockLintSfc,
    getLintRules: mockGetLintRules,
    getLocales: mockGetLocales,
    formatSfc: mockFormatSfc,
    formatTemplate: mockFormatTemplate,
    formatScript: mockFormatScript,
    Compiler: MockCompiler,
  };
}

function buildMockAst(template: string): object {
  const children: object[] = [];

  // Simple regex-based parsing for mock AST
  const elementMatch = template.match(/<(\w+)([^>]*)>([\s\S]*?)<\/\1>/);
  if (elementMatch) {
    const [, tag, attrs, content] = elementMatch;
    const props: object[] = [];

    // Parse attributes
    const vIfMatch = attrs.match(/v-if="([^"]+)"/);
    if (vIfMatch) {
      props.push({ type: 'DIRECTIVE', name: 'if', exp: vIfMatch[1] });
    }

    const vForMatch = attrs.match(/v-for="([^"]+)"/);
    if (vForMatch) {
      props.push({ type: 'DIRECTIVE', name: 'for', exp: vForMatch[1] });
    }

    const vOnMatch = attrs.match(/@(\w+)="([^"]+)"/);
    if (vOnMatch) {
      props.push({ type: 'DIRECTIVE', name: 'on', arg: vOnMatch[1], exp: vOnMatch[2] });
    }

    const vBindMatch = attrs.match(/:(\w+)="([^"]+)"/);
    if (vBindMatch) {
      props.push({ type: 'DIRECTIVE', name: 'bind', arg: vBindMatch[1], exp: vBindMatch[2] });
    }

    const classMatch = attrs.match(/class="([^"]+)"/);
    if (classMatch) {
      props.push({ type: 'ATTRIBUTE', name: 'class', value: classMatch[1] });
    }

    // Parse children
    const childElements: object[] = [];
    const interpMatch = content.match(/\{\{\s*([^}]+)\s*\}\}/g);
    if (interpMatch) {
      interpMatch.forEach((interp) => {
        const exp = interp.match(/\{\{\s*([^}]+)\s*\}\}/)?.[1] || '';
        childElements.push({
          type: 'INTERPOLATION',
          content: { type: 'SIMPLE_EXPRESSION', content: exp.trim() },
        });
      });
    }

    children.push({
      type: 'ELEMENT',
      tag,
      tagType: 'ELEMENT',
      props,
      children: childElements,
      isSelfClosing: false,
    });
  }

  return {
    type: 'ROOT',
    children,
    helpers: [],
    components: [],
    directives: [],
    hoists: [],
    imports: [],
    cached: [],
  };
}

export function isWasmLoaded(): boolean {
  return wasmModule !== null && !usingMock;
}

export function isUsingMock(): boolean {
  return usingMock;
}
