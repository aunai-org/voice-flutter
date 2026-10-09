import 'dart:math';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:voice_flutter/voice_flutter.dart';

Future<void> main() async {
  await VoiceAnalyzer.init();
  runApp(const ExampleApp());
}

/// Analyses a generated 200 Hz tone so the bridge can be checked on a device
/// without a microphone. The record-and-analyse screen comes in F2.
class ExampleApp extends StatelessWidget {
  const ExampleApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('voice_flutter')),
        body: Center(
          child: FutureBuilder<VoiceReport>(
            future: VoiceAnalyzer.analyzeSamples(_tone(), sampleRate: 16000),
            builder: (context, snap) {
              if (snap.hasError) return Text('Error: ${snap.error}');
              final r = snap.data;
              if (r == null) return const CircularProgressIndicator();
              return Text(
                'duration ${r.durationS.toStringAsFixed(2)} s\n'
                'median f0 ${r.f0MedianHz?.toStringAsFixed(1)} Hz\n'
                'loudness ${r.lufs.toStringAsFixed(1)} LUFS',
                key: const Key('report'),
              );
            },
          ),
        ),
      ),
    );
  }
}

Float32List _tone() => Float32List.fromList([
  for (var i = 0; i < 32000; i++) 0.3 * sin(2 * pi * 200 * i / 16000),
]);
