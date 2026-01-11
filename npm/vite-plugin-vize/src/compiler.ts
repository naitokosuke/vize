import fs from 'node:fs';
import { createRequire } from 'node:module';
import type {
  CompileSfcFn,
  CompiledModule,
  SfcCompileOptionsNapi,
} from './types.js';
import { generateScopeId } from './utils.js';

const require = createRequire(import.meta.url);

let compileSfc: CompileSfcFn | null = null;

export function loadNative(): CompileSfcFn {
  if (compileSfc) return compileSfc;

  try {
    const native = require('@vize/native');
    compileSfc = native.compileSfc;
    return compileSfc!;
  } catch (e) {
    throw new Error(
      `Failed to load @vize/native. Make sure it's installed and built:\n${e}`
    );
  }
}

export function compileFile(
  filePath: string,
  cache: Map<string, CompiledModule>,
  options: { sourceMap: boolean; ssr: boolean },
  source?: string
): CompiledModule {
  const compile = loadNative();
  const content = source ?? fs.readFileSync(filePath, 'utf-8');
  const scopeId = generateScopeId(filePath);
  const hasScoped = /<style[^>]*\bscoped\b/.test(content);

  const result = compile(content, {
    filename: filePath,
    sourceMap: options.sourceMap,
    ssr: options.ssr,
    scopeId: hasScoped ? `data-v-${scopeId}` : undefined,
  });

  if (result.errors.length > 0) {
    const errorMsg = result.errors.join('\n');
    console.error(`[vize] Compilation error in ${filePath}:\n${errorMsg}`);
  }

  if (result.warnings.length > 0) {
    result.warnings.forEach((warning) => {
      console.warn(`[vize] Warning in ${filePath}: ${warning}`);
    });
  }

  const compiled: CompiledModule = {
    code: result.code,
    css: result.css,
    scopeId,
    hasScoped,
  };

  cache.set(filePath, compiled);
  return compiled;
}
