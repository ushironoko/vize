/**
 * Vize configuration options
 */
export interface VizeConfig {
  /**
   * Vue compiler options
   */
  compiler?: CompilerConfig

  /**
   * Vite plugin options
   */
  vite?: VitePluginConfig
}

/**
 * Compiler configuration
 */
export interface CompilerConfig {
  /**
   * Enable Vapor mode compilation
   * @default false
   */
  vapor?: boolean

  /**
   * Enable SSR mode
   * @default false
   */
  ssr?: boolean

  /**
   * Enable source map generation
   * @default true in development, false in production
   */
  sourceMap?: boolean
}

/**
 * Vite plugin configuration
 */
export interface VitePluginConfig {
  /**
   * Files to include in compilation
   * @default /\.vue$/
   */
  include?: string | RegExp | (string | RegExp)[]

  /**
   * Files to exclude from compilation
   * @default /node_modules/
   */
  exclude?: string | RegExp | (string | RegExp)[]

  /**
   * Glob patterns to scan for .vue files during pre-compilation
   * @default ['**\/*.vue']
   */
  scanPatterns?: string[]

  /**
   * Glob patterns to ignore during pre-compilation
   * @default ['node_modules/**', 'dist/**', '.git/**']
   */
  ignorePatterns?: string[]
}

/**
 * Options for loading vize.config file
 */
export interface LoadConfigOptions {
  /**
   * Config file search mode
   * - 'root': Search only in the specified root directory
   * - 'auto': Search from cwd upward until finding a config file
   * @default 'root'
   */
  mode?: 'root' | 'auto'

  /**
   * Custom config file path (overrides automatic search)
   */
  configFile?: string
}
