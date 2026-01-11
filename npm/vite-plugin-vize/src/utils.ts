import { createHash } from 'node:crypto';
import type { CompiledModule } from './types.js';

export function generateScopeId(filename: string): string {
  const hash = createHash('sha256').update(filename).digest('hex');
  return hash.slice(0, 8);
}

export function createFilter(
  include?: string | RegExp | (string | RegExp)[],
  exclude?: string | RegExp | (string | RegExp)[]
): (id: string) => boolean {
  const includePatterns = include
    ? Array.isArray(include)
      ? include
      : [include]
    : [/\.vue$/];
  const excludePatterns = exclude
    ? Array.isArray(exclude)
      ? exclude
      : [exclude]
    : [/node_modules/];

  return (id: string) => {
    const matchInclude = includePatterns.some((pattern) =>
      typeof pattern === 'string' ? id.includes(pattern) : pattern.test(id)
    );
    const matchExclude = excludePatterns.some((pattern) =>
      typeof pattern === 'string' ? id.includes(pattern) : pattern.test(id)
    );
    return matchInclude && !matchExclude;
  };
}

export function generateOutput(
  compiled: CompiledModule,
  isProduction: boolean,
  isDev: boolean
): string {
  let output = compiled.code;

  // Rewrite "export default" to named variable for HMR
  const hasExportDefault = output.includes('export default');
  if (hasExportDefault) {
    output = output.replace('export default', 'const _sfc_main =');
    output += '\nexport default _sfc_main;';
  }

  // Inject CSS
  if (compiled.css) {
    const cssCode = JSON.stringify(compiled.css);
    const cssId = JSON.stringify(`vize-style-${compiled.scopeId}`);
    output = `
const __vize_css__ = ${cssCode};
const __vize_css_id__ = ${cssId};
(function() {
  if (typeof document !== 'undefined') {
    let style = document.getElementById(__vize_css_id__);
    if (!style) {
      style = document.createElement('style');
      style.id = __vize_css_id__;
      style.textContent = __vize_css__;
      document.head.appendChild(style);
    } else {
      style.textContent = __vize_css__;
    }
  }
})();
${output}`;
  }

  // Add HMR support in development
  if (!isProduction && isDev && hasExportDefault) {
    output += `
if (import.meta.hot) {
  _sfc_main.__hmrId = ${JSON.stringify(compiled.scopeId)};
  import.meta.hot.accept((mod) => {
    if (!mod) return;
    const { default: updated } = mod;
    if (typeof __VUE_HMR_RUNTIME__ !== 'undefined') {
      __VUE_HMR_RUNTIME__.reload(_sfc_main.__hmrId, updated);
    }
  });
  if (typeof __VUE_HMR_RUNTIME__ !== 'undefined') {
    __VUE_HMR_RUNTIME__.createRecord(_sfc_main.__hmrId, _sfc_main);
  }
}`;
  }

  return output;
}
