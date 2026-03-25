# LitQuid - Liquid to Lit Template Pipeline

## Overview

LitQuid converts Liquid templates to Lit template modules with SSR hydration support.

## Project Structure

```
├── liquid/                        # csr filter implementations
│   └── csr/
│       ├── dotnet/                # .NET DotLiquid
│       ├── java/                  # Java Liqp
│       └── typescript/            # TypeScript LiquidJS
│
└── litquid/                       # Core library, tooling, and samples
    ├── runtime/                   # @litquid/runtime - LitQuidElement
    ├── toolchain/                 # Preprocessor & watch server (Rust)
    │   └── src/codegen/           # TargetEmitter trait + language emitters
    └── samples/
        ├── components/            # Shared templates & scripts
        └── dotnet/                # .NET sample app
            └── Generated/         # Build-time emitted C# renderers
```

## Key Concepts

### 1. `csr` Filter

A Liquid filter that marks dynamic values for client-side extraction:

```liquid
{{-- explicit arg --}}
{{ firstName | capitalize | csr: "${this.firstName}" }}

{{-- no arg: preprocessor auto-translates the filter chain --}}
{{ firstName | capitalize | csr }}
```

- **SSR (Liquid path)**: Wraps the processed value in `<!--lit-part-->value<!--/lit-part-->` markers so `@lit-labs/ssr-client` can hydrate each binding position
- **SSR (Generated path)**: Not used — the preprocessor emits the markers directly into the `Render()` function
- **Preprocessor**: Extracts or auto-translates the expression for the Lit template

### 2. Two SSR Paths

Both paths produce identical HTML. Choose per-component or mix freely within a page.

**Liquid path** — runtime rendering via Liquid engine:
```csharp
var dsd = await templateService.RenderSsrComponentAsync("my-component.liquid", data);
// → reads digest from my-component.template.json
// → csr filter wraps each binding in <!--lit-part--> markers
// → injects outer <!--lit-part DIGEST--> wrapper
```

**Generated path** — build-time emitted C# renderer:
```csharp
var dsd = MyComponentTemplate.Render(firstName, lastName, role, age.ToString());
// → digest is a compile-time constant
// → no template engine, no async, no manifest read
// → pure string.Concat
```

### 3. LitQuidElement

Base class for Lit components with SSR hydration:

```typescript
import { LitQuidElement, css, type TemplateResult } from '@litquid/runtime';

class MyComponent extends LitQuidElement {
    renderTemplate() {
        return getTemplate(this);
    }
}
```

`@lit-labs/ssr-client` (imported via `@litquid/runtime`) patches `LitElement` to call `hydrate()` on the first update when `<!--lit-part-->` markers are present — no re-render, full reactivity.

### 4. Rust Preprocessor

Converts `.liquid` → `.template.js` (and optionally server-side renderers):

```bash
# JS output only
litquid --input ./templates --output ./dist

# JS + C# server-side renderers
litquid --input ./templates --output ./dist \
  --emit csharp --namespace MyApp.Generated

# Watch mode with live reload server
litquid-watch --input ./templates --output ./dist \
  --emit csharp --namespace MyApp.Generated --port 35729
```

Options:
- `--emit <targets>` - Server-side render targets. Supported: `csharp`
- `--namespace <ns>` - Namespace for generated code (default: `LitQuid.Generated`)
- `--lit-import <path>` - Customize lit import path (default: `"lit"`)
- `--port <port>` - Live reload server port (default: `35729`)

### 5. Live Reload

`litquid-watch` includes a built-in SSE server. Add to your HTML:

```html
<script src="http://localhost:35729/livereload.js"></script>
```

## Build Commands

```bash
# Runtime package
cd litquid/runtime && npm install && npm run build

# TypeScript filter
cd liquid/csr/typescript && npm install && npm run build  # registers as 'csr' filter

# .NET filter
cd liquid/csr/dotnet && dotnet build

# Rust toolchain
cd litquid/toolchain && cargo build --release

# .NET sample (runs npm build + dotnet)
cd litquid/samples/dotnet && dotnet run
```

## SSR HTML Structure

Both SSR paths produce the same structure:

```html
<my-component ssr firstname="Alice" age="28" role="Engineer">
    <template shadowrootmode="open">
        <!--lit-part mKjZkQEA/6Y=-->
          <div class="card">
            <p><!--lit-part-->Alice<!--/lit-part--></p>
            <p><!--lit-part-->Engineer<!--/lit-part--> · <!--lit-part-->28<!--/lit-part--> yrs</p>
          </div>
        <!--/lit-part-->
    </template>
</my-component>
```

The outer `<!--lit-part DIGEST-->` digest is verified by `@lit-labs/ssr-client`. The inner `<!--lit-part-->` markers identify each reactive binding position.

## .NET Filter Registration

DotLiquid registers filters by **method name**. The filter class must have a method named `Csr` (matched case-insensitively to `{{ value | csr }}`):

```csharp
// liquid/csr/dotnet/Csr.cs
public static class CsrFilter
{
    public static string Csr(object input, string clientExpression = "")
        => $"<!--lit-part-->{input?.ToString() ?? ""}<!--/lit-part-->";

    public static void Register() => Template.RegisterFilter(typeof(CsrFilter));
}

// Program.cs
CsrFilter.Register();
```
