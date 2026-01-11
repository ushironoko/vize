import { defineConfig } from 'vizejs'

export default defineConfig({
  compiler: {
    vapor: false,
    ssr: false,
    sourceMap: true,
  },
  vite: {
    scanPatterns: ['src/**/*.vue'],
    ignorePatterns: ['node_modules/**', 'dist/**'],
  },
})
