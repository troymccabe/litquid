using DotLiquid;

namespace LitQuid.Liquid.Filters
{
    /// <summary>
    /// LitQuid Liquid filter for marking client-side template values.
    /// Usage: {{ value | clientTemplateValue: "${this.propertyName}" }}
    /// </summary>
    public static class ClientTemplateValue
    {
        /// <summary>
        /// A noop filter that acts as a marker for build-time extraction.
        /// This allows Liquid to parse the template safely while marking
        /// which values should be extracted for client-side Lit templates.
        /// </summary>
        /// <param name="input">The input value to pass through.</param>
        /// <param name="clientExpression">The client-side expression (ignored at runtime, used by preprocessor).</param>
        /// <returns>The string representation of the input, or empty string if null.</returns>
        public static string ClientTemplateValueFilter(object input, string clientExpression = "")
        {
            // The clientExpression is only used by the Rust preprocessor
            // At runtime, we just return the server-rendered value
            return input?.ToString() ?? string.Empty;
        }

        /// <summary>
        /// Registers the filter with DotLiquid.
        /// Call this before rendering templates.
        /// </summary>
        public static void Register()
        {
            Template.RegisterFilter(typeof(ClientTemplateValue));
        }
    }
}
