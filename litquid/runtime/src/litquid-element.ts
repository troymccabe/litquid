import { LitElement, nothing, TemplateResult } from 'lit';

/**
 * Base class for Lit elements that support SSR hydration.
 *
 * When a component extends LitQuidElement and is rendered server-side with
 * the `ssr` boolean attribute, the component will adopt the existing
 * declarative shadow DOM and skip re-rendering.
 *
 * For CSR components, renderTemplate() is called normally.
 *
 * @example
 * ```typescript
 * import { html, css } from 'lit';
 * import { LitQuidElement } from '@litquid/runtime';
 *
 * class MyComponent extends LitQuidElement {
 *     static properties = { name: { type: String } };
 *     static styles = css`:host { display: block; }`;
 *
 *     name = '';
 *
 *     renderTemplate(): TemplateResult {
 *         return html`<p>Hello ${this.name}</p>`;
 *     }
 * }
 * ```
 *
 * SSR HTML:
 * ```html
 * <my-component ssr>
 *     <template shadowrootmode="open">
 *         <!-- content -->
 *     </template>
 * </my-component>
 * ```
 */
export class LitQuidElement extends LitElement {
    private _ssrHydrated = false;

    override createRenderRoot(): HTMLElement | DocumentFragment {
        // Check for declarative shadow DOM from SSR
        if (this.shadowRoot) {
            // Check for ssr boolean attribute on the host element
            if (this.hasAttribute('ssr')) {
                this.removeAttribute('ssr');
                this._ssrHydrated = true;
            }
            return this.shadowRoot;
        }
        return super.createRenderRoot();
    }

    override render(): TemplateResult | typeof nothing {
        // Skip render if we adopted SSR content - return nothing to preserve existing DOM
        if (this._ssrHydrated) {
            return nothing;
        }
        return this.renderTemplate();
    }

    /**
     * Override this method in subclasses to define the component's template.
     * This is only called for CSR components (not SSR hydrated).
     */
    protected renderTemplate(): TemplateResult | typeof nothing {
        return nothing;
    }
}
