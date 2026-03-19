# @litquid/runtime

LitQuid runtime - SSR hydration support for Lit components.

## Installation

```bash
npm install @litquid/runtime
```

## Usage

```typescript
import { html, css, TemplateResult } from 'lit';
import { LitQuidElement } from '@litquid/runtime';
import { getTemplate } from './my-component.template.js';

class MyComponent extends LitQuidElement {
    static properties = {
        name: { type: String }
    };

    static styles = css`
        :host { display: block; }
    `;

    name = '';

    protected renderTemplate(): TemplateResult {
        return getTemplate(this);
    }
}

customElements.define('my-component', MyComponent);
```

## How It Works

`LitQuidElement` extends `LitElement` with SSR hydration support:

1. **SSR Detection**: Checks for the `ssr` boolean attribute on the host element
2. **Shadow DOM Adoption**: If SSR content exists (declarative shadow DOM), adopts it
3. **Skip Re-render**: For SSR-hydrated components, `render()` returns `nothing`
4. **CSR Fallback**: For non-SSR components, calls `renderTemplate()`

### SSR HTML Structure

```html
<my-component ssr>
    <template shadowrootmode="open">
        <!-- SSR content here -->
    </template>
</my-component>
```

## API

### `LitQuidElement`

Base class extending `LitElement`.

#### Methods

- `renderTemplate(): TemplateResult | typeof nothing` - Override to provide the component template for CSR rendering.

## Building

```bash
npm install
npm run build
```

## License

MIT
