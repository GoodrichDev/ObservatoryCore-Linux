# Core native notification/audio source audit

Reference: installed Core 1.5.2, read offline from `~/.cache/observatory-readable/core`.
No original assembly, speech synthesizer, game process or notification window was
started to obtain these source captures.

| Source | SHA-256 |
| --- | --- |
| Observatory.UI/NotificationForm.cs | 1b7cf860665462a3b143317f9fe7bd552232e5e69bee35fbe38ac4649492bd11 |
| Observatory.NativeNotification/NativeVoice.cs | d85ad478577532db57415f8fdcfde1bb278ea6fd9bec667fd47d2071fff1c408 |
| Observatory.Utils/AudioHandler.cs | b12c54ef43d65e2e202b5567d7efbbb8cf213f9a2bdf6ab8020b0e4cc4772638 |

## Popup behavior

The original `PluginCore.SendNotification`, `UpdateNotification` and
`CancelNotification` channel predicates are now captured independently in
`fixtures/core-notification-routing-oracle.json`: all 768 action/rendering/batch/
global-toggle/override combinations match. Core notification-event delivery,
notifier batch acceptance, native popup/audio output, update events and cancel
events are separate stages. Updates preserve the source behavior that visual
updates ignore the global popup toggle and overrides, while audio updates still
require the global voice toggle. Cancel events work during a batch read.

The actual Engine dispatch uses those predicates for worker findings, explicit
Core API notifications and Botanist sampler events. Runtime tests verify shared
IDs, source Explorer SSML, batch delivery to opted-in notifiers, distinct update/
cancel callbacks, audio-only updates leaving popups unchanged, and sampler
replacement. The optional findings database never deduplicates UI rows or gates
notification delivery. Direct Core Settings popup/audio tests retain their source
bypass of global notification toggles.

The native manager follows NativePopup.InvokeNativeNotification/NotifyWindow_Closed:
new default-position windows shift older windows by their own height; closing a
window shifts only older windows. Explicit percentage positions do not shift.
AdjustOffset reads the **current Core corner**, including after a settings change;
Manager.set_settings is called by the host to preserve that behavior.

`fixtures/capture-core-popup.py` compiles the two original geometry methods against
inert forms/screens. All **480 vectors** compare initial placement, two stack
increments and a close after changing the corner: 8 corners, 4 monitors (including
negative offsets and odd sizes), 3 popup sizes and 5 percentage/default positions.
The test caught and fixed .NET ties-to-even rounding, integer center coordinates,
scaled whole-pixel dimensions and the current-corner dependency. Source odd-center
monitor arithmetic is preserved, including its use of Right/2 and Bottom/2.

NotificationForm constructor/rendering were reviewed against native_popup.rs:
355x145 base client size; title (5,5,338,35), body (12,40,320,85), 18/14 point
fonts scaled by both controls, body vertical adjustment, unscaled six-pixel border,
ARGB color, transparency or (25,30,30) background, configured screen/corner and
source default 8000ms timeout. Update replaces title/detail without resetting
position, style or expiration; missing GUID is rejected. Timeout 0 is persistent,
-1 uses Core.NativeNotifyTimeout, and cancel/update target only the matching ID.
Source non-activation/click-through/topmost flags map to the native passive-window
backend; actual Linux/Wayland/X11 focus probes are host-owned verification.
Invalid saved monitor indices fall back safely instead of reproducing the source
out-of-range indexing exception.

## Audio behavior

The six bundled chime WAV files are **byte-identical** to the installed Core's
resx resources. `capture-core-chimes.py`, `core-chime-assets.json` and the native
SHA-256 regression test preserve receipts and the original 0..5 selection.

`capture-core-ssml.py` compiles original AddVoiceToSsml without loading a speech
engine. Ten original XML/error vectors compare namespace-aware semantic trees:
only the first direct speak/voice name attribute is replaced; nested or unnamed
voice elements are retained; malformed XML is rejected. The native plan retains
separate title/detail SSML, configured native voice, title suppression while a
matching trimmed/case-folded title is queued or playing, voice volume, rate and
current playback device/volume. Updates can play audio and preserve popup TTL.
The optional DBus backend now preserves the remaining TTL on updates too.

Native Linux synthesis uses installed eSpeak voices and Pulse/PipeWire playback;
SAPI voice names and its numeric speaking-rate implementation cannot be identical
to Linux engine voices. The existing explicit VoiceRate-to-words-per-minute
adapter remains documented in settings import. Voice/AudioDevice changes retain
both the source property names and native backend identifiers. A silent actual
eSpeak SSML synthesis produced a valid mono 22050Hz, 16-bit WAV with 34081 frames;
no speaker playback was performed for this check.

Notifications.play_file implements original Core.PlayAudioFile/AudioHandler
queued playback, Instant concurrent playback, actual completion, DeleteAfterPlay,
and current device/volume. Instant jobs and the main queue are cancelled/reaped
on shutdown. There is no invented 30-second audio truncation. Host options carry
request_id/RequestID and plugin/Plugin; completion reports actual success/error,
cancellation or a source-compatible skipped missing-file operation.

An injected playback I/O seam tests queue ordering, concurrent Instant jobs,
current settings at dequeue, no completion before playback returns, successful
delete, failed playback preserving the file, and correlated error completions.
Production uses paplay through the same seam. Other tests cover source SSML plans,
chime hashes and child cancellation/reaping. Five audio tests and two popup tests
pass. Desktop clippy for all targets passes with warnings denied.

## Routing and controls owned by the host

Source PluginCore.SendNotification: bit4 sends plugin events even during batch;
other native effects require !batch, global toggles and absence of the matching
notifier override. Source UpdateNotification: all effects require !batch; bit4
sends Notification, bit1 updates existing native popup regardless of global
popup toggle/override, bit2 plus VoiceNotify plays audio, then UpdateNotificationEvent
fires independently. Cancel always targets native popup and emits cancellation.
The host owns these dispatch predicates and their integration tests.

Source CoreSettings.TestButton_Click and VoiceTestButton_Click bypass the global
Enabled flags by invoking NativePopup and NativeVoice directly. The host handles
that bypass; source routing-test checkboxes still use normal PluginCore dispatch.
Core notification control names, bounds and source scale/rate/timeout ranges were
reviewed. Native font/voice/device inventories depend on actual Linux resources;
installed Windows voices are not presented as working Linux voices.

This audit establishes source comparisons and finite tests for these modules;
it does not substitute for host live focus, monitor/DPI/font, audio hardware or
full application routing verification.
