# Native diagnostics

Use the same Linux executable for desktop operation, replay and bounded monitoring. `--help` lists the public CLI. `--discover` prints existing journal-directory candidates; `--journal-dir` or `ELITE_JOURNAL_DIR` overrides discovery.

## Keep a diagnostic run separate

`--config` changes the settings file only. Findings, plugin storage and plugins still use `XDG_DATA_HOME`. To keep all diagnostic state together, set both XDG locations explicitly:

```sh
diagnostic_root=$(mktemp -d)
export XDG_CONFIG_HOME="$diagnostic_root/config"
export XDG_DATA_HOME="$diagnostic_root/data"
observatory-core --headless --journal-dir "/path/to/Elite Dangerous" > "$diagnostic_root/replay.json"
```

The source journals are opened for reading. The JSON report contains replay totals, findings, Botanist rows, communications and bounded errors. Repeating replay against the same diagnostic data directory exercises exact finding deduplication; start with a new data directory when comparing independent runs.

The report also includes native BioInsights and Evaluator rows under `plugins`.
Read All rebuilds Hecate's journal/ledger state when Hecate is enabled. To keep a
live diagnostic entirely offline, set `plugins.Hecate.RemoteLookups=false` and
`plugins.BioInsights.GalacticCodexRemote=false` in its temporary settings.

## Watch live activity for a bounded interval

```sh
observatory-core --headless --watch-seconds 30 --journal-dir "/path/to/Elite Dangerous" > "$diagnostic_root/watch.json"
```

Watch mode initializes current-system context through the normal controller, then monitors for the requested duration. The duration starts after context initialization. Its JSON report separates `initial_events` from `live_events`, includes `last_event`, `status_loaded`, `errors` and elapsed seconds, and stops the controller afterward. Valid durations are 1–86,400 seconds. `--watch-seconds` requires `--headless` and cannot be combined with `--export`.

A session with no new game actions may correctly report zero live events. To verify the append path, perform an action that writes a journal event while watch mode runs, then inspect `live_events` and `last_event`. For a controlled filesystem check, use a temporary journal directory, append complete and split JSON records, rotate the journal, and replace `Status.json`; do not inject synthetic records into the game's own files.

## Replay and export

```sh
observatory-core --headless --journal-dir /path/to/journals --export "$diagnostic_root/findings.csv" > "$diagnostic_root/replay.json"
```

The desktop also exports individual worker grids. The generic headless CSV contains persisted findings with quoted delimiters/newlines. File/line diagnostics identify invalid journal records; malformed settings fail with an error rather than silently resetting them.

## Reproducible automated checks

```sh
cargo test --workspace --locked
cargo test -p observatory-plugins --test parity_replay --locked
sh native/scripts/verify-package.sh dist
```

See [PARITY.md](PARITY.md) for source contracts, synthetic fixtures, recorded development-session evidence and the remaining replacement-parity acceptance gates. The CI smoke opens an actual native window under Xvfb; a successful display startup is distinct from a full interactive game-session acceptance test.

## Explorer CLR criteria

Release packages include the complete self-contained Linux .NET 8/NLua runtime
under `lib/observatory-core/clr`. Core discovers it relative to its executable;
a system .NET installation is unnecessary. Development builds can select a host
with `OBSERVATORY_CLR_CRITERIA_HOST=/absolute/path/observatory-clr-criteria`.
`OBSERVATORY_CLR_TIMEOUT_MS` overrides the default ten-second request deadline.
A crash/exit/timeout produces one diagnostic and does not automatically replay
script side effects. Idle engine polls deliver CLR event callbacks and history
mutations without requiring a new journal event.

Use synthetic criteria when checking the runtime independently of a player
profile. These tests create temporary scripts and exercise real CLR methods,
object identities, mutation echoes and timers:

```sh
runtime_root="$PWD/target/clr-criteria"
bash native/observatory-clr-criteria/build.sh "$runtime_root"
python3 native/observatory-clr-criteria/tests/smoke.py "$runtime_root/observatory-clr-criteria"
python3 native/observatory-clr-criteria/tests/mutation_roundtrip.py "$runtime_root/observatory-clr-criteria"
OBSERVATORY_CLR_CRITERIA_HOST="$runtime_root/observatory-clr-criteria" cargo test -p observatory-plugins --test clr_oracle --locked
OBSERVATORY_CLR_CRITERIA_HOST="$runtime_root/observatory-clr-criteria" cargo test -p observatory-plugins --lib clr_state_tests --locked
```

`DOTNET_COMMAND` selects a build SDK outside `PATH`; packaged operation does not
need that SDK. Ship the entire publish directory, including `liblua54.so` and
the .NET runtime. [CLR_SCRIPT_PARITY.md](CLR_SCRIPT_PARITY.md) records the
independent installed-Framework oracle, API metadata comparison and ordinary
Linux .NET limits for Windows-only assemblies.
