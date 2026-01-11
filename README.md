<p align="center">
  <img src="./assets/logo.svg" alt="Vize Logo" width="400" />
</p>

<h1 align="center">Vize</h1>

<p align="center">
  <strong>Unofficial High-Performance Vue.js Toolchain in Rust</strong>
</p>

<p align="center">
  <em>/viːz/ — Named after Vizier + Visor + Advisor: a wise tool that sees through your code.</em>
</p>

<p align="center">
  <a href="https://ubugeeei.github.io/vize/"><strong>Playground</strong></a>
</p>

> [!WARNING]
> This project is under active development and is not yet ready for production use.
> APIs and features may change without notice.

---

## Crates

<table>
  <tr>
    <th>Crate</th>
    <th>Description</th>
  </tr>
  <tr>
    <td><img src="./crates/vize_carton/logo.png" width="32" align="center" /> <a href="./crates/vize_carton">vize_carton</a></td>
    <td>Shared utilities & arena allocator</td>
  </tr>
  <tr>
    <td><img src="./crates/vize_relief/logo.png" width="32" align="center" /> <a href="./crates/vize_relief">vize_relief</a></td>
    <td>AST definitions, errors, options</td>
  </tr>
  <tr>
    <td><img src="./crates/vize_armature/logo.png" width="32" align="center" /> <a href="./crates/vize_armature">vize_armature</a></td>
    <td>Parser & tokenizer</td>
  </tr>
  <tr>
    <td><img src="./crates/vize_atelier_core/logo.png" width="32" align="center" /> <a href="./crates/vize_atelier_core">vize_atelier_core</a></td>
    <td>Transforms & code generation</td>
  </tr>
  <tr>
    <td><img src="./crates/vize_atelier_dom/logo.png" width="32" align="center" /> <a href="./crates/vize_atelier_dom">vize_atelier_dom</a></td>
    <td>DOM (VDom) compiler</td>
  </tr>
  <tr>
    <td><img src="./crates/vize_atelier_vapor/logo.png" width="32" align="center" /> <a href="./crates/vize_atelier_vapor">vize_atelier_vapor</a></td>
    <td>Vapor mode compiler</td>
  </tr>
  <tr>
    <td><img src="./crates/vize_atelier_sfc/logo.png" width="32" align="center" /> <a href="./crates/vize_atelier_sfc">vize_atelier_sfc</a></td>
    <td>SFC (.vue) compiler</td>
  </tr>
  <tr>
    <td><img src="./crates/vize_vitrine/logo.png" width="32" align="center" /> <a href="./crates/vize_vitrine">vize_vitrine</a></td>
    <td>Node.js / WASM bindings</td>
  </tr>
  <tr>
    <td><img src="./crates/vize/logo.png" width="32" align="center" /> <a href="./crates/vize">vize</a></td>
    <td>Command-line interface</td>
  </tr>
  <tr>
    <td><img src="./crates/vize_canon/logo.png" width="32" align="center" /> <a href="./crates/vize_canon">vize_canon</a></td>
    <td>TypeScript type checker</td>
  </tr>
  <tr>
    <td><img src="./crates/vize_patina/logo.png" width="32" align="center" /> <a href="./crates/vize_patina">vize_patina</a></td>
    <td>Vue.js linter</td>
  </tr>
  <tr>
    <td><img src="./crates/vize_glyph/logo.png" width="32" align="center" /> <a href="./crates/vize_glyph">vize_glyph</a></td>
    <td>Vue.js formatter</td>
  </tr>
  <tr>
    <td><img src="./crates/vize_maestro/logo.png" width="32" align="center" /> <a href="./crates/vize_maestro">vize_maestro</a></td>
    <td>Language Server Protocol</td>
  </tr>
  <tr>
    <td><img src="./crates/vize_musea/logo.png" width="32" align="center" /> <a href="./crates/vize_musea">vize_musea</a></td>
    <td>Component gallery (Storybook)</td>
  </tr>
</table>

## Naming Theme

Vize crates are named after **art and sculpture terminology**, reflecting how each component shapes and transforms Vue code:

| Name | Origin | Meaning |
|------|--------|---------|
| **Carton** | /kɑːˈtɒn/ | Artist's portfolio case — stores and organizes tools |
| **Relief** | /rɪˈliːf/ | Sculptural technique projecting from a surface — AST structure |
| **Armature** | /ˈɑːrmətʃər/ | Internal skeleton supporting a sculpture — parsing framework |
| **Atelier** | /ˌætəlˈjeɪ/ | Artist's workshop — compiler workspaces |
| **Vitrine** | /vɪˈtriːn/ | Glass display case — bindings exposing the compiler |
| **Canon** | /ˈkænən/ | Standard of ideal proportions — type checking |
| **Patina** | /ˈpætɪnə/ | Aged surface indicating quality — linting |
| **Glyph** | /ɡlɪf/ | Carved symbol or letterform — formatting |
| **Maestro** | /ˈmaɪstroʊ/ | Master conductor — LSP orchestration |
| **Musea** | /mjuːˈziːə/ | Plural of museum — component gallery |

## Architecture

<p align="center">
  <img src="./assets/architecture.png" alt="Vize Architecture" width="800" />
</p>

## Quick Start

```bash
mise install && mise setup
mise cli      # Enable `vize` CLI command
mise dev      # Playground
```

## Usage

### CLI

```bash
vize [COMMAND] [OPTIONS]
```

| Command | Description |
|---------|-------------|
| `build` | Compile Vue SFC files (default) |
| `fmt` | Format Vue SFC files |
| `lint` | Lint Vue SFC files |
| `check` | Type check Vue SFC files |
| `musea` | Start component gallery server |
| `lsp` | Start Language Server Protocol server |

```bash
vize --help           # Show help
vize <command> --help # Show command-specific help
```

**Examples:**

```bash
vize                              # Compile ./**/*.vue to ./dist
vize build src/**/*.vue -o out    # Custom input/output
vize build --ssr                  # SSR mode
vize build --script_ext=preserve  # Keep .ts/.tsx/.jsx extensions
vize fmt --check                  # Check formatting
vize lint --fix                   # Auto-fix lint issues
vize check --strict               # Strict type checking
```

### Node.js / WASM

```javascript
// Node.js
const { compileSfc } = require('@vize/native');
const { code } = compileSfc(`<template><div>{{ msg }}</div></template>`, { filename: 'App.vue' });

// Browser
import init, { compileSfc } from '@vize/wasm';
await init();
const { code } = compileSfc(`...`, { filename: 'App.vue' });
```

## Compiler Coverage

| Category | Coverage |
|----------|----------|
| SFC (`<script setup>`, macros, CSS) | 93% (13/14) |
| Template (directives, components) | 100% (22/22) |
| Vapor mode | 92% (11/12) |

## Performance

Compiling **15,000 SFC files** (36.9 MB):

|  | @vue/compiler-sfc | Vize | Speedup |
|--|-------------------|------|---------|
| **Single Thread** | 16.21s | 6.65s | **2.4x** |
| **Multi Thread** | 4.13s | 498ms | **8.3x** |

## License

MIT
