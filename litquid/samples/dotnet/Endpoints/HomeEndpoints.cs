using Sample.Models;
using Sample.Services;

namespace Sample.Endpoints;

/// <summary>
/// Endpoint handlers for the home page.
/// </summary>
public static class HomeEndpoints
{
    public static void MapHomeEndpoints(this WebApplication app)
    {
        app.MapGet("/", HandleHomeAsync);
    }

    private static async Task HandleHomeAsync(HttpContext context, TemplateService templateService)
    {
        // SSR data - this would typically come from a database or API
        var ssrData = new UserViewModel
        {
            FirstName = "John",
            LastName = "Doe",
            Age = 30
        };

        // Render the SSR version of the component using Liquid
        var ssrRenderedContent = await templateService.RenderLiquidTemplateAsync(
            "my-component.liquid",
            new { firstName = ssrData.FirstName, lastName = ssrData.LastName, age = ssrData.Age }
        );

        // Render the full page view with the SSR content
        var html = await templateService.RenderViewAsync("index.html", new Dictionary<string, string>
        {
            { "SSR_CONTENT", ssrRenderedContent }
        });

        context.Response.ContentType = "text/html";
        await context.Response.WriteAsync(html);
    }
}
