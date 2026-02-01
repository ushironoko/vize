import { defineConfig } from 'tsdown'

export default defineConfig({
  entry: ['src/index.ts', 'src/vrt.ts', 'src/cli.ts'],
  format: 'esm',
  dts: true,
  clean: true,
})
