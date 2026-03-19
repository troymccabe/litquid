# LitQuid - Liquid to Lit Template Pipeline

## Overview

LitQuid converts Liquid templates to Lit template modules with SSR hydration support.

## Project Structure

```
├── liquid/                        # clientTemplateValue filter implementations
│   └── clientTemplateValue/
│       ├── dotnet/                # .NET DotLiquid
│       ├── java/                  # Java Liqp
│       └── typescript/            # TypeScript LiquidJS
│
└── litquid/                       # Core library, tooling, and samples
    ├── runtime/                   # @litquid/runtime - LitQuidElement
    ├── toolchain/                 # Preprocessor & watch server (Rust)
    └── samples/
        ├── components/            # Shared templates & scripts
        └── dotnet/                # .NET sample app
```

## Key Concepts

### 1. clientTemplateValue Filter

A noop filter that marks values for client-side extraction:

```liquid
{{ firstName | capitalize | clientTemplateValue: "${this.firstName}" }}
```

- **SSR**: Returns the processed value (after `capitalize`)
- **Preprocessor**: Extracts `${this.firstName}` for the Lit template

### 2. LitQuidElement

Base class for Lit components with SSR hydration:

```typescript
import { LitQuidElement } from '@litquid/runtime';

class MyComponent extends LitQuidElement {
    renderTemplate() {
        return getTemplate(this);
    }
}
```

- Detects SSR content via `ssr` boolean attribute
- Adopts declarative shadow DOM
- Skips re-render for hydrated content

### 3. Rust Preprocessor

Converts `.liquid` → `.template.js`:

```bash
# One-time build
litquid --input ./templates --output ./dist

# Watch mode with live reload server
litquid-watch --input ./templates --output ./dist --port 35729
```

Options:
- `--lit-import <path>` - Customize lit import path (default: `"lit"`)
- `--port <port>` - Live reload server port (default: `35729`)

### 4. Live Reload

`litquid-watch` includes a built-in SSE server. Add to your HTML:

```html
<script src="http://localhost:35729/livereload.js"></script>
```

## Build Commands

```bash
# Runtime package
cd litquid/runtime && npm install && npm run build

# TypeScript filter
cd liquid/clientTemplateValue/typescript && npm install && npm run build

# .NET filter
cd liquid/clientTemplateValue/dotnet && dotnet build

# Rust toolchain
cd litquid/toolchain && cargo build --release

# .NET sample
cd litquid/samples/dotnet && dotnet run
```

## SSR HTML Structure

```html
<my-component ssr>
    <template shadowrootmode="open">
        <!-- Server-rendered content -->
    </template>
</my-component>
```
