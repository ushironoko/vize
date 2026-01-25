import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'path';

export default defineConfig({
  plugins: [
    vue({
      template: {
        compilerOptions: {
          // Treat fresco elements as custom elements
          isCustomElement: (tag) => ['box', 'text', 'input'].includes(tag),
        },
      },
    }),
  ],
  resolve: {
    alias: {
      // Map vue to full build (includes compiler)
      vue: 'vue/dist/vue.esm-bundler.js',
      // Shim for SSR imports (script setup generates these)
      '@vue/runtime-core/server-renderer': resolve(__dirname, 'ssr-shim.ts'),
    },
  },
  build: {
    target: 'node18',
    lib: {
      entry: 'main.ts',
      formats: ['es'],
      fileName: 'main',
    },
    rollupOptions: {
      external: ['@vizejs/fresco-native', '@vizejs/fresco'],
    },
  },
});
