using System.Diagnostics;
using System.Text;
using Sample.Generated;
using Sample.Models;
using Sample.Services;

namespace Sample.Endpoints;

public static class HomeEndpoints
{
    private static readonly UserViewModel[] Users =
    [
        new() { FirstName = "Alice",   LastName = "Chen",     Age = 28, Role = "Engineer"  },
        new() { FirstName = "Marcus",  LastName = "Silva",    Age = 34, Role = "Designer"  },
        new() { FirstName = "Priya",   LastName = "Patel",    Age = 26, Role = "Product"   },
        new() { FirstName = "Omar",    LastName = "Khalil",   Age = 31, Role = "DevOps"    },
        new() { FirstName = "Sara",    LastName = "Johnson",  Age = 29, Role = "Data"      },
        new() { FirstName = "Luca",    LastName = "Romano",   Age = 38, Role = "Architect" },
        new() { FirstName = "Yuki",    LastName = "Tanaka",   Age = 25, Role = "Frontend"  },
        new() { FirstName = "Elena",   LastName = "Petrov",   Age = 32, Role = "Backend"   },
    ];

    public static void MapHomeEndpoints(this WebApplication app)
    {
        app.MapGet("/", HandleHomeAsync);
    }

    private static async Task HandleHomeAsync(HttpContext context, TemplateService templateService)
    {
        // Pre-compute team stats from the Users array.
        var memberCount  = Users.Length.ToString();
        var avgAge       = ((int)Math.Round(Users.Average(u => u.Age))).ToString();
        var uniqueRoles  = Users.Select(u => u.Role).Distinct().Count().ToString();
        var seniorCount  = Users.Count(u => u.Age >= 30).ToString();

        // ── Liquid SSR ──────────────────────────────────────────────────────────
        // Renders stat-cards #1 (avg age) and #3 (senior count) plus user-cards
        // [0-2] — all via DotLiquid + csr filter, in parallel.
        var swLiquid = Stopwatch.StartNew();

        var liquidTasks = new Task<string>[]
        {
            templateService.RenderSsrComponentAsync("stat-card.liquid",
                new { count = avgAge,      label = "Avg Age", sub = "years"           }),
            templateService.RenderSsrComponentAsync("stat-card.liquid",
                new { count = seniorCount, label = "30+",     sub = "senior members"  }),
            templateService.RenderSsrComponentAsync("my-component.liquid",
                new { firstName = Users[0].FirstName, lastName = Users[0].LastName,
                      age = Users[0].Age, role = Users[0].Role }),
            templateService.RenderSsrComponentAsync("my-component.liquid",
                new { firstName = Users[1].FirstName, lastName = Users[1].LastName,
                      age = Users[1].Age, role = Users[1].Role }),
            templateService.RenderSsrComponentAsync("my-component.liquid",
                new { firstName = Users[2].FirstName, lastName = Users[2].LastName,
                      age = Users[2].Age, role = Users[2].Role }),
        };
        var liquidResults = await Task.WhenAll(liquidTasks);
        swLiquid.Stop();

        var statLiquid1Dsd  = liquidResults[0]; // avg age
        var statLiquid3Dsd  = liquidResults[1]; // 30+
        var userLiquidDsds  = liquidResults[2..];

        // ── Generated SSR ───────────────────────────────────────────────────────
        // Renders stat-cards #0 (members) and #2 (roles) plus user-cards [3-5]
        // via the emitted C# renderer — synchronous, no template engine.
        var swGen = Stopwatch.StartNew();

        var statGen0Dsd = StatCardTemplate.Render(memberCount, "Members",    "active this sprint");
        var statGen2Dsd = StatCardTemplate.Render(uniqueRoles, "Roles",      "no overlap"        );
        var userGenDsds = Users[3..6]
            .Select(u => MyComponentTemplate.Render(u.FirstName, u.LastName, u.Role, u.Age.ToString()))
            .ToArray();

        swGen.Stop();

        // ── HTML assembly ───────────────────────────────────────────────────────

        // Stat cards: alternate strategies — Generated / Liquid / Generated / CSR
        var statCards = new StringBuilder();
        statCards.AppendLine(
            $"""<div class="card-wrap" data-render="SSR · Generated"><stat-card ssr count="{memberCount}" label="Members" sub="active this sprint">{statGen0Dsd}</stat-card></div>""");
        statCards.AppendLine(
            $"""<div class="card-wrap" data-render="SSR · Liquid"><stat-card ssr count="{avgAge}" label="Avg Age" sub="years">{statLiquid1Dsd}</stat-card></div>""");
        statCards.AppendLine(
            $"""<div class="card-wrap" data-render="SSR · Generated"><stat-card ssr count="{uniqueRoles}" label="Roles" sub="no overlap">{statGen2Dsd}</stat-card></div>""");
        statCards.AppendLine(
            $"""<div class="card-wrap" data-render="CSR"><stat-card count="{seniorCount}" label="30+" sub="senior members"></stat-card></div>""");

        // User cards: first 3 SSR-Liquid, next 3 SSR-Generated, last 2 CSR
        var mixedCards = new StringBuilder();

        for (var i = 0; i < 3; i++)
        {
            var u = Users[i];
            mixedCards.AppendLine(
                $"""
                <div class="card-wrap" data-render="SSR · Liquid">
                  <my-component ssr firstname="{u.FirstName}" lastname="{u.LastName}" age="{u.Age}" role="{u.Role}">
                    {userLiquidDsds[i]}
                  </my-component>
                </div>
                """);
        }

        for (var i = 0; i < 3; i++)
        {
            var u = Users[3 + i];
            mixedCards.AppendLine(
                $"""
                <div class="card-wrap" data-render="SSR · Generated">
                  <my-component ssr firstname="{u.FirstName}" lastname="{u.LastName}" age="{u.Age}" role="{u.Role}">
                    {userGenDsds[i]}
                  </my-component>
                </div>
                """);
        }

        for (var i = 6; i < Users.Length; i++)
        {
            var u = Users[i];
            mixedCards.AppendLine(
                $"""<div class="card-wrap" data-render="CSR"><my-component firstname="{u.FirstName}" lastname="{u.LastName}" age="{u.Age}" role="{u.Role}"></my-component></div>""");
        }

        var html = await templateService.RenderViewAsync("index.html", new Dictionary<string, string>
        {
            { "STAT_CARDS",   statCards.ToString()                     },
            { "MIXED_CARDS",  mixedCards.ToString()                    },
            { "SSR_TIME_MS",  swLiquid.ElapsedMilliseconds.ToString()  },
            { "GEN_TIME_MS",  swGen.ElapsedMilliseconds.ToString()     },
        });

        context.Response.ContentType = "text/html; charset=utf-8";
        await context.Response.WriteAsync(html);
    }
}
