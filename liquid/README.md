# LitQuid Liquid Filters

Language-specific implementations of the `csr` filter for Liquid templating engines.

## Purpose

The `csr` filter serves two roles depending on context:

- **SSR runtime**: Wraps the rendered value in `<!--lit-part-->value<!--/lit-part-->` markers so `@lit-labs/ssr-client` can identify each reactive binding position and hydrate without a full re-render
- **Preprocessor (build time)**: Acts as a marker so the Rust preprocessor knows which expressions become Lit bindings in the generated `.template.js`

> **Note:** If you use the **Generated path** (`--emit csharp` / `--emit go` etc.) the `csr` filter is only needed for the Liquid path. Generated renderers bake the markers directly into the emitted `Render(...)` function — no filter registration required.

## Usage in Templates

```liquid
{{-- explicit arg: preprocessor uses the provided JS expression --}}
<h2>{{ firstName | capitalize | csr: "${this.firstName}" }}</h2>

{{-- no arg: preprocessor auto-translates the preceding filter chain --}}
<h2>{{ firstName | capitalize | csr }}</h2>
<p>{{ lastName | csr }}</p>
```

When no argument is provided, the preprocessor auto-translates the preceding filter chain to an equivalent JS expression.

## Implementations

### .NET (DotLiquid)

```bash
cd dotnet && dotnet build
```

DotLiquid registers filters by **method name**. The filter class must expose a method named `Csr` (matched case-insensitively) — the enclosing class is named `CsrFilter` to avoid the C# restriction on methods sharing their class name.

```csharp
using LitQuid.Liquid.Filters;

// Register before rendering templates
CsrFilter.Register();
```

### Java (Liqp)

```bash
cd java && mvn package
```

```java
import com.litquid.liquid.filters.Csr;

TemplateParser parser = TemplateParser.builder()
    .withFilter(Csr.create())
    .build();
```

### TypeScript (LiquidJS)

```bash
cd typescript && npm install && npm run build
```

```typescript
import { Liquid } from 'liquidjs';
import { registerCsr } from '@litquid/liquid-csr';

const engine = new Liquid();
registerCsr(engine);
```

## Adding New Languages

To add support for a new language:

1. Create a new directory under `csr/{language}/`
2. Implement a filter that:
   - Is registered under the name `csr`
   - Takes an input value and an optional client expression parameter (ignored at runtime)
   - Returns `<!--lit-part-->` + `input.toString()` + `<!--/lit-part-->`
3. Add build configuration for the language
4. Document usage in this README

## License

MIT
