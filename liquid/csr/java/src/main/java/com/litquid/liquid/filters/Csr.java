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
 * {{ firstName | csr: "${this.firstName}" }}
 * </pre>
 */
public class Csr extends Filter {

    public Csr() {
        super("csr");
    }

    @Override
    public Object apply(Object value, Object... params) {
        // params[0] would be the client expression string, but we ignore it at runtime.
        // We wrap the value in lit-part markers so @lit-labs/ssr-client can adopt the
        // server-rendered DOM and apply reactivity on hydration.
        String rendered = value == null ? "" : value.toString();
        return "<!--lit-part-->" + rendered + "<!--/lit-part-->";
    }

    /**
     * Register this filter with the Liqp template engine.
     *
     * @return a new instance of the filter for registration
     */
    public static Csr create() {
        return new Csr();
    }
}
