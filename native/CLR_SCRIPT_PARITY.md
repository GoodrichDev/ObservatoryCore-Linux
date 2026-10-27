# Explorer CLR script parity

Original Explorer criteria run in a separate Linux .NET 8.0.33 process with real
NLua 1.7.3 and KeraLua 1.4.1. The linked original `CustomCriteriaManager` supplies
annotation parsing, helpers, globals, reload/error behavior and notifications.
The complete source-built `ObservatoryFramework.dll` supplies real DTOs and
methods. Core, its UI and all four requested installed-plugin workers remain
native Rust; no Wine or original desktop application is launched.

## Independent evidence

- [Framework API metadata](fixtures/clr-criteria-framework-api.json): all 448
  exported public types match the installed reference, including declared member
  signatures, type flags, base types/interfaces, constants, setter/getter
  visibility, parameter names/defaults and non-compiler attributes. No types or
  checked members are missing, added or changed. This is API metadata evidence,
  not a claim that recompilation produces identical DLL bytes.
- [Original oracle](fixtures/clr-criteria-oracle.json): 27 synthetic scenarios,
  89 responses, captured using the installed Framework DLL and unchanged
  decompiled installed criteria manager/settings/exception on Linux .NET 8.0.33.
  `DOTNET_ReadyToRun=0` disables the DLL's Windows native image. Original desktop
  and plugin entry points and the user's custom scripts are not executed.
- Six [integration tests](observatory-plugins/tests/clr_oracle.rs) compare the
  actual JSONL host and full Rust Criteria pipeline against those responses.
  They also check stream isolation, startup notifications and managed exit
  reported once without replaying script effects.
- Three [Explorer state tests](observatory-plugins/tests/clr_explorer_state/mod.rs)
  verify mutation feedback under original dictionary keys, signal mutations
  reaching native default criteria, and real idle timer mutation/callbacks.
  Two [host smoke scripts](observatory-clr-criteria/tests) additionally check
  canonical native echoes, CLR identity, separate `Parent` projection,
  dirty-only updates and pruning while Lua-held references remain valid.

Coverage includes CLR assembly imports, constructors,
properties, reflection, collections, delegates/events, ref/out parameters,
actual `DateTime`, enum, `JsonElement`, `PluginMessage.Clone`, reputation and
journal utility methods, non-finite floating-point values, and source path/read-all
transition quirks. Finite comparisons establish these cases rather than every
possible program or external assembly.

The installed reference identity is
`ObservatoryFramework, Version=1.10.26274.2239, Culture=neutral, PublicKeyToken=null`;
its SHA-256 is
`cdca8ae5ff361c2ab208da635814c5a6c97c58dc101b4e0a99c399564bfda1e1`.
The build records all 404 linked original source inputs in
`source-provenance.json`; their aggregate SHA-256 is
`e0419a8e44c12a2de0c5f6e2ba1add23905e61e4b83aa664cd1ac0695992a416`.
The oracle records its independent installed-source/input/driver hashes.

## Runtime and verification

Packages include the entire self-contained runtime, 193 files and about 75 MB/72 MiB unpacked, under
`lib/observatory-core/clr`. Core discovers it relative to its executable,
including install prefixes containing spaces. No system .NET install is needed.
For a development build, set `OBSERVATORY_CLR_CRITERIA_HOST` to its apphost.
The verified local publish is
`/home/loic/.cache/observatory-clr-criteria-net8/observatory-clr-criteria`.
Its frozen content-manifest SHA-256 is
`cd8691af4e235d1944b0019851d77459c0bc56f682351f42067fee611f9ce755`;
the apphost SHA-256 is
`d5a1c44594cff2e9081b59cc99daad88df85038581091c44ea60adcd150717b5`.
The integrated source passed 396 workspace tests with zero failures and 12
explicit/manual checks ignored; all-target workspace clippy passed. Current Core
and native-worker hashes are recorded in [VALIDATION.md](VALIDATION.md).

Run from the repository root on Linux with a .NET SDK available for building:

```sh
runtime_root="$PWD/target/clr-criteria"
bash native/observatory-clr-criteria/build.sh "$runtime_root"
python3 native/observatory-clr-criteria/tests/smoke.py "$runtime_root/observatory-clr-criteria"
python3 native/observatory-clr-criteria/tests/mutation_roundtrip.py "$runtime_root/observatory-clr-criteria"
OBSERVATORY_CLR_CRITERIA_HOST="$runtime_root/observatory-clr-criteria" cargo test -p observatory-plugins --test clr_oracle --locked
OBSERVATORY_CLR_CRITERIA_HOST="$runtime_root/observatory-clr-criteria" cargo test -p observatory-plugins --lib clr_state_tests --locked
```

`DOTNET_COMMAND` selects an SDK outside `PATH`. NuGet dependencies use the
committed lockfile. [Runtime/protocol details](observatory-clr-criteria/README.md)
describe typed snapshot echoes, exact `JournalBase.Json`, original history keys,
scan/signal/Parent mutation feedback, idle callback polling, 64 MiB frames and
the default ten-second request deadline. A timed-out or exited child is reported
once and is not automatically restarted to replay arbitrary side effects.
Trusted criteria retain CLR filesystem and assembly access; native-extension Lua
limits are a separate API.

Compatible managed assemblies load through real NLua. Windows-only libraries
retain normal Linux .NET limitations, and loading a criteria helper assembly
does not implement the original Windows EOP/WinForms desktop plugin ABI.
Private fullscreen/audio checks use synthetic clients and a PulseAudio null sink;
[their receipt](fixtures/private-fullscreen-audio-linux-qa.json) does not claim
actual Elite gameplay, user-screen access or physical audio listening. A
Linux/Proton game installation was unavailable for that acceptance work.

The startup-failure regression also executes a real CLR initializer that writes a synthetic receipt and exits. Initial failure, edited failure and a new failing path are attempted once each; subsequent scans, Read All and reset do not replay those globals. Repair permits recovery. The full CLR integration suite has six tests.
