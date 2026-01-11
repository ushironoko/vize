import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { musea } from '@vizejs/vite-plugin-musea'

export default defineConfig({
  plugins: [
    vue(),
    musea({
      include: ['src/**/*.art.vue'],
      basePath: '/__musea__',
    }),
  ],
})
