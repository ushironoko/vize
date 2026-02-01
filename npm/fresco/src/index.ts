/**
 * @vizejs/fresco - Vue TUI Framework
 *
 * Build terminal user interfaces with Vue.js
 */

// Core
export { createApp, type App, type AppOptions, lastKeyEvent, type KeyEvent } from "./app.js";
export { createRenderer } from "./renderer.js";

// Components
export * from "./components/index.js";

// Composables
export * from "./composables/index.js";

// Re-export native bindings types
export type {
  StyleNapi,
  FlexStyleNapi,
  RenderNodeNapi,
  InputEventNapi,
  ImeStateNapi,
  TerminalInfoNapi,
} from "@vizejs/fresco-native";
