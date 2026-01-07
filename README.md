# vue-compiler-rs

A high-performance Rust implementation of the Vue.js compiler.

**[Playground](https://ubugeeei.github.io/vue-compiler-rs/)**

## Performance

Compiling **15,000 SFC files** (36.9 MB):

|  | @vue/compiler-sfc | vue-compiler-rs | Speedup |
|--|-------------------|-----------------|---------|
| **Single Thread** | 19.28s | 4.80s | **4.0x** |
| **Multi Thread** (10 workers) | 5.95s | 692ms | **8.6x** (27.9x vs Original 1T) |

## Compatibility

Snapshot tests against `@vue/compiler-sfc` (v3.6.0-beta):

| Category | Passed | Total | Coverage |
|----------|--------|-------|----------|
| **VDom** | 267 | 338 | 79.0% |
| **Vapor** | 29 | 98 | 29.6% |
| **SFC** | 3 | 70 | 4.3% |
| **Total** | 299 | 506 | 59.1% |

### TypeScript Output Snapshots

We maintain **70 snapshot tests** for TypeScript output mode in `tests/snapshots/sfc/ts/`. These capture the current behavior for:

- Basic script setup patterns
- defineProps/defineEmits/defineModel
- Props destructure with defaults
- Generic components (Vue 3.3+)
- Complex TypeScript types (arrow functions, unions, intersections)
- Top-level await
- withDefaults patterns
- Real-world patterns from production codebases

Run `mise run snapshot` to update snapshots after changes.

### CLI Output Modes

The CLI supports two output modes via `--script-ext`:

- `downcompile` (default): Transpiles TypeScript to JavaScript
- `preserve`: Keeps TypeScript output as-is

```bash
# Preserve TypeScript output (recommended for TypeScript projects)
vuec "src/**/*.vue" --script-ext preserve -o dist

# Downcompile to JavaScript (default)
vuec "src/**/*.vue" -o dist
```

### Known Limitations

Some Vue 3.3+ features are not yet fully supported:
- Generic component declarations (`<script setup generic="T">`)
- Complex TypeScript type extraction from interfaces
- `as const` assertions in multiline expressions

## Quick Start

```bash
mise install && mise run setup
mise run build    # Build bindings
mise run test     # Run tests
mise run cov      # Coverage report
mise run dev      # Playground
```

Run `mise tasks` to see all available commands.

## Usage

### CLI

```bash
# Build CLI
cargo build -p vue_compiler_cli --release

# Compile single file
./target/release/vue-compiler "src/**/*.vue"

# Compile with output directory
./target/release/vue-compiler "src/**/*.vue" -o dist

# Show statistics only
./target/release/vue-compiler "src/**/*.vue" -f stats

# SSR mode
./target/release/vue-compiler "src/**/*.vue" --ssr

# Control thread count
./target/release/vue-compiler "src/**/*.vue" -j 4
```

Options:
- `-o, --output <DIR>` - Output directory (stdout if not specified)
- `-f, --format <FORMAT>` - Output format: `js`, `json`, `stats` (default: js)
- `-j, --threads <N>` - Number of threads (default: CPU count)
- `--ssr` - Enable SSR mode
- `--continue-on-error` - Continue on errors
- `-v, --verbose` - Verbose output

### Node.js / Browser

```javascript
// Node.js (Native)
const { compileSfc } = require('@vue-compiler-rs/native');
const { code } = compileSfc(`<template><div>{{ msg }}</div></template>`, { filename: 'App.vue' });

// Browser (WASM)
import init, { compileSfc } from '@vue-compiler-rs/wasm';
await init();
const { code } = compileSfc(`...`, { filename: 'App.vue' });
```

## License

MIT
