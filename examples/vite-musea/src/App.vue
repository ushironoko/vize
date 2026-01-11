<script setup lang="ts">
import Button from './components/Button.vue'
</script>

<template>
  <div class="app">
    <h1>Vite + Vue + Musea Example</h1>
    <p>
      Visit <a href="/__musea__">/__musea__</a> to see the component gallery.
    </p>
    <div class="demo">
      <Button>Default Button</Button>
      <Button variant="primary">Primary Button</Button>
      <Button variant="secondary">Secondary Button</Button>
      <Button disabled>Disabled Button</Button>
    </div>
  </div>
</template>

<style>
.app {
  max-width: 800px;
  margin: 0 auto;
  padding: 2rem;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

h1 {
  color: #333;
}

.demo {
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
  margin-top: 2rem;
}
</style>
