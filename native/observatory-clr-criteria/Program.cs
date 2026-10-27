using System.Reflection;
using System.Collections.Immutable;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Text.Json.Serialization;
using Microsoft.Win32.SafeHandles;
using Observatory.Explorer;
using Observatory.Framework.Files.Journal;
using Observatory.Framework.Files.ParameterTypes;
using Observatory.Utils;

// This process is only the optional Explorer script runtime. Desktop, ingestion,
// history, built-in workers, persistence and presentation remain native Rust.
// Use the original manager and Framework objects instead of an imitation of CLR.
internal sealed class CriteriaHost
{
    private readonly object gate = new();
    private readonly List<object> notifications = new();
    private readonly List<object> diagnostics = new();
    private ExplorerSettings settings;
    private CustomCriteriaManager manager;
    private bool readAll;
    private sealed class HistoryEntry
    {
        public JournalBase Value;
        public string InputSource;
        public string InputState;
        public string SentState;
    }
    private sealed record Snapshot(JsonElement Event, JsonElement Parent, string RawJson, string State);
    private readonly Dictionary<(string Kind, ulong System, int Body), HistoryEntry> history = new();
    private static readonly JsonSerializerOptions WireJson = new()
    {
        // CLR scripts may assign non-finite floats. JSON string literals retain
        // those values across the process boundary and are accepted on echoes.
        NumberHandling = JsonNumberHandling.AllowNamedFloatingPointLiterals,
    };
    private static readonly Dictionary<string, Type> JournalTypes = JournalReader.PopulateEventClasses();
    private static readonly MethodInfo RefreshMethod = typeof(CustomCriteriaManager)
        .GetMethod("MaybeRefreshCriteria", BindingFlags.Instance | BindingFlags.NonPublic);

    private void Log(Exception error, string detail)
    {
        lock (gate)
            diagnostics.Add(new { message = error.Message, detail, exception_type = error.GetType().FullName });
    }

    private void Notify(string time, string title, string detail, string extended, int? bodyId)
    {
        lock (gate)
            notifications.Add(new { time, title, detail, extended_detail = extended, body_id = bodyId });
    }

    private static JsonElement Data(JsonElement value) =>
        value.TryGetProperty("event", out var property) && property.ValueKind == JsonValueKind.Object ? property : value;

    private static string Raw(JsonElement value) =>
        value.TryGetProperty("raw_json", out var raw) && raw.ValueKind == JsonValueKind.String ? raw.GetString() : null;

    private static JournalBase Journal(JsonElement value, string raw = null, Type forcedType = null)
    {
        // A history entry may be wrapped to preserve the exact original journal
        // bytes exposed through JournalBase.Json, including whitespace/ordering.
        if (value.TryGetProperty("event", out var property) && property.ValueKind == JsonValueKind.Object)
        {
            raw ??= Raw(value);
            value = property;
        }
        string eventName = value.TryGetProperty("event", out var name) && name.ValueKind == JsonValueKind.String ? name.GetString() : "Scan";
        if (!JournalTypes.TryGetValue(eventName ?? "", out var type))
            type = typeof(JournalBase);
        // A synthetic barycentre Scan has properties different from its retained
        // original Json (and Event is null). Deserialize the typed snapshot first,
        // then assign Json once, as in the original ConvertBarycentre path.
        var source = value.GetRawText();
        var journal = (JournalBase)JsonSerializer.Deserialize(source, forcedType ?? type, WireJson)
            ?? throw new JsonException("Unable to deserialize journal snapshot.");
        journal.Json = raw ?? source;
        return journal;
    }

    private static Snapshot Capture(JournalBase journal)
    {
        var data = JsonSerializer.SerializeToElement(journal, journal.GetType(), WireJson);
        // Parent is a separately mutable, JsonIgnored projection. Keep it
        // distinct from Parents, as the original DTO does.
        var parents = JsonSerializer.SerializeToElement(journal is Scan scan
            ? scan.Parent?.Select(item => new { parent_type = item.ParentType.ToString(), body = item.Body }).ToArray()
            : null, WireJson);
        var state = JsonSerializer.Serialize(new { @event = data, parent = parents, raw_json = journal.Json }, WireJson);
        return new(data, parents, journal.Json, state);
    }

    private static bool SameState(string left, string right) =>
        left == right || (left != null && right != null && JsonNode.DeepEquals(JsonNode.Parse(left), JsonNode.Parse(right)));

    private static void ApplyParentProjection(Scan scan, JsonElement input)
    {
        if (!input.TryGetProperty("parent", out var projection))
            return;
        ImmutableList<(ParentType ParentType, int Body)> parents = null;
        if (projection.ValueKind == JsonValueKind.Array)
            parents = projection.EnumerateArray().Select(item =>
                (Enum.Parse<ParentType>(item.GetProperty("parent_type").GetString()), item.GetProperty("body").GetInt32())).ToImmutableList();
        typeof(Scan).GetProperty(nameof(Scan.Parent)).SetValue(scan, parents);
    }

    private (ulong System, int Body, HistoryEntry Entry, bool Added) ImportHistory(JsonElement input, string kind)
    {
        var data = Data(input);
        ulong address = input.TryGetProperty("system_id", out var systemKey) ? systemKey.GetUInt64()
            : data.TryGetProperty("SystemAddress", out var addressValue) ? addressValue.GetUInt64() : 0;
        int body = input.TryGetProperty("body_id", out var bodyKey) ? bodyKey.GetInt32()
            : data.TryGetProperty("BodyID", out var bodyValue) ? bodyValue.GetInt32() : 0;
        var key = (kind, address, body);
        var source = input.GetRawText();
        if (history.TryGetValue(key, out var old) && old.InputSource == source)
            return (address, body, old, false);
        var journal = Journal(input, Raw(input) ?? old?.Value.Json, kind == "scan" ? typeof(Scan) : typeof(FSSBodySignals));
        if (journal is Scan scan)
            ApplyParentProjection(scan, input);
        var snapshot = Capture(journal);
        // Native echoes can contain CLR defaults, a different JSON member order,
        // or the last emitted mutation while an asynchronous callback has since
        // changed the CLR object again. All preserve the existing CLR identity.
        if (old != null && (SameState(snapshot.State, old.InputState)
            || SameState(snapshot.State, old.SentState) || SameState(snapshot.State, Capture(old.Value).State)))
        {
            old.InputSource = source;
            old.InputState = snapshot.State;
            return (address, body, old, false);
        }
        var entry = new HistoryEntry { Value = journal, InputSource = source, InputState = snapshot.State, SentState = snapshot.State };
        history[key] = entry;
        return (address, body, entry, true);
    }

    private List<object> HistoryUpdates()
    {
        var updates = new List<object>();
        foreach (var (key, entry) in history)
        {
            try
            {
                var snapshot = Capture(entry.Value);
                if (SameState(snapshot.State, entry.SentState))
                    continue;
                var update = new Dictionary<string, object>
                {
                    ["kind"] = key.Kind, ["system_id"] = key.System, ["body_id"] = key.Body,
                    ["event"] = snapshot.Event, ["raw_json"] = snapshot.RawJson,
                };
                if (key.Kind == "scan")
                    update["parent"] = snapshot.Parent;
                updates.Add(update);
                entry.SentState = snapshot.State;
            }
            catch (Exception error)
            {
                // User-managed callbacks may be modifying AdditionalProperties
                // concurrently. Retain the old baseline so the next poll retries.
                Log(error, $"Unable to serialize CLR {key.Kind} history mutation for {key.System}/{key.Body}.");
            }
        }
        return updates;
    }

    private void RetainHistory(JsonElement request)
    {
        var keys = request.GetProperty("keys").EnumerateArray().Select(item =>
            (item.GetProperty("kind").GetString(), item.GetProperty("system_id").GetUInt64(), item.GetProperty("body_id").GetInt32())).ToHashSet();
        var kinds = request.TryGetProperty("kinds", out var requestedKinds)
            ? requestedKinds.EnumerateArray().Select(kind => kind.GetString()).ToHashSet() : null;
        foreach (var key in history.Keys.Where(key => (kinds == null || kinds.Contains(key.Kind)) && !keys.Contains(key)).ToArray())
            history.Remove(key);
    }

    private void RequireManager()
    {
        if (manager == null)
            throw new InvalidOperationException("Send an init request before processing criteria.");
    }

    public object Handle(JsonElement request)
    {
        var results = new List<object>();
        JournalBase processedEvent = null;
        try
        {
            switch (request.GetProperty("op").GetString())
            {
                case "init":
                    history.Clear();
                    settings = new ExplorerSettings { CustomCriteriaFile = request.GetProperty("path").GetString() ?? "" };
                    readAll = request.TryGetProperty("read_all", out var initialMode) && initialMode.GetBoolean();
                    manager = new CustomCriteriaManager(settings, Log, Notify);
                    manager.ReadAllMode = readAll;
                    break;
                case "set_read_all":
                    RequireManager();
                    readAll = request.GetProperty("value").GetBoolean();
                    manager.ReadAllMode = readAll;
                    break;
                case "set_path":
                    RequireManager();
                    // Explorer shares this settings instance with the manager;
                    // it does not assign manager.Settings after construction.
                    // CriteriaFileNeedsRefresh detects the changed path lazily,
                    // including the original read-all transition ordering.
                    settings.CustomCriteriaFile = request.GetProperty("path").GetString() ?? "";
                    break;
                case "reset":
                    RequireManager();
                    // A read-all transition in Explorer force-loads the same file
                    // and preserves the previous eventTime. This uses that exact
                    // reload path without manufacturing a mode transition.
                    RefreshMethod.Invoke(manager, new object[] { true });
                    break;
                case "process":
                    RequireManager();
                    Process(request, results, out processedEvent);
                    break;
                case "clear_history":
                    history.Clear();
                    break;
                case "retain_history":
                    RetainHistory(request);
                    break;
                case "poll":
                    break;
                case "ping":
                    break;
                default:
                    throw new ArgumentException("Unknown criteria operation.");
            }
            return Response(true, results, null, processedEvent);
        }
        catch (Exception error)
        {
            if (error is TargetInvocationException { InnerException: not null } invocation)
                error = invocation.InnerException;
            Log(error, "CLR criteria request failed.");
            return Response(false, results, error.Message, processedEvent);
        }
    }

    private object Response(bool ok, List<object> results, string error, JournalBase journal)
    {
        Snapshot current = null;
        try
        {
            if (journal != null)
                current = Capture(journal);
        }
        catch (Exception exception)
        {
            Log(exception, "Unable to serialize mutated CLR journal event.");
        }
        var updates = HistoryUpdates();
        lock (gate)
        {
            var output = new
            {
                ok, error, protocol = 1, runtime = ".NET", nlua = "1.7.3", results,
                @event = current?.Event, event_parent = current?.Parent, event_raw_json = current?.RawJson,
                history_updates = updates, notifications = notifications.ToArray(), diagnostics = diagnostics.ToArray(),
            };
            notifications.Clear();
            diagnostics.Clear();
            return output;
        }
    }

    private void Process(JsonElement request, List<object> results, out JournalBase journal)
    {
        Type forcedType = null;
        if (request.TryGetProperty("event_type", out var eventType) && eventType.ValueKind == JsonValueKind.String)
            JournalTypes.TryGetValue(eventType.GetString(), out forcedType);
        journal = Journal(request.GetProperty("event"),
            request.TryGetProperty("raw_json", out var raw) && raw.ValueKind == JsonValueKind.String ? raw.GetString() : null, forcedType);
        var scanHistory = new Dictionary<ulong, Dictionary<int, Scan>>();
        var signalHistory = new Dictionary<ulong, Dictionary<int, FSSBodySignals>>();
        var addedSignals = new HashSet<(ulong System, int Body)>();
        if (request.TryGetProperty("system", out var system))
            foreach (var value in system.EnumerateArray())
            {
                // Retained CLR objects can have properties set by Lua even on
                // init-only DTOs. Dictionary keys stay those of the native
                // history, just as the original dictionary's keys stay fixed.
                var (address, body, entry, _) = ImportHistory(value, "scan");
                if (!scanHistory.TryGetValue(address, out var bodies))
                    scanHistory[address] = bodies = new();
                bodies[body] = (Scan)entry.Value;
            }
        if (request.TryGetProperty("signals", out var signals))
            foreach (var value in signals.EnumerateArray())
            {
                // Native history also accepts SAASignalsFound records. Original
                // CheckInterest's container type is FSSBodySignals, whose base
                // SAASignalsFound exposes precisely the same signal properties.
                var (address, body, entry, added) = ImportHistory(value, "signal");
                if (!signalHistory.TryGetValue(address, out var bodies))
                    signalHistory[address] = bodies = new();
                bodies[body] = (FSSBodySignals)entry.Value;
                if (added)
                    addedSignals.Add((address, body));
            }
        if (journal is FSSBodySignals incoming && addedSignals.Contains((incoming.SystemAddress, incoming.BodyID)))
        {
            var key = ("signal", incoming.SystemAddress, incoming.BodyID);
            // RecordSignal adds only the first event. Later duplicate events
            // still reach the handler as distinct objects, while history keeps
            // its first instance and any mutation performed on it by Lua.
            var entry = history[key];
            if (JsonNode.DeepEquals(JsonNode.Parse(Capture(entry.Value).Event.GetRawText()), JsonNode.Parse(Capture(incoming).Event.GetRawText())))
            {
                entry.Value = incoming;
                entry.SentState = Capture(incoming).State;
                signalHistory[incoming.SystemAddress][incoming.BodyID] = incoming;
            }
        }
        switch (journal)
        {
            case Scan scan:
                if (!scanHistory.ContainsKey(scan.SystemAddress))
                    scanHistory[scan.SystemAddress] = new();
                // The original excludes system-address-zero scans from history.
                // If this event is in the native snapshot, share its CLR identity
                // with system[index]. Existing retained snapshots keep their CLR
                // identity across subsequent events and user global references.
                if (scanHistory[scan.SystemAddress].ContainsKey(scan.BodyID))
                {
                    var entry = history[("scan", scan.SystemAddress, scan.BodyID)];
                    entry.Value = scan;
                    entry.SentState = Capture(scan).State;
                    scanHistory[scan.SystemAddress][scan.BodyID] = scan;
                }
                foreach (var (title, detail, systemWide) in manager.CheckInterest(scan, scanHistory, signalHistory, settings))
                    results.Add(new { title, detail, system_wide = systemWide });
                break;
            case FSSDiscoveryScan discovery:
                manager.CustomDiscovery(discovery, scanHistory);
                break;
            case FSSAllBodiesFound allBodies:
                manager.CustomAllBodies(allBodies, scanHistory);
                break;
            case FSDJump jump:
                manager.CustomJump(jump, scanHistory);
                break;
            case SAASignalsFound bodySignals:
                manager.CustomSignals(bodySignals, scanHistory);
                break;
        }
    }
}

internal static class Program
{
    [DllImport("libc", SetLastError = true)]
    private static extern int dup(int descriptor);
    [DllImport("libc", SetLastError = true)]
    private static extern int dup2(int source, int target);

    private static int Main(string[] args)
    {
        if (args is ["--version"])
        {
            Console.WriteLine("observatory-clr-criteria 1; NLua 1.7.3; ObservatoryFramework source DTOs");
            return 0;
        }
        if (args.Length != 0)
        {
            Console.Error.WriteLine("Usage: observatory-clr-criteria [--version] (JSONL on stdin/stdout)");
            return 2;
        }
        // Keep the protocol streams private from Lua/CLR standard stream APIs.
        // User CLR code is trusted code, with the same OS permissions as the app.
        Stream protocolInput;
        Stream protocolOutput;
        if (OperatingSystem.IsLinux() || OperatingSystem.IsMacOS())
        {
            var inputDescriptor = dup(0);
            var outputDescriptor = dup(1);
            using var nullInput = File.OpenRead("/dev/null");
            if (inputDescriptor < 0 || outputDescriptor < 0
                || dup2(nullInput.SafeFileHandle.DangerousGetHandle().ToInt32(), 0) < 0 || dup2(2, 1) < 0)
                throw new IOException("Unable to isolate criteria protocol streams.");
            protocolInput = new FileStream(new SafeFileHandle((IntPtr)inputDescriptor, ownsHandle: true), FileAccess.Read);
            protocolOutput = new FileStream(new SafeFileHandle((IntPtr)outputDescriptor, ownsHandle: true), FileAccess.Write);
        }
        else
        {
            protocolInput = Console.OpenStandardInput();
            protocolOutput = Console.OpenStandardOutput();
        }
        using var input = new StreamReader(protocolInput, Encoding.UTF8);
        using var output = new StreamWriter(protocolOutput, new UTF8Encoding(false)) { AutoFlush = true };
        Console.SetIn(TextReader.Null);
        Console.SetOut(Console.Error);
        var host = new CriteriaHost();
        string line;
        while ((line = input.ReadLine()) != null)
        {
            try
            {
                if (line.Length > 64 * 1024 * 1024)
                    throw new ArgumentException("Criteria request exceeds 64 MiB.");
                using var document = JsonDocument.Parse(line);
                output.WriteLine(JsonSerializer.Serialize(host.Handle(document.RootElement)));
            }
            catch (Exception error)
            {
                output.WriteLine(JsonSerializer.Serialize(new { ok = false, error = error.Message, results = Array.Empty<object>(), notifications = Array.Empty<object>(), diagnostics = Array.Empty<object>() }));
            }
        }
        return 0;
    }
}
