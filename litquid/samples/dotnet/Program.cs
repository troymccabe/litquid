using LitQuid.Liquid.Filters;
using Sample.Endpoints;
using Sample.Services;

var builder = WebApplication.CreateBuilder(args);

// Add services
builder.Services.AddCors(options =>
{
    options.AddDefaultPolicy(policy =>
    {
        policy.AllowAnyOrigin()
              .AllowAnyHeader()
              .AllowAnyMethod();
    });
});

builder.Services.AddScoped<TemplateService>();

var app = builder.Build();

// Register the ClientTemplateValue filter with DotLiquid
ClientTemplateValue.Register();

// Middleware
app.UseCors();

// Static files with no-cache headers for development
app.UseStaticFiles(new StaticFileOptions
{
    OnPrepareResponse = ctx =>
    {
        ctx.Context.Response.Headers.Append("Cache-Control", "no-cache, no-store, must-revalidate");
        ctx.Context.Response.Headers.Append("Pragma", "no-cache");
        ctx.Context.Response.Headers.Append("Expires", "0");
    }
});

// Map endpoints
app.MapHomeEndpoints();

app.Run();
