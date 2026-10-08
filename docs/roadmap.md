# voice-flutter: Roadmap

Rough one-developer estimates, not commitments. Status: draft v0.1 (2026-10-06).

| Milestone | Scope | Rough effort |
|---|---|---|
| **F0 Spike** | FRB plugin with stub function running on Windows desktop and a real Android device (iOS later, needs a Mac); versions recorded | 2-3 days |
| **F1 Batch bridge + live basics** | Real `analyze` calls into voice-core, basic frame stream (level, speech/silence), Dart facade; CI for Windows and Android (iOS later) | 1 week (after voice-core M1) |
| **F2 Example + benchmark app** | Record, analyze, show metrics, timing screen; device matrix numbers | 1 week |
| **F3 Streaming** | Frame stream and `finish()`; jank profiling | 1 week (after voice-core M4) |
| **0.1.0 release** | pub.dev publish, docs, pinned to voice-core 0.1.0; used by mimic | 3-4 days |
| **F4 Web** | WASM build, bundle size and latency check, example on Flutter web | 1-2 weeks, after mobile is stable |
| **F5 Prebuilt binaries** | Release artifacts per platform so apps do not need Rust installed | 1 week, if requested |

## Gates
- Do not build the real bridge until the stub runs on both a real Android and iOS device.
- Do not publish 0.1.0 until timings on the low-end device meet the spec target or the gap is documented.

## Relationship to other repos
Tracks voice-core milestones (M1 for F1, M4 for F3). mimic depends on F1 for its first measuring feature and on F3 for live feedback.
