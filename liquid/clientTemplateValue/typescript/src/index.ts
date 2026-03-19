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
 * {{ firstName | clientTemplateValue: "${this.firstName}" }}
 * ```
 *
 * @example
 * Registration with liquidjs:
 * ```typescript
 * import { Liquid } from 'liquidjs';
 * import { registerClientTemplateValue } from '@litquid/liquid-clienttemplatevalue';
 *
 * const engine = new Liquid();
 * registerClientTemplateValue(engine);
 * ```
 */

import type { Liquid } from 'liquidjs';

/**
 * The clientTemplateValue filter function.
 * Returns the input value unchanged (passthrough).
 * @param value - The value from the previous filter/variable
 * @param clientExpression - The client-side expression (ignored at runtime, used by preprocessor)
 */
export function clientTemplateValue(value: unknown, clientExpression?: string): string {
    // clientExpression is only used by the Rust preprocessor at build time
    return value?.toString() ?? '';
}

/**
 * Register the clientTemplateValue filter with a Liquid engine instance.
 */
export function registerClientTemplateValue(engine: Liquid): void {
    engine.registerFilter('clientTemplateValue', clientTemplateValue);
}

export default clientTemplateValue;
