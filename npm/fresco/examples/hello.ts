/**
 * Hello World example for Fresco
 *
 * Run with: npx tsx examples/hello.ts
 */

import { h, ref, defineComponent } from '@vue/runtime-core';
import { createApp, Box, Text } from '../src/index.js';

// Define a simple component
const App = defineComponent({
  setup() {
    const count = ref(0);

    // Increment counter every second
    setInterval(() => {
      count.value++;
    }, 1000);

    return () =>
      h(Box, { border: 'single', padding: 1 }, [
        h(Text, { bold: true, fg: 'green' }, 'Hello, Fresco!'),
        h(Text, {}, `Counter: ${count.value}`),
        h(Text, { dim: true }, 'Press Ctrl+C to exit'),
      ]);
  },
});

// Create and run the app
const app = createApp(App, {
  exitOnCtrlC: true,
});

app.mount().then(() => {
  console.log('App mounted');
});

app.waitUntilExit().then(() => {
  console.log('Goodbye!');
});
