# voice-flutter: Specification

Repo: github.com/aunai-org/voice-flutter · License: MIT · Status: draft v0.1 (2026-10-06)

## 1. Purpose
Make [voice-core](../voice-core/spec.md) usable from Flutter apps (mimic first) on Android and iOS, with web later. It contains no signal-processing logic; it only exposes voice-core to Dart, handles recording and threading, and ships prebuilt/automatically built native libraries.

## 2. Decisions this follows
- Bridge: `flutter_rust_bridge` (FRB). UniFFI was rejected because it has no official Dart support.
- Everything measured on-device, no network.
- v1 analyzes a finished recording, but the streaming analyzer's basic frame output (level, speech/silence) ships early for the live meter; pitch frames and the full streaming report come later.
- Web (WASM) is a later milestone, not v1.

## 3. Repo layout
- `rust/` crate `voice_flutter`: depends on `voice-core`, defines the bridged functions and plain data types.
- `lib/`: Dart package `voice_flutter` (generated bindings + a small hand-written facade).
- `example/`: demo app (record, analyze, show numbers) that doubles as the on-device benchmark app.
- `android/`, `ios/`: plugin/build glue so apps just add the dependency.

## 4. Dart API (facade)
```dart
final report = await VoiceAnalyzer.analyzeSamples(samples, sampleRate: 16000);
final report = await VoiceAnalyzer.analyzeFile(path);       // WAV

final session = await VoiceAnalyzer.startStream(sampleRate: 16000);
session.frames.listen((f) { /* pitch, level, speaking? */ });
session.push(chunkOfFloat32);
final report = await session.finish();
```
- `VoiceReport`, `FrameFeatures`, `Config` mirror voice-core's types as Dart classes (generated).
- Samples are `Float32List`. No copying beyond what FRB requires; large buffers pass as typed data.
- Errors map to a typed Dart exception with the voice-core error and warnings preserved.

Dart API additions for mimic: `VoiceAnalyzer.windowStats(report, startS, endS)` (emphasis coach), `report.maxPhonationTime`, `report.levelLadder`, `report.hesitations`; and the stream exposes level and speech/silence frames from the first release because the Week 1 recording screen needs a live meter.

## 5. Recording
Out of scope for the Rust side. The example app uses the Flutter `record` package to capture mono 16-bit or float PCM at 16 kHz (or native rate; voice-core resamples). A thin helper converts to `Float32List`. Microphone permission handling stays in the host app.

## 6. Threading and performance
- Analysis runs off the UI thread (FRB worker pool / Dart isolate). The UI must never block.
- Target: 2 minutes of audio analyzed in under 1 s on a mid-range phone (inherited from voice-core).
- Streaming: `push` is non-blocking; frame events arrive on a Dart stream.

## 7. Platform support
| Platform | v1 | Notes |
|---|---|---|
| Android arm64 (and armv7/x86_64 if cheap) | yes | built with `cargo-ndk`, packaged as `.so` |
| iOS arm64 + simulator | yes | static lib / xcframework |
| Windows desktop | yes (first-class) | mimic's first test target; build the Rust DLL and load it from the Flutter Windows runner |
| macOS/Linux desktop | best effort | development and tests |
| Web | later | WASM via FRB web support or direct `wasm-bindgen` from voice-core |

## 8. Distribution
Pub package consuming the Rust source via FRB's build integration, so app developers need the Rust toolchain; evaluate prebuilt binaries per release to remove that requirement. Version in lockstep with voice-core (`voice-flutter 0.x` pins `voice-core 0.x`).

## 9. Non-goals
Recording UI, speech-to-text, model downloads, subscription gating. Those belong to mimic (or a separate ML package).

## 10. Risks
- FRB version churn: pin versions, keep the bridged surface small.
- iOS/Android toolchain setup is the usual pain point; CI must build both.
- Binary size: keep `voice-core` lean, strip symbols, measure per-ABI size.
- Not verified here: current FRB version and its web support level; confirm before starting.
