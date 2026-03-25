// Side-effect import: patches LitElement to call hydrate() instead of render()
// on the first update when <!--lit-part--> markers are present in the shadow root.
// This enables full Lit reactivity on server-rendered declarative shadow DOM.
import '@lit-labs/ssr-client/lit-element-hydrate-support.js';

import { LitElement, nothing, TemplateResult } from 'lit';

/**
 * Base class for Lit elements with SSR hydration via @lit-labs/ssr-client.
 *
 * When a component's shadow root contains <!--lit-part DIGEST-->...<!--/lit-part-->
 * markers (produced by a server rendering the Liquid template with the `csr` filter),
 * LitElement will hydrate the existing DOM rather than re-rendering, preserving the
 * server-rendered content while applying full Lit reactivity.
 *
 * The digest in the marker is computed by the Rust preprocessor (matching
 * @lit-labs/ssr-client's `digestForTemplateResult`) and emitted into the
 * `.template.json` manifest alongside the `.template.js` module. The server reads
 * the manifest to wrap its Liquid-rendered output with the correct markers:
 *
 * ```html
 * <my-component>
 *   <template shadowrootmode="open">
 *     <!--lit-part DIGEST-->
 *       <h2><!--lit-part-->Alice<!--/lit-part--></h2>
 *     <!--/lit-part-->
 *   </template>
 * </my-component>
 * ```
 *
 * @example
 * ```typescript
 * import { LitQuidElement, css, type TemplateResult } from '@litquid/runtime';
 *
 * class MyComponent extends LitQuidElement {
 *     static properties = { name: { type: String } };
 *     static styles = css`:host { display: block; }`;
 *
 *     name = '';
 *
 *     renderTemplate(): TemplateResult {
 *         return getTemplate(this);
 *     }
 * }
 * ```
 */
export class LitQuidElement extends LitElement {
    override render(): TemplateResult | typeof nothing {
        return this.renderTemplate();
    }

    /**
     * Override this method to define the component's template.
     * Must return the same template that was rendered server-side so the digest matches.
     */
    protected renderTemplate(): TemplateResult | typeof nothing {
        return nothing;
    }
}
