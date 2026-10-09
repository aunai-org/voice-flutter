/// Voice measurements (pitch, loudness, pace, pauses) for Flutter apps, backed
/// by the Rust library voice-core. All analysis runs on the device.
library;

import 'dart:typed_data';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart'
    show ExternalLibrary;

import 'src/rust/api/analysis.dart' as bridge;
import 'src/rust/frb_generated.dart';

export 'src/rust/api/analysis.dart' show VoiceReport, VoiceWarning;
export 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart'
    show ExternalLibrary;
export 'src/rust/frb_generated.dart' show RustLib;

/// Thrown when voice-core rejects an input (empty samples, invalid rate).
class VoiceAnalysisException implements Exception {
  VoiceAnalysisException(this.message);
  final String message;
  @override
  String toString() => 'VoiceAnalysisException: $message';
}

/// Entry point for analysing recordings.
class VoiceAnalyzer {
  VoiceAnalyzer._();

  /// Loads the native library. Call once before any analysis (apps normally
  /// call it in `main`). Tests may pass an `ExternalLibrary`.
  static Future<void> init({ExternalLibrary? externalLibrary}) =>
      RustLib.init(externalLibrary: externalLibrary);

  /// Analyses mono [samples] (floats in -1..1) recorded at [sampleRate] Hz.
  /// Runs off the UI thread.
  static Future<bridge.VoiceReport> analyzeSamples(
    Float32List samples, {
    required int sampleRate,
  }) async {
    try {
      return await bridge.analyzeSamples(
        samples: samples,
        sampleRate: sampleRate,
      );
    } on String catch (message) {
      throw VoiceAnalysisException(message);
    }
  }

  /// Converts 16-bit little-endian PCM bytes (as the `record` package emits)
  /// to floats in -1..1.
  static Float32List pcm16ToFloat32(Uint8List bytes) {
    final data = ByteData.sublistView(bytes);
    final n = bytes.lengthInBytes ~/ 2;
    final out = Float32List(n);
    for (var i = 0; i < n; i++) {
      out[i] = data.getInt16(i * 2, Endian.little) / 32768.0;
    }
    return out;
  }
}
