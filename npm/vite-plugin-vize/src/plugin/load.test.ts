import assert from "node:assert/strict";

import type { VizePluginState } from "./state.js";
import { getBoundaryPlaceholderCode } from "./load.js";
import { loadHook } from "./load.js";
import { toVirtualId } from "../virtual.js";

const ssrClientPlaceholder = getBoundaryPlaceholderCode("/src/Foo.client.vue", true);
assert.ok(ssrClientPlaceholder, "SSR should stub .client.vue components");
assert.match(
  ssrClientPlaceholder,
  /createElementBlock\("div"\)/,
  "SSR .client.vue placeholder should render a simple div",
);

const clientServerPlaceholder = getBoundaryPlaceholderCode("/src/Foo.server.vue", false);
assert.ok(clientServerPlaceholder, "Client build should stub .server.vue components");
assert.match(
  clientServerPlaceholder,
  /ServerPlaceholder/,
  "Client .server.vue placeholder should use the server placeholder component",
);

assert.equal(
  getBoundaryPlaceholderCode("/src/Foo.client.vue", false),
  null,
  "Client build must not stub .client.vue components",
);
assert.equal(
  getBoundaryPlaceholderCode("/src/Foo.server.vue", true),
  null,
  "SSR build must not stub .server.vue components",
);
assert.equal(
  getBoundaryPlaceholderCode("/src/Foo.vue", true),
  null,
  "Regular SFCs must not be stubbed",
);

const realPath = "/src/Hmr.vue";
const hmrState: VizePluginState = {
  cache: new Map([
    [
      realPath,
      {
        code: `export function render() { return null }
const _sfc_main = {}
_sfc_main.render = render
export default _sfc_main`,
        scopeId: "hmr12345",
        hasScoped: false,
        styles: [],
      },
    ],
  ]),
  ssrCache: new Map(),
  collectedCss: new Map(),
  precompileMetadata: new Map(),
  pendingHmrUpdateTypes: new Map([[realPath, "template-only"]]),
  isProduction: false,
  root: "/src",
  clientViteBase: "/",
  serverViteBase: "/",
  server: {} as never,
  filter: () => true,
  scanPatterns: ["**/*.vue"],
  ignorePatterns: [],
  mergedOptions: {},
  initialized: true,
  dynamicImportAliasRules: [],
  cssAliasRules: [],
  extractCss: false,
  clientViteDefine: {},
  serverViteDefine: {},
  logger: {
    log() {},
    info() {},
    warn() {},
    error() {},
  } as never,
};

const firstLoad = loadHook(hmrState, toVirtualId(realPath), { ssr: false });
assert.ok(firstLoad && typeof firstLoad === "object", "Virtual module should load as code object");
assert.equal(firstLoad.moduleType, "js", "Virtual modules should be marked as JS for Vite 8");
assert.match(
  firstLoad.code,
  /__hmrUpdateType = "template-only"/,
  "Pending template-only HMR updates must stay granular when render is exposed",
);
assert.equal(
  hmrState.pendingHmrUpdateTypes.has(realPath),
  false,
  "Pending HMR updates should be consumed after the client load",
);

const secondLoad = loadHook(hmrState, toVirtualId(realPath), { ssr: false });
assert.ok(
  secondLoad && typeof secondLoad === "object",
  "Subsequent virtual module loads should still succeed",
);
assert.match(
  secondLoad.code,
  /__hmrUpdateType = "full-reload"/,
  "Consumed pending updates must fall back to the default HMR mode",
);

const inlinePath = "/src/InlineHmr.vue";
const inlineState: VizePluginState = {
  ...hmrState,
  cache: new Map([
    [
      inlinePath,
      {
        code: `export default {
  __name: "InlineHmr",
  setup() {
    return (_ctx, _cache) => null
  },
}`,
        scopeId: "inline1234",
        hasScoped: false,
        styles: [],
      },
    ],
  ]),
  ssrCache: new Map(),
  pendingHmrUpdateTypes: new Map([[inlinePath, "template-only"]]),
};

const inlineLoad = loadHook(inlineState, toVirtualId(inlinePath), { ssr: false });
assert.ok(
  inlineLoad && typeof inlineLoad === "object",
  "Inline-template virtual modules should load as code objects",
);
assert.equal(
  inlineLoad.moduleType,
  "js",
  "Inline-template virtual modules should be marked as JS for Vite 8",
);
assert.match(
  inlineLoad.code,
  /__hmrUpdateType = "full-reload"/,
  "Inline-template components must downgrade template-only HMR to full-reload",
);

const envPath = "/src/Environment.vue";
const environmentState: VizePluginState = {
  ...hmrState,
  cache: new Map([
    [
      envPath,
      {
        code: `export default { __name: "ClientCompiled" }`,
        scopeId: "clientenv",
        hasScoped: false,
        styles: [],
      },
    ],
  ]),
  ssrCache: new Map([
    [
      envPath,
      {
        code: `export default { __name: "ServerCompiled" }`,
        scopeId: "serverenv",
        hasScoped: false,
        styles: [],
      },
    ],
  ]),
  pendingHmrUpdateTypes: new Map(),
};

const clientEnvironmentLoad = loadHook(environmentState, toVirtualId(envPath), { ssr: false });
assert.ok(
  clientEnvironmentLoad && typeof clientEnvironmentLoad === "object",
  "Client environment loads should succeed",
);
assert.equal(
  clientEnvironmentLoad.moduleType,
  "js",
  "Client virtual modules should be marked as JS for Vite 8",
);
assert.match(
  clientEnvironmentLoad.code,
  /ClientCompiled/,
  "Client loads should read from the client compilation cache",
);

const ssrEnvironmentLoad = loadHook(environmentState, toVirtualId(envPath, true), { ssr: true });
assert.ok(
  ssrEnvironmentLoad && typeof ssrEnvironmentLoad === "object",
  "SSR environment loads should succeed",
);
assert.equal(
  ssrEnvironmentLoad.moduleType,
  "js",
  "SSR virtual modules should be marked as JS for Vite 8",
);
assert.match(
  ssrEnvironmentLoad.code,
  /ServerCompiled/,
  "SSR loads should read from the SSR compilation cache",
);

const assetUrlPath = "/src/components/common/BrandLogo.vue";
const assetUrlState: VizePluginState = {
  ...hmrState,
  root: "/src",
  cache: new Map([
    [
      assetUrlPath,
      {
        code: `export default {
  __name: "BrandLogo",
  setup() {
    const getIcon = (name, theme) => new URL(\`../../assets/logos/brand_\${name}_\${theme}.svg\`, import.meta.url).href;
    return { getIcon };
  }
}`,
        scopeId: "asset1234",
        hasScoped: false,
        styles: [],
      },
    ],
  ]),
  ssrCache: new Map(),
  pendingHmrUpdateTypes: new Map(),
};

const assetUrlLoad = loadHook(assetUrlState, toVirtualId(assetUrlPath), { ssr: false });
assert.ok(
  assetUrlLoad && typeof assetUrlLoad === "object",
  "Virtual modules with dynamic asset URLs should still load as code objects",
);
assert.equal(
  assetUrlLoad.moduleType,
  "js",
  "Dynamic asset URL virtual modules should be marked as JS for Vite 8",
);
assert.match(
  assetUrlLoad.code,
  /import\.meta\.glob\("\/assets\/logos\/brand_\*_\*\.svg", \{"eager":true,"import":"default","query":"\?url"\}\)/,
  "Dynamic new URL(import.meta.url) expressions should be rewritten to a root-absolute import.meta.glob",
);
assert.match(
  assetUrlLoad.code,
  /`\/assets\/logos\/brand_\$\{name\}_\$\{theme\}\.svg`/,
  "Rewritten dynamic asset URLs should use a root-absolute lookup key",
);
assert.doesNotMatch(
  assetUrlLoad.code,
  /new URL\(`\.\.\/\.\.\/assets\/logos/,
  "Original relative dynamic new URL(import.meta.url) expression should be rewritten away",
);

console.log("✅ vite-plugin-vize load boundary tests passed!");
