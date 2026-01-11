#!/usr/bin/env node

/**
 * Musea MCP Server CLI.
 *
 * Usage:
 *   musea-mcp [project-root]
 *
 * Environment:
 *   MUSEA_PROJECT_ROOT - Project root directory (default: cwd)
 */

import { startServer } from './index.js';

const projectRoot = process.argv[2] || process.env.MUSEA_PROJECT_ROOT || process.cwd();

console.error(`[musea-mcp] Starting server for project: ${projectRoot}`);

startServer(projectRoot).catch((error) => {
  console.error('[musea-mcp] Failed to start:', error);
  process.exit(1);
});
