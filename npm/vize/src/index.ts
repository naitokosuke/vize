/**
 * Vize - High-performance Vue.js toolchain in Rust
 *
 * This package provides:
 * - CLI binary for compilation, linting, and formatting
 * - Configuration utilities for programmatic use
 */

// Types
export type { VizeConfig, CompilerConfig, VitePluginConfig, LoadConfigOptions } from "./types.js";

// Config utilities
export { defineConfig, loadConfig } from "./config.js";
