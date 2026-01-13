/**
 * Vize - High-performance Vue.js toolchain in Rust
 *
 * This package provides:
 * - CLI binary for compilation, linting, and formatting
 * - Configuration utilities for programmatic use
 */

export {
  defineConfig,
  loadConfig,
  type VizeConfig,
  type LoadConfigOptions,
} from './config.js';
