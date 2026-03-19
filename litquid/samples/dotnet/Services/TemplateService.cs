using DotLiquid;
using LitQuid.Liquid.Filters;

namespace Sample.Services;

/// <summary>
/// Service for rendering Liquid templates with SSR support.
/// </summary>
public class TemplateService
{
    private readonly IWebHostEnvironment _env;

    public TemplateService(IWebHostEnvironment env)
    {
        _env = env;
    }

    /// <summary>
    /// Renders a Liquid template with the given data.
    /// Templates are loaded from the shared components directory.
    /// </summary>
    public async Task<string> RenderLiquidTemplateAsync(string templateName, object data)
    {
        // Use shared templates from samples/components/templates
        var templatePath = Path.Combine(_env.ContentRootPath, "..", "components", "templates", templateName);
        var templateContent = await File.ReadAllTextAsync(templatePath);
        var template = Template.Parse(templateContent);
        return template.Render(Hash.FromAnonymousObject(data));
    }

    /// <summary>
    /// Reads an HTML view and replaces placeholders.
    /// </summary>
    public async Task<string> RenderViewAsync(string viewName, Dictionary<string, string> replacements)
    {
        var viewPath = Path.Combine(_env.ContentRootPath, "Views", viewName);
        var content = await File.ReadAllTextAsync(viewPath);

        foreach (var (key, value) in replacements)
        {
            content = content.Replace($"{{{{{key}}}}}", value);
        }

        return content;
    }
}
