/**
 * Vize configuration utilities
 */

import fs from "node:fs";
import path from "node:path";

/**
 * Vize configuration for vite plugin
 */
export interface VizeConfig {
  /**
   * Vite plugin specific options
   */
  vite?: {
    /**
     * Patterns to include for compilation
     */
    include?: string | RegExp | (string | RegExp)[];
    /**
     * Patterns to exclude from compilation
     */
    exclude?: string | RegExp | (string | RegExp)[];
    /**
     * Glob patterns for file scanning
     */
    scanPatterns?: string[];
    /**
     * Glob patterns to ignore
     */
    ignorePatterns?: string[];
  };

  /**
   * Compiler options
   */
  compiler?: {
    /**
     * Enable SSR mode
     */
    ssr?: boolean;
    /**
     * Generate source maps
     */
    sourceMap?: boolean;
    /**
     * Enable Vapor mode
     */
    vapor?: boolean;
  };

  /**
   * Linter options
   */
  linter?: {
    /**
     * Enable linting
     */
    enabled?: boolean;
    /**
     * Rules to enable/disable
     */
    rules?: Record<string, "off" | "warn" | "error">;
  };
}

/**
 * Options for loading configuration
 */
export interface LoadConfigOptions {
  /**
   * Config loading mode
   * - 'root': Look for config in project root
   * - 'nearest': Search up directory tree
   * - 'auto': Automatically detect (same as 'nearest')
   * - 'none': Don't load config file
   */
  mode?: "root" | "nearest" | "auto" | "none";
  /**
   * Explicit config file path
   */
  configFile?: string;
}

/**
 * Define a Vize configuration with type checking
 */
export function defineConfig(config: VizeConfig): VizeConfig {
  return config;
}

const CONFIG_FILES = [
  "vize.config.ts",
  "vize.config.js",
  "vize.config.mjs",
  "vize.config.cjs",
  "vize.config.json",
];

/**
 * Load Vize configuration from file
 */
export async function loadConfig(
  root: string,
  options: LoadConfigOptions = {},
): Promise<VizeConfig | null> {
  const { mode = "root", configFile } = options;

  if (mode === "none") {
    return null;
  }

  // Treat 'auto' as 'nearest'
  const searchMode = mode === "auto" ? "nearest" : mode;

  // If explicit config file is provided
  if (configFile) {
    const configPath = path.isAbsolute(configFile) ? configFile : path.resolve(root, configFile);
    return loadConfigFile(configPath);
  }

  // Search for config file
  let searchDir = root;

  while (true) {
    for (const filename of CONFIG_FILES) {
      const configPath = path.join(searchDir, filename);
      if (fs.existsSync(configPath)) {
        return loadConfigFile(configPath);
      }
    }

    if (searchMode === "root") {
      break;
    }

    // Move to parent directory
    const parentDir = path.dirname(searchDir);
    if (parentDir === searchDir) {
      break;
    }
    searchDir = parentDir;
  }

  return null;
}

async function loadConfigFile(configPath: string): Promise<VizeConfig | null> {
  if (!fs.existsSync(configPath)) {
    return null;
  }

  const ext = path.extname(configPath);

  if (ext === ".json") {
    const content = fs.readFileSync(configPath, "utf-8");
    return JSON.parse(content);
  }

  // For JS/TS files, use dynamic import
  try {
    const module = await import(configPath);
    return module.default ?? module;
  } catch (e) {
    console.warn(`[vize] Failed to load config from ${configPath}:`, e);
    return null;
  }
}
