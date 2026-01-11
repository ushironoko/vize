# @vizejs/core

Core JavaScript API for Vize Vue compiler.

## Installation

```bash
npm install @vizejs/core
```

## Usage

```ts
import { compile } from '@vizejs/core'

const result = compile(`
<template>
  <div>{{ msg }}</div>
</template>
<script setup>
const msg = 'Hello'
</script>
`)

console.log(result.code)
```

## API

### `compile(source, options?)`

Compile a Vue SFC source string.

```ts
interface CompileOptions {
  filename?: string
  ssr?: boolean
  vapor?: boolean
  sourceMap?: boolean
}

interface CompileResult {
  code: string
  css?: string
  map?: SourceMap
  errors: Error[]
}
```

## License

MIT
