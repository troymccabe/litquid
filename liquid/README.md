# LitQuid Liquid Filters

Language-specific implementations of the `clientTemplateValue` filter for Liquid templating engines.

## Purpose

The `clientTemplateValue` filter is a **noop passthrough filter** that:

1. Returns the input value unchanged during SSR
2. Acts as a **marker** for the LitQuid preprocessor to extract client-side expressions

## Usage in Templates

```liquid
<h2>{{ firstName | capitalize | clientTemplateValue: "${this.firstName}" }}</h2>
<p>{{ lastName | clientTemplateValue: "${this.lastName}" }}</p>
```

The argument to `clientTemplateValue:` is a quoted string containing the JavaScript template literal expression used in the generated Lit template.

## Implementations

### .NET (DotLiquid)

```bash
cd dotnet && dotnet build
```

```csharp
using LitQuid.Liquid.Filters;

// Register before rendering
ClientTemplateValue.Register();
```

### Java (Liqp)

```bash
cd java && mvn package
```

```java
import com.litquid.filters.ClientTemplateValue;

TemplateParser parser = TemplateParser.builder()
    .withFilter(ClientTemplateValue.create())
    .build();
```

### TypeScript (LiquidJS)

```bash
cd typescript && npm install && npm run build
```

```typescript
import { Liquid } from 'liquidjs';
import { registerClientTemplateValue } from '@litquid/liquid-clienttemplatevalue';

const engine = new Liquid();
registerClientTemplateValue(engine);
```

## Adding New Languages

To add support for a new language:

1. Create a new directory under `clientTemplateValue/{language}/`
2. Implement a filter that:
   - Takes an input value
   - Returns `input?.toString() ?? ''`
   - Accepts (and ignores) a parameter for the client expression
3. Add build configuration for the language
4. Document usage in this README

## License

MIT
