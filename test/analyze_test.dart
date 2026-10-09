import 'dart:math';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:voice_flutter/voice_flutter.dart';

// Needs the Rust library built first: `cargo build` in rust/.
const _lib = 'rust/target/debug/libvoice_flutter.so';

Float32List _sine(double hz, double secs, int sr) => Float32List.fromList([
  for (var i = 0; i < (secs * sr).round(); i++)
    0.3 * sin(2 * pi * hz * i / sr),
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
}
