# voice-flutter

Flutter package (via `flutter_rust_bridge`) wrapping [voice-core](https://github.com/aunai-org/voice-core). All measuring runs on the device; there is no network use.

Status: pre-alpha. Planning docs: [docs/spec.md](docs/spec.md), [docs/plan.md](docs/plan.md), [docs/roadmap.md](docs/roadmap.md).

## What works today

| Feature | State |
|---|---|
| `VoiceAnalyzer.analyzeSamples(Float32List, sampleRate:)` returning `VoiceReport` (duration, level, LUFS, median f0, voiced fraction, syllables and pace, pauses, warnings) | works; tested from Dart against the native library on Linux |
| `VoiceAnalyzer.pcm16ToFloat32` (16-bit PCM bytes from `record` to floats) | works |
| Typed errors (`VoiceAnalysisException`) | works |
| Example app showing a report | builds for Windows and Android in CI; not yet run on a device |
| Windows desktop (example app) and Android (debug APK) builds | green in CI |
| `VoiceAnalyzer.startStream(sampleRate:)` returning a `StreamAnalyzer`: `push` audio chunks, get one `FrameInfo` per 10 ms (level, speech/silence, pitch, syllable nucleus, 3 s pace), about 0.17 s behind the audio; `finish` for the last frames | works; tested from Dart on Linux |
| `analyzeFile`, window stats | planned |
| iOS, web | later |

Numbers come from voice-core `analyze` on its main branch (pinned by commit in `rust/Cargo.toml` until voice-core is on crates.io); see that repo for accuracy against Praat.

## Use

```dart
await VoiceAnalyzer.init();                       // once, in main()
final report = await VoiceAnalyzer.analyzeSamples(samples, sampleRate: 16000);
print(report.f0MedianHz);

// Live: push chunks as the microphone delivers them.
final live = VoiceAnalyzer.startStream(sampleRate: 16000);
final frames = await live.push(samples: chunk);   // List<FrameInfo>
final rest = await live.finish();
```

## Develop

```sh
cargo install flutter_rust_bridge_codegen --version 2.13.0 --locked
flutter pub get
flutter_rust_bridge_codegen generate   # after changing rust/src/api; CI fails if generated files are stale
(cd rust && cargo test)
cargo build --manifest-path rust/Cargo.toml && flutter test
```

Versions used: Flutter 3.47.7, Dart 3.13.5, flutter_rust_bridge 2.13.0.

License: MIT.
