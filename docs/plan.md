# voice-flutter: Implementation Plan

Companion to [spec.md](spec.md). Status: draft v0.1 (2026-10-06).

## Preconditions
voice-core has at least a batch `analyze()` (milestone M1) so there is something to bridge. Before that, bridge a stub returning fixed values to prove the toolchain.

## Steps
1. **Toolchain spike (stub).** Create the FRB plugin template, bridge one function (`analyze` returning dummy `VoiceReport`), run it on a real Android phone and an iPhone/simulator. Record the exact FRB, Flutter and Rust versions that work.
2. **CI.** GitHub Actions: Rust fmt/clippy/test, `flutter analyze`/`test`, Android build, iOS build (macOS runner), codegen drift check (fail if generated bindings are stale).
3. **Real bridge.** Depend on `voice-core`; expose `analyzeSamples`, `analyzeFile`, config and report types. Keep bridged types plain and serializable.
4. **Dart facade.** `VoiceAnalyzer` with error mapping, `Float32List` helpers, docs and dartdoc examples.
5. **Example app.** Record with `record`, run analysis, display pitch, loudness, pace, pauses, steadiness and warnings. Add a "benchmark" screen timing 2-minute clips and showing device model.
6. **Streaming.** Bridge `Analyzer` push/finish with a Dart `Stream` of frame features; ensure no UI jank (profile in Flutter DevTools).
7. **Device testing.** Matrix: one low-end and one recent Android, one older and one recent iPhone. Check timings, memory, app size per ABI.
8. **Packaging.** pub.dev metadata, README, license, version pinning to voice-core; decide prebuilt binaries vs build-from-source.
9. **Web spike (later).** Try FRB web target or a separate `wasm-bindgen` package from voice-core; measure bundle size and latency.

## Changes for mimic (v0.2)
- Windows desktop is built and tested alongside Android from the spike onward (the user is on Windows); iOS needs a Mac and comes later.
- Step 6 splits: basic frame stream (level, speech/silence) in F1 for the live meter, pitch frames and the full report stream later.
- Expose `windowStats` and the new report fields as voice-core adds them.

## Definition of done (v0.1)
- Example app analyzes a 2-minute recording on Android and iOS under the performance target without UI jank.
- CI builds both platforms and checks bindings are up to date.
- mimic can add the package and call `analyzeSamples` with no native setup beyond the Rust toolchain.

## Testing
Rust unit tests inherited from voice-core; Dart unit tests with fixture WAVs and golden JSON; integration test on emulator/simulator; manual device benchmark recorded in the README.
