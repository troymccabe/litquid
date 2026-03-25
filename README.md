# LitQuid

Converts Liquid templates to Lit template modules with SSR hydration support.

## Project Structure

```
├── liquid/                        # Liquid filter implementations
│   └── csr/
│       ├── dotnet/                # .NET DotLiquid filter
│       ├── java/                  # Java Liqp filter
│       └── typescript/            # TypeScript LiquidJS filter
│
└── litquid/                       # Core library, tooling, and samples
    ├── runtime/                   # @litquid/runtime - SSR hydration
    │   └── src/
    │       └── litquid-element.ts # LitQuidElement base class
    │
    ├── toolchain/                 # Preprocessor & watch server (Rust)
    │   ├── src/
    │   │   ├── lib.rs             # Core processing logic
    │   │   ├── main.rs            # CLI preprocessor
    │   │   ├── watch.rs           # File watcher with live reload
    │   │   └── codegen/           # Server-side code emitters
    │   │       ├── mod.rs         # TargetEmitter trait
    │   │       └── csharp.rs      # C# emitter
    │   └── Cargo.toml
    │
    └── samples/
        ├── components/            # Shared component definitions
        │   ├── templates/         # .liquid template files
        │   └── scripts/           # Lit component TypeScript files
        └── dotnet/                # .NET sample application
            └── Generated/         # Build-time emitted C# renderers
```

## Overview

LitQuid gives you two paths for SSR content — choose per component, mix freely:

| | Liquid path | Generated path |
|---|---|---|
| **How** | Liquid engine renders `.liquid` at request time | Rust preprocessor emits typed render function at build time |
| **Runtime dep** | Liquid engine + `csr` filter | None — pure `string.Concat` |
| **Flexibility** | Full Liquid template language | Parameterised values only |
| **Best for** | Templates that need runtime conditionals, loops, includes | Maximum throughput; simple data binding |

Both paths produce **identical HTML** with the same `<!--lit-part DIGEST-->` markers, so `@lit-labs/ssr-client` hydrates and applies Lit reactivity the same way regardless of which path rendered the page.

## How It Works

### Step 1 — Write a `.liquid` template with `csr` markers

```liquid
<template shadowrootmode="open">
  <div class="card">
    <p class="name">{{ firstName | capitalize | csr }}</p>
    <p class="meta">{{ role | csr }} · {{ age | csr }} yrs</p>
  </div>
</template>
```

The `csr` filter is a passthrough at render time. It also tells the preprocessor which expressions become reactive Lit bindings.

### Step 2 — Run the preprocessor

```bash
cd litquid/toolchain

# JS only (default) — generates .template.js for client-side Lit
cargo run --bin litquid -- --input ./templates --output ./dist

# JS + C# renderer — also emits a typed server-side render function
cargo run --bin litquid -- --input ./templates --output ./dist --emit csharp --namespace MyApp.Generated
```

Outputs per template:

| File | Used by |
|---|---|
| `my-component.template.js` | Client-side Lit (`getTemplate`) |
| `my-component.template.json` | Liquid SSR path (digest lookup) |
| `my-component.template.cs` | Generated SSR path (`MyComponentTemplate.Render`) |

### Step 3 — Render SSR content (choose a path)

**Liquid path** — render the `.liquid` file at request time, inject markers:

```csharp
// TemplateService wraps the DotLiquid output with <!--lit-part DIGEST--> markers
// by reading the digest from the .template.json manifest.
var dsd = await templateService.RenderSsrComponentAsync("my-component.liquid",
    new { firstName = "Alice", role = "Engineer", age = 28 });
```

**Generated path** — call the emitted C# method directly, no template engine:

```csharp
// Digest is a compile-time constant; no manifest read, no async, no allocations
// beyond the string concatenation itself.
var dsd = MyComponentTemplate.Render("Alice", lastName, role, age.ToString());
```

Both return a `<template shadowrootmode="open"><!--lit-part DIGEST-->...<!--/lit-part--></template>` string ready to nest inside the custom element tag.

### Step 4 — Emit the HTML

```html
<my-component ssr firstname="Alice" role="Engineer" age="28">
  <!-- dsd variable from either path above -->
</my-component>
```

### Step 5 — Client hydration

```typescript
import { LitQuidElement, css, type TemplateResult } from '@litquid/runtime';
import { getTemplate } from './templates/my-component.template.js';

class MyComponent extends LitQuidElement {
    static properties = { firstName: { type: String }, role: { type: String }, age: { type: Number } };
    protected renderTemplate(): TemplateResult { return getTemplate(this); }
}
customElements.define('my-component', MyComponent);
```

`@lit-labs/ssr-client` hydrates the `<!--lit-part-->` markers — no re-render, full Lit reactivity.

## Packages

### @litquid/runtime

Base class for Lit components with SSR hydration support.

```bash
cd litquid/runtime && npm install && npm run build
```

### @litquid/liquid-csr (TypeScript)

```bash
cd liquid/csr/typescript && npm install && npm run build
```

### LitQuid.Liquid.Csr (.NET)

```bash
cd liquid/csr/dotnet && dotnet build
```

### Java Filter

```bash
cd liquid/csr/java && mvn package
```

## Toolchain

### Preprocessor

Converts `.liquid` templates to Lit template modules (and optionally server-side renderers):

```bash
cd litquid/toolchain
cargo run --bin litquid -- \
  --input ./templates \
  --output ./dist \
  [--emit csharp] \
  [--namespace MyApp.Generated] \
  [--lit-import "lit"]
```

Options:
- `--emit <targets>` - Additional server-side render targets. Supported: `csharp`
- `--namespace <ns>` - Namespace for generated code (used with `--emit csharp`; default: `LitQuid.Generated`)
- `--lit-import <path>` - Customize lit import path (default: `"lit"`)

### Watch Server

File watcher with built-in live reload server. Regenerates all outputs (JS, JSON, and any `--emit` targets) on every `.liquid` change:

```bash
cargo run --bin litquid-watch -- \
  --input ./templates \
  --output ./dist \
  [--emit csharp] \
  [--namespace MyApp.Generated] \
  [--port 35729]
```

Add the live reload script to your HTML:

```html
<script src="http://localhost:35729/livereload.js"></script>
```

## Samples

### .NET Sample

Demonstrates all three rendering strategies in a single page — Liquid SSR, Generated SSR, and CSR:

```bash
cd litquid/samples/dotnet
dotnet run
```

## License

MIT
