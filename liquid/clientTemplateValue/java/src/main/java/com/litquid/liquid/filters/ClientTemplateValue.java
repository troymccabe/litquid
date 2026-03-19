package com.litquid.liquid.filters;

import liqp.filters.Filter;

/**
 * LitQuid Liquid filter for marking client-side template values.
 *
 * A noop filter that acts as a marker for build-time extraction.
 * This allows Liquid to parse the template safely while marking
 * which values should be extracted for client-side Lit templates.
 *
 * Usage in .liquid templates:
 * <pre>
 * {{ firstName | clientTemplateValue: "${this.firstName}" }}
 * </pre>
 */
public class ClientTemplateValue extends Filter {

    public ClientTemplateValue() {
        super("clientTemplateValue");
    }

    @Override
    public Object apply(Object value, Object... params) {
        // Pass through the value unchanged
        // params[0] would be the client expression string, but we ignore it at runtime
        return value == null ? "" : value.toString();
    }

    /**
     * Register this filter with the Liqp template engine.
     *
     * @return a new instance of the filter for registration
     */
    public static ClientTemplateValue create() {
        return new ClientTemplateValue();
    }
}
