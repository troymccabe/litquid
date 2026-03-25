/**
 * LitQuid Liquid filter for marking client-side template values.
 *
 * A noop filter that acts as a marker for build-time extraction.
 * This allows Liquid to parse the template safely while marking
 * which values should be extracted for client-side Lit templates.
 *
 * @example
 * Usage in .liquid templates:
 * ```liquid
 * {{ firstName | csr: "${this.firstName}" }}
 * ```
 *
 * @example
 * Registration with liquidjs:
 * ```typescript
 * import { Liquid } from 'liquidjs';
 * import { registerCsr } from '@litquid/liquid-clienttemplatevalue';
 *
 * const engine = new Liquid();
 * registerCsr(engine);
 * ```
 */

import type { Liquid } from 'liquidjs';

/**
 * The csr filter function.
 * Returns the input value unchanged (passthrough).
 * @param value - The value from the previous filter/variable
 * @param clientExpression - The client-side expression (ignored at runtime, used by preprocessor)
 */
export function csr(value: unknown, clientExpression?: string): string {
    // clientExpression is only used by the Rust preprocessor at build time.
    // At SSR runtime we wrap the value in lit-part markers so @lit-labs/ssr-client
    // can adopt the server-rendered DOM and apply reactivity on hydration.
    const rendered = value?.toString() ?? '';
    return `<!--lit-part-->${rendered}<!--/lit-part-->`;
}

/**
 * Register the csr filter with a Liquid engine instance.
 */
export function registerCsr(engine: Liquid): void {
    engine.registerFilter('csr', csr);
}

export default csr;
