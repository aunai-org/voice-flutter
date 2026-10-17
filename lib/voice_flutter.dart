/// Voice measurements (pitch, loudness, pace, pauses) for Flutter apps, backed
/// by the Rust library voice-core. All analysis runs on the device.
library;

import 'dart:typed_data';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart'
    show ExternalLibrary;

import 'src/rust/api/analysis.dart' as bridge;
import 'src/rust/api/measures.dart' as measures;
import 'src/rust/api/stream.dart' as live;
import 'src/rust/frb_generated.dart';

export 'src/rust/api/analysis.dart' show VoiceReport, VoiceWarning;
export 'src/rust/api/measures.dart'
    show
        EmphasisBand,
        EmphasisReport,
        Hesitation,
        HesitationKind,
        LadderStep,
        Phonation,
        TypicalRange,
        WindowStats,
        WordEmphasis;
export 'src/rust/api/stream.dart' show FrameInfo, StreamAnalyzer;

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

  /// The longest voiced stretch of mono [samples] (maximum phonation time), or
  /// null when no stretch has at least 10 voiced frames.
  static Future<measures.Phonation?> maxPhonation(
    Float32List samples, {
    required int sampleRate,
  }) => _guard(
    () => measures.maxPhonation(samples: samples, sampleRate: sampleRate),
  );

  /// Speech level in each of the [steps], given as `(start, end)` windows in
  /// seconds. The baseline is the first step with speech unless
  /// [baselineDbfs] is given.
  static Future<List<measures.LadderStep>> levelLadder(
    Float32List samples, {
    required int sampleRate,
    required List<(double, double)> steps,
    double? baselineDbfs,
  }) => _guard(
    () => measures.levelLadder(
      samples: samples,
      sampleRate: sampleRate,
      steps: steps,
      baselineDbfs: baselineDbfs,
    ),
  );

  /// Level and pitch statistics of each of the [windows], given as
  /// `(start, end)` in seconds (for example word timestamps), in the order
  /// given. Pitch disagrees with Praat by an octave on about 1 window in 8, so
  /// treat it as a guide.
  static Future<List<measures.WindowStats>> windowStats(
    Float32List samples, {
    required int sampleRate,
    required List<(double, double)> windows,
  }) => _guard(
    () => measures.windowStats(
      samples: samples,
      sampleRate: sampleRate,
      windows: windows,
    ),
  );

  /// How much each of the [words], given as `(start, end)` in seconds, stood
  /// out in level and pitch against the rest of the recording, and whether a
  /// pause sat next to it, with the recording's typical range. Descriptive
  /// only: it says how far a word sat from the rest of the take, not whether a
  /// listener heard emphasis. Pitch disagrees with Praat by an octave on about
  /// 1 window in 8.
  static Future<measures.EmphasisReport> emphasis(
    Float32List samples, {
    required int sampleRate,
    required List<(double, double)> words,
  }) => _guard(
    () => measures.emphasis(
      samples: samples,
      sampleRate: sampleRate,
      words: words,
    ),
  );

  /// Pauses (0.5 s or longer) and held sounds (a steady voiced stretch of
  /// 0.4 s or longer) in time order. Pauses match Praat; held sounds are only
  /// checked on fluent speech so far, so do not present them to people as
  /// fact yet.
  static Future<List<measures.Hesitation>> hesitations(
    Float32List samples, {
    required int sampleRate,
  }) => _guard(
    () => measures.hesitations(samples: samples, sampleRate: sampleRate),
  );

  static Future<T> _guard<T>(Future<T> Function() call) async {
    try {
      return await call();
    } on String catch (message) {
      throw VoiceAnalysisException(message);
    }
  }

  /// Starts a live analysis of audio that arrives in chunks. Call `push` with
  /// each chunk (mono floats in -1..1) to get one `FrameInfo` per 10 ms (level,
  /// speech or silence, pitch, syllable nuclei, a 3 s pace), about 0.17 s behind
  /// the audio, and `finish` at the end for the last frames.
  ///
  /// [referenceLevelDbfs] is the loudest level of an earlier recording, if
  /// known, so the first words are judged against a realistic speech level.
  static live.StreamAnalyzer startStream({
    required int sampleRate,
    double? referenceLevelDbfs,
  }) {
    try {
      return live.StreamAnalyzer(
        sampleRate: sampleRate,
        referenceLevelDbfs: referenceLevelDbfs,
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
