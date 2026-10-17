import 'dart:math';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:voice_flutter/voice_flutter.dart';

// Needs the Rust library built first: `cargo build` in rust/.
const _lib = 'rust/target/debug/libvoice_flutter.so';

Float32List _sine(double hz, double secs, int sr) => Float32List.fromList([
  for (var i = 0; i < (secs * sr).round(); i++) 0.3 * sin(2 * pi * hz * i / sr),
]);

void main() {
  setUpAll(() async {
    await VoiceAnalyzer.init(externalLibrary: ExternalLibrary.open(_lib));
  });

  test('a 200 Hz sine reports about 200 Hz and its duration', () async {
    final r = await VoiceAnalyzer.analyzeSamples(
      _sine(200, 2, 16000),
      sampleRate: 16000,
    );
    expect(r.durationS, closeTo(2.0, 1e-6));
    expect(r.f0MedianHz, closeTo(200, 2));
    expect(r.warnings, isEmpty);
  });

  test('silence carries the too-quiet warning', () async {
    final r = await VoiceAnalyzer.analyzeSamples(
      Float32List(32000),
      sampleRate: 16000,
    );
    expect(r.warnings, contains(VoiceWarning.tooQuiet));
  });

  test('empty input throws VoiceAnalysisException', () async {
    expect(
      VoiceAnalyzer.analyzeSamples(Float32List(0), sampleRate: 16000),
      throwsA(isA<VoiceAnalysisException>()),
    );
  });

  test('pcm16ToFloat32 scales to -1..1', () {
    final bytes = Uint8List.fromList([0x00, 0x80, 0xFF, 0x7F, 0x00, 0x00]);
    final f = VoiceAnalyzer.pcm16ToFloat32(bytes);
    expect(f[0], -1.0);
    expect(f[1], closeTo(0.99997, 1e-4));
    expect(f[2], 0.0);
  });

  test('a live stream gives pitch and speech frames chunk by chunk', () async {
    final stream = VoiceAnalyzer.startStream(sampleRate: 16000);
    final samples = _sine(200, 2, 16000);
    final frames = <FrameInfo>[];
    for (var i = 0; i < samples.length; i += 1600) {
      frames.addAll(await stream.push(samples: samples.sublist(i, i + 1600)));
    }
    frames.addAll(await stream.finish());
    expect(frames.length, greaterThan(195));
    final mid = frames[100];
    expect(mid.speech, isTrue);
    expect(mid.f0Hz, closeTo(200, 3));
    expect(await stream.push(samples: Float32List(1600)), isEmpty);
  });

  test('a stream with a zero sample rate throws', () {
    expect(
      () => VoiceAnalyzer.startStream(sampleRate: 0),
      throwsA(isA<VoiceAnalysisException>()),
    );
  });

  test(
    'the report carries jitter, shimmer and HNR for a steady tone',
    () async {
      final r = await VoiceAnalyzer.analyzeSamples(
        _sine(200, 2, 16000),
        sampleRate: 16000,
      );
      expect(r.jitterLocal, isNotNull);
      expect(r.shimmerLocal, isNotNull);
      expect(r.hnrDb, greaterThan(20));
    },
  );

  test('silence has no jitter, shimmer or HNR', () async {
    final r = await VoiceAnalyzer.analyzeSamples(
      Float32List(32000),
      sampleRate: 16000,
    );
    expect(r.jitterLocal, isNull);
    expect(r.shimmerLocal, isNull);
    expect(r.hnrDb, isNull);
  });

  test('maximum phonation of a held tone, none for silence', () async {
    final p = await VoiceAnalyzer.maxPhonation(
      _sine(200, 2, 16000),
      sampleRate: 16000,
    );
    expect(p, isNotNull);
    expect(p!.durationS, greaterThan(1.5));
    expect(p.meanF0Hz, closeTo(200, 3));
    expect(
      await VoiceAnalyzer.maxPhonation(Float32List(32000), sampleRate: 16000),
      isNull,
    );
  });

  test(
    'the level ladder gives a step per window, relative to the first',
    () async {
      final loud = _sine(200, 1, 16000);
      final soft = Float32List.fromList([for (final v in loud) v * 0.5]);
      final steps = await VoiceAnalyzer.levelLadder(
        Float32List.fromList([...loud, ...soft]),
        sampleRate: 16000,
        steps: [(0.0, 1.0), (1.0, 2.0)],
      );
      expect(steps, hasLength(2));
      expect(steps[0].relativeDb, 0.0);
      expect(steps[1].relativeDb, closeTo(-6, 1));
    },
  );

  test('a bad ladder step throws VoiceAnalysisException', () {
    expect(
      VoiceAnalyzer.levelLadder(
        Float32List(1600),
        sampleRate: 16000,
        steps: [(1.0, 0.5)],
      ),
      throwsA(isA<VoiceAnalysisException>()),
    );
  });

  test('window stats come back in the order given', () async {
    final w = await VoiceAnalyzer.windowStats(
      _sine(200, 2, 16000),
      sampleRate: 16000,
      windows: [(0.5, 1.0), (0.0, 0.5)],
    );
    expect(w.map((e) => e.startS), [0.5, 0.0]);
    expect(w[0].meanF0Hz, closeTo(200, 3));
  });

  test(
    'emphasis returns one entry per word and no range for a short take',
    () async {
      final r = await VoiceAnalyzer.emphasis(
        _sine(200, 2, 16000),
        sampleRate: 16000,
        words: [(0.5, 0.9), (1.0, 1.4)],
      );
      expect(r.words.map((e) => e.startS), [0.5, 1.0]);
      expect(r.words[0].pauseAfterS, 0);
      expect(r.typical.levelDb, isNull);
    },
  );

  test('a silent gap is a pause hesitation', () async {
    final tone = _sine(200, 1, 16000);
    final h = await VoiceAnalyzer.hesitations(
      Float32List.fromList([...tone, ...Float32List(16000), ...tone]),
      sampleRate: 16000,
    );
    expect(h.any((e) => e.kind == HesitationKind.pause), isTrue);
  });
}
