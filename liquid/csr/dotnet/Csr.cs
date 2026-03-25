using DotLiquid;

namespace LitQuid.Liquid.Filters
{
    /// <summary>
    /// LitQuid Liquid filter for marking client-side template values.
    /// Usage: {{ value | csr: "${this.propertyName}" }}
    /// </summary>
    public static class CsrFilter
    {
        /// <summary>
        /// A noop filter that acts as a marker for build-time extraction.
        /// This allows Liquid to parse the template safely while marking
        /// which values should be extracted for client-side Lit templates.
        /// </summary>
        /// <param name="input">The input value to pass through.</param>
        /// <param name="clientExpression">The client-side expression (ignored at runtime, used by preprocessor).</param>
        /// <returns>The string representation of the input, wrapped in lit-part markers.</returns>
        public static string Csr(object input, string clientExpression = "")
        {
            // clientExpression is only used by the Rust preprocessor at build time.
            // At SSR runtime we wrap the value in lit-part markers so @lit-labs/ssr-client
            // can adopt the server-rendered DOM and apply reactivity on hydration.
            var rendered = input?.ToString() ?? string.Empty;
            return $"<!--lit-part-->{rendered}<!--/lit-part-->";
        }

        /// <summary>
        /// Registers the filter with DotLiquid.
        /// Call this before rendering templates.
        /// </summary>
        public static void Register()
        {
            Template.RegisterFilter(typeof(CsrFilter));
        }
    }
}
