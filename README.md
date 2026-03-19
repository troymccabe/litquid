# LitQuid

Converts Liquid templates to Lit template modules with SSR hydration support.

## Project Structure

```
├── liquid/                        # Liquid filter implementations
│   └── clientTemplateValue/
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
    │   │   └── watch.rs           # File watcher with live reload
    │   └── Cargo.toml
    │
    └── samples/
        ├── components/            # Shared component definitions
        │   ├── templates/         # .liquid template files
        │   └── scripts/           # Lit component TypeScript files
        └── dotnet/                # .NET sample application
```

## Overview

LitQuid enables a federated template approach where:

1. **Liquid templates** are rendered server-side (SSR) with real data
2. **Lit components** hydrate the SSR content on the client
3. **Shared templates** ensure consistency between server and client

### How It Works

1. Write `.liquid` templates with `clientTemplateValue` markers:
   ```liquid
   <h2>{{ firstName | capitalize | clientTemplateValue: "${this.firstName}" }}</h2>
   ```

2. The Rust preprocessor extracts client-side expressions:
   ```javascript
   // Generated: my-component.template.js
   export function getTemplate(ctx) {
     return html`<h2>${ctx.firstName}</h2>`;
   }
   ```

3. Lit components use the generated template:
   ```typescript
   import { LitQuidElement } from '@litquid/runtime';
   import { getTemplate } from './my-component.template.js';

   class MyComponent extends LitQuidElement {
     renderTemplate() {
       return getTemplate(this);
     }
   }
   ```

4. SSR renders the Liquid template; client hydrates with `LitQuidElement`

## Packages

### @litquid/runtime

Base class for Lit components with SSR hydration support.

```bash
cd litquid/runtime && npm install && npm run build
```

### @litquid/liquid-clienttemplatevalue (TypeScript)

```bash
cd liquid/clientTemplateValue/typescript && npm install && npm run build
```

### LitQuid.Liquid.ClientTemplateValue (.NET)

```bash
cd liquid/clientTemplateValue/dotnet && dotnet build
```

### Java Filter

```bash
cd liquid/clientTemplateValue/java && mvn package
```

## Toolchain

### Preprocessor

Converts `.liquid` templates to Lit template modules:

```bash
cd litquid/toolchain
cargo run --bin litquid -- --input ./templates --output ./dist
```

### Watch Server

File watcher with built-in live reload server:

```bash
cargo run --bin litquid-watch -- \
  --input ./templates \
  --output ./dist
```

Add the live reload script to your HTML:

```html
<script src="http://localhost:35729/livereload.js"></script>
```

Options:
- `--port <port>` - Live reload server port (default: `35729`)
- `--lit-import <path>` - Customize lit import path (default: `"lit"`)

## Samples

### .NET Sample

```bash
cd litquid/samples/dotnet
dotnet run
```

## License

MIT
