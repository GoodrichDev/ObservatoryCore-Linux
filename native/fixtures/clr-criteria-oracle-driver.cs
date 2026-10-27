// Independent capture driver: original installed Framework DLL and decompiled
// CustomCriteriaManager source only. No Observatory application startup executes.
using System.Globalization;
using System.Text.Json;
using Observatory.Explorer;
using Observatory.Framework.Files.Journal;

CultureInfo.CurrentCulture = CultureInfo.GetCultureInfo("en-US");
if (args is ["--assembly-identity"])
{
    Console.WriteLine(typeof(Scan).Assembly.FullName);
    return;
}
if (args is ["--metadata"])
{
    Console.WriteLine(JsonSerializer.Serialize(new { framework = typeof(Scan).Assembly.FullName, runtime = Environment.Version.ToString() }));
    return;
}
var options = new JsonSerializerOptions { IncludeFields = true };
var scenarios = JsonDocument.Parse(Console.In.ReadToEnd()).RootElement;
var output = new List<object>();
foreach (var scenario in scenarios.EnumerateArray())
{
    var scenarioId = scenario.GetProperty("id").GetString()!;
    var scriptPath = Path.Combine(Path.GetTempPath(), "clr-criteria-oracle-" + scenarioId + ".lua");
    var createdPaths = new List<string> { scriptPath };
    File.WriteAllText(scriptPath, scenario.GetProperty("source").GetString());
    var diagnostics = new List<object>();
    var notifications = new List<object>();
    var settings = new ExplorerSettings { CustomCriteriaFile = scriptPath };
    var manager = new CustomCriteriaManager(settings,
        (error, detail) => diagnostics.Add(new { message = error.Message, detail, exception_type = error.GetType().FullName }),
        (time, title, detail, extended, bodyId) => notifications.Add(new { time, title, detail, extended_detail = extended, body_id = bodyId }));
    var history = new Dictionary<ulong, Dictionary<int, Scan>>();
    var signalHistory = new Dictionary<ulong, Dictionary<int, FSSBodySignals>>();
    var responses = new List<object> { new { ok = true, results = Array.Empty<object>(), notifications = notifications.ToArray(), diagnostics = diagnostics.ToArray() } };
    diagnostics.Clear(); notifications.Clear();
    foreach (var step in scenario.GetProperty("steps").EnumerateArray())
    {
        var results = new List<object>();
        switch (step.GetProperty("op").GetString())
        {
            case "set_read_all": manager.ReadAllMode = step.GetProperty("value").GetBoolean(); break;
            case "wait": Thread.Sleep(step.GetProperty("milliseconds").GetInt32()); break;
            case "set_path":
                var newPath = scriptPath + ".next.lua";
                File.WriteAllText(newPath, step.GetProperty("source").GetString());
                createdPaths.Add(newPath);
                // Explorer and manager share one settings instance. The original
                // settings UI mutates this field without invoking manager.Settings.
                settings.CustomCriteriaFile = newPath;
                break;
            case "rewrite":
                File.WriteAllText(scriptPath, step.GetProperty("source").GetString());
                File.SetLastWriteTimeUtc(scriptPath, File.GetLastWriteTimeUtc(scriptPath).AddSeconds(2));
                break;
            case "process":
                var json = step.GetProperty("event");
                var raw = step.TryGetProperty("raw_json", out var rawJson) ? rawJson.GetString()! : json.GetRawText();
                var typeName = json.GetProperty("event").GetString()!;
                var type = typeof(JournalBase).Assembly.GetType("Observatory.Framework.Files.Journal." + typeName)!;
                var journal = (JournalBase)JsonSerializer.Deserialize(raw, type, options)!;
                journal.Json = raw;
                ulong address = json.TryGetProperty("SystemAddress", out var sa) ? sa.GetUInt64() : 0;
                if (!history.ContainsKey(address)) history[address] = new();
                if (!signalHistory.ContainsKey(address)) signalHistory[address] = new();
                if (step.TryGetProperty("system", out var system))
                    foreach (var bodyJson in system.EnumerateArray())
                    {
                        var body = bodyJson.Deserialize<Scan>(options)!;
                        if (!history.ContainsKey(body.SystemAddress)) history[body.SystemAddress] = new();
                        if (!history[body.SystemAddress].ContainsKey(body.BodyID))
                        {
                            body.Json = bodyJson.GetRawText();
                            history[body.SystemAddress][body.BodyID] = body;
                        }
                    }
                if (journal is FSSBodySignals signal && !signalHistory[address].ContainsKey(signal.BodyID)) signalHistory[address][signal.BodyID] = signal;
                switch (journal)
                {
                    case Scan scan:
                        history[address][scan.BodyID] = scan;
                        results.AddRange(manager.CheckInterest(scan, history, signalHistory, settings).Select(row => new { title = row.Item1, detail = row.Item2, body = scan.BodyName }));
                        break;
                    case FSDJump jump: manager.CustomJump(jump, history); break;
                    case FSSDiscoveryScan discovery: manager.CustomDiscovery(discovery, history); break;
                    case SAASignalsFound bodySignals: manager.CustomSignals(bodySignals, history); break;
                    case FSSAllBodiesFound allBodies: manager.CustomAllBodies(allBodies, history); break;
                }
                break;
        }
        responses.Add(new { ok = true, results, notifications = notifications.ToArray(), diagnostics = diagnostics.ToArray() });
        diagnostics.Clear(); notifications.Clear();
    }
    output.Add(new { id = scenarioId, responses });
    foreach (var path in createdPaths) File.Delete(path);
}
Console.WriteLine(JsonSerializer.Serialize(output));
