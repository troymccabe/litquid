using System.Collections.Concurrent;
using System.Text.Json;
using DotLiquid;

namespace Sample.Services;

/// <summary>
/// Renders Liquid templates with full Lit SSR marker support.
///
/// For each SSR component the rendered Liquid output is wrapped in
/// <!--lit-part DIGEST-->...<!--/lit-part--> so @lit-labs/ssr-client can hydrate
/// the declarative shadow DOM and apply Lit reactivity without a re-render.
///
/// The digest is read from the .template.json manifest produced by the Rust
/// preprocessor alongside each .template.js file.
/// </summary>
public class TemplateService
{
    private readonly IWebHostEnvironment _env;

    // Digest cache: template name → base64 digest string
    private static readonly ConcurrentDictionary<string, string> DigestCache = new();

    public TemplateService(IWebHostEnvironment env)
    {
        _env = env;
    }

    /// <summary>
    /// Renders a component for SSR: runs the Liquid template, reads the Lit digest from
    /// the .template.json manifest, and injects <!--lit-part DIGEST--> markers so the
    /// client can hydrate the shadow DOM without re-rendering.
    ///
    /// Returns the full <template shadowrootmode="open"> string ready to nest inside
    /// the custom element tag.
    /// </summary>
    public async Task<string> RenderSsrComponentAsync(string templateName, object data)
    {
        var rendered = await RenderLiquidTemplateAsync(templateName, data);
        var digest   = await GetDigestAsync(templateName);
        return InjectLitPartMarkers(rendered, digest);
    }

    /// <summary>
    /// Renders a Liquid template with the given data object.
    /// Templates are loaded from samples/components/templates/.
    /// </summary>
    public async Task<string> RenderLiquidTemplateAsync(string templateName, object data)
    {
        var templatePath    = Path.Combine(_env.ContentRootPath, "..", "components", "templates", templateName);
        var templateContent = await File.ReadAllTextAsync(templatePath);
        var template        = Template.Parse(templateContent);
        return template.Render(Hash.FromAnonymousObject(data));
    }

    /// <summary>Reads an HTML view and replaces {{PLACEHOLDER}} tokens.</summary>
    public async Task<string> RenderViewAsync(string viewName, Dictionary<string, string> replacements)
    {
        var viewPath = Path.Combine(_env.ContentRootPath, "Views", viewName);
        var content  = await File.ReadAllTextAsync(viewPath);

        foreach (var (key, value) in replacements)
            content = content.Replace($"{{{{{key}}}}}", value);

        return content;
    }

    // -------------------------------------------------------------------------

    /// <summary>
    /// Returns the Lit template digest for the given template, reading it from the
    /// .template.json manifest written by the Rust preprocessor.
    /// The result is cached for the lifetime of the application.
    /// </summary>
    private async Task<string> GetDigestAsync(string templateName)
    {
        var baseName = Path.GetFileNameWithoutExtension(templateName); // e.g. "my-component"

        if (DigestCache.TryGetValue(baseName, out var cached))
            return cached;

        // Manifest lives next to the generated .template.js in components/scripts/templates/
        var manifestPath = Path.Combine(
            _env.ContentRootPath, "..", "components", "scripts", "templates",
            baseName + ".template.json");

        if (!File.Exists(manifestPath))
            throw new FileNotFoundException(
                $"Lit template manifest not found at '{manifestPath}'. " +
                "Run the Rust preprocessor (npm run build:toolchain) first.", manifestPath);

        var json   = await File.ReadAllTextAsync(manifestPath);
        var digest = JsonSerializer.Deserialize<JsonElement>(json).GetProperty("digest").GetString()
                     ?? throw new InvalidDataException($"'digest' missing in {manifestPath}");

        DigestCache[baseName] = digest;
        return digest;
    }

    /// <summary>
    /// Injects <!--lit-part DIGEST-->...<!--/lit-part--> inside a
    /// &lt;template shadowrootmode="open"&gt; string produced by Liquid rendering.
    /// </summary>
    private static string InjectLitPartMarkers(string rendered, string digest)
    {
        var openTagEnd   = rendered.IndexOf('>');
        var closeTagStart = rendered.LastIndexOf("</template>");

        if (openTagEnd < 0 || closeTagStart < 0)
            return rendered; // not a template element — return as-is

        var openTag  = rendered[..(openTagEnd + 1)];
        var inner    = rendered[(openTagEnd + 1)..closeTagStart];
        return $"{openTag}<!--lit-part {digest}-->{inner}<!--/lit-part--></template>";
    }
}
