import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:voice_flutter/voice_flutter.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await VoiceAnalyzer.init());
  test('analyzes a tone through the native library', () async {
    final r = await VoiceAnalyzer.analyzeSamples(
      Float32List.fromList(List.filled(32000, 0.0)),
      sampleRate: 16000,
    );
    expect(r.durationS, closeTo(2.0, 1e-6));
  });
}
