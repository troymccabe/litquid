# LitQuid Toolchain & Samples

Bridge tooling and sample applications for LitQuid.

## Toolchain

### Rust Preprocessor (`litquid`)

Converts `.liquid` templates to Lit template modules.

```bash
cd toolchain
cargo build --release

# One-time processing
cargo run --bin litquid -- \
  --input ./path/to/templates \
  --output ./path/to/output

# Watch mode with live reload
cargo run --bin litquid-watch -- \
  --input ./path/to/templates \
  --output ./path/to/output
```

Options:
- `--port <port>` - Live reload server port (default: `35729`)
- `--lit-import <path>` - Customize lit import path (default: `"lit"`)

Add the live reload script to your HTML:

```html
<script src="http://localhost:35729/livereload.js"></script>
```

### How the Preprocessor Works

1. Parses `.liquid` files
2. Finds `{{ ... }}` expressions containing `clientTemplateValue(...)`
3. Extracts the argument (the client-side JavaScript expression)
4. Generates a `.template.js` file with a `getTemplate(ctx)` function

**Input:**
```liquid
<h2>{{ firstName | capitalize | clientTemplateValue(${this.firstName}) }}</h2>
```

**Output:**
```javascript
import { html } from 'lit';

export function getTemplate(ctx) {
  return html`<h2>${this.firstName}</h2>`;
}
```

## Samples

### Shared Components

The `components/` directory contains shared component definitions:

- `templates/` - `.liquid` template files (SSR source)
- `scripts/` - TypeScript Lit component definitions (CSR)

### .NET Sample

A complete ASP.NET Core sample demonstrating SSR + CSR:

```bash
cd samples/dotnet
dotnet run
```

Features:
- Server-side Liquid rendering with DotLiquid
- Client-side hydration with LitQuidElement
- Live reload for development
- esbuild for JS/CSS bundling

## License

MIT
