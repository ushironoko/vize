/**
 * Musea plugin options.
 */
export interface MuseaOptions {
  /**
   * Glob patterns to include Art files.
   * @default ['**\/*.art.vue']
   */
  include?: string[];

  /**
   * Glob patterns to exclude.
   * @default ['node_modules/**', 'dist/**']
   */
  exclude?: string[];

  /**
   * Base path for Musea gallery UI.
   * @default '/__musea__'
   */
  basePath?: string;

  /**
   * Enable Storybook CSF output.
   * @default false
   */
  storybookCompat?: boolean;

  /**
   * Storybook output directory (when storybookCompat is true).
   * @default '.storybook/stories'
   */
  storybookOutDir?: string;

  /**
   * VRT (Visual Regression Testing) configuration.
   */
  vrt?: VrtOptions;
}

/**
 * VRT configuration options.
 */
export interface VrtOptions {
  /**
   * Snapshot storage directory.
   * @default '.vize/snapshots'
   */
  snapshotDir?: string;

  /**
   * Pixel difference threshold for comparison.
   * @default 100
   */
  threshold?: number;

  /**
   * Viewports to capture.
   * @default [{ width: 1280, height: 720 }, { width: 375, height: 667 }]
   */
  viewports?: ViewportConfig[];
}

/**
 * Viewport configuration.
 */
export interface ViewportConfig {
  /** Width in pixels */
  width: number;
  /** Height in pixels */
  height: number;
  /** Device scale factor */
  deviceScaleFactor?: number;
  /** Viewport name for identification */
  name?: string;
}

/**
 * Art file metadata.
 */
export interface ArtMetadata {
  title: string;
  description?: string;
  component?: string;
  category?: string;
  tags: string[];
  status: "draft" | "ready" | "deprecated";
  order?: number;
}

/**
 * Art variant definition.
 */
export interface ArtVariant {
  name: string;
  template: string;
  isDefault: boolean;
  skipVrt: boolean;
  args?: Record<string, unknown>;
}

/**
 * Parsed Art file information.
 */
export interface ArtFileInfo {
  /** Absolute file path */
  path: string;
  /** Art metadata */
  metadata: ArtMetadata;
  /** Variant definitions */
  variants: ArtVariant[];
  /** Whether file has script setup */
  hasScriptSetup: boolean;
  /** Whether file has regular script */
  hasScript: boolean;
  /** Number of style blocks */
  styleCount: number;
}

/**
 * CSF output from Art transformation.
 */
export interface CsfOutput {
  /** Generated CSF code */
  code: string;
  /** Suggested filename */
  filename: string;
}
