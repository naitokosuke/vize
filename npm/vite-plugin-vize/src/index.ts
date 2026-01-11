import type { Plugin, ResolvedConfig, ViteDevServer, HmrContext } from 'vite';
import path from 'node:path';
import fs from 'node:fs';
import { glob } from 'tinyglobby';

import type { VizeOptions, CompiledModule } from './types.js';
import { compileFile } from './compiler.js';
import { createFilter, generateOutput } from './utils.js';

export type { VizeOptions, CompiledModule };

const VIRTUAL_PREFIX = '\0vize:';

export function vize(options: VizeOptions = {}): Plugin {
  const filter = createFilter(options.include, options.exclude);
  const cache = new Map<string, CompiledModule>();
  // Map from virtual ID to real file path
  const virtualToReal = new Map<string, string>();

  let isProduction: boolean;
  let root: string;
  let server: ViteDevServer | null = null;

  const scanPatterns = options.scanPatterns ?? ['**/*.vue'];
  const ignorePatterns = options.ignorePatterns ?? [
    'node_modules/**',
    'dist/**',
    '.git/**',
  ];

  async function compileAll(): Promise<void> {
    const startTime = performance.now();
    const files = await glob(scanPatterns, {
      cwd: root,
      ignore: ignorePatterns,
      absolute: true,
    });

    console.log(`[vize] Pre-compiling ${files.length} Vue files...`);

    let successCount = 0;
    let errorCount = 0;

    for (const file of files) {
      try {
        compileFile(file, cache, {
          sourceMap: options.sourceMap ?? !isProduction,
          ssr: options.ssr ?? false,
        });
        successCount++;
      } catch (e) {
        errorCount++;
        console.error(`[vize] Failed to compile ${file}:`, e);
      }
    }

    const elapsed = (performance.now() - startTime).toFixed(2);
    console.log(
      `[vize] Pre-compilation complete: ${successCount} succeeded, ${errorCount} failed (${elapsed}ms)`
    );
  }

  function resolveVuePath(id: string, importer?: string): string {
    let resolved: string;
    if (path.isAbsolute(id)) {
      resolved = id;
    } else if (importer) {
      // Remove virtual prefix from importer if present
      const realImporter = importer.startsWith(VIRTUAL_PREFIX)
        ? virtualToReal.get(importer) ?? importer.slice(VIRTUAL_PREFIX.length)
        : importer;
      resolved = path.resolve(path.dirname(realImporter), id);
    } else {
      resolved = path.resolve(root, id);
    }
    return path.normalize(resolved);
  }

  return {
    name: 'vite-plugin-vize',
    enforce: 'pre',

    configResolved(resolvedConfig: ResolvedConfig) {
      isProduction = options.isProduction ?? resolvedConfig.isProduction;
      root = options.root ?? resolvedConfig.root;
    },

    configureServer(devServer: ViteDevServer) {
      server = devServer;
    },

    async buildStart() {
      await compileAll();
    },

    resolveId(id: string, importer?: string) {
      if (id.includes('?vue&type=style')) {
        return id;
      }

      if (id.endsWith('.vue')) {
        const resolved = resolveVuePath(id, importer);

        // Return virtual module ID if cached
        if (cache.has(resolved)) {
          const virtualId = VIRTUAL_PREFIX + resolved;
          virtualToReal.set(virtualId, resolved);
          return virtualId;
        }
      }

      return null;
    },

    load(id: string) {
      if (id.includes('?vue&type=style')) {
        const [filename] = id.split('?');
        const realPath = filename.startsWith(VIRTUAL_PREFIX)
          ? virtualToReal.get(filename) ?? filename.slice(VIRTUAL_PREFIX.length)
          : filename;
        const compiled = cache.get(realPath);
        if (compiled?.css) {
          return compiled.css;
        }
        return '';
      }

      // Handle virtual module
      if (id.startsWith(VIRTUAL_PREFIX)) {
        const realPath = virtualToReal.get(id) ?? id.slice(VIRTUAL_PREFIX.length);
        const compiled = cache.get(realPath);

        if (compiled) {
          return {
            code: generateOutput(compiled, isProduction, server !== null),
            map: null,
          };
        }
      }

      return null;
    },

    async handleHotUpdate(ctx: HmrContext) {
      const { file, server, read } = ctx;

      if (file.endsWith('.vue') && filter(file)) {
        try {
          const source = await read();
          compileFile(file, cache, {
            sourceMap: options.sourceMap ?? !isProduction,
            ssr: options.ssr ?? false,
          }, source);
          console.log(`[vize] Re-compiled: ${path.relative(root, file)}`);
        } catch (e) {
          console.error(`[vize] Re-compilation failed for ${file}:`, e);
        }

        // Find the virtual module for this file
        const virtualId = VIRTUAL_PREFIX + file;
        const modules = server.moduleGraph.getModulesByFile(virtualId)
          ?? server.moduleGraph.getModulesByFile(file);
        if (modules) {
          return [...modules];
        }
      }
    },
  };
}

export default vize;
