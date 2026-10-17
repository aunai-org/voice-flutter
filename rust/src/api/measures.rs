//! Measures beyond the basic report: maximum phonation time, the level
//! ladder, per-window statistics and hesitations. Mirrors the voice-core
//! modules of the same names; no signal processing lives here.

/// The longest voiced stretch of a recording (mirror of `voice_core::phonation::Phonation`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Phonation {
    /// Length of the stretch in seconds.
    pub duration_s: f32,
    /// Where it starts, in seconds from the start of the recording.
    pub start_s: f32,
    /// Mean f0 over its voiced frames, in Hz.
    pub mean_f0_hz: f32,
    /// Standard deviation of f0 over its voiced frames, in semitones.
    pub f0_sd_semitones: f32,
    /// RMS level of the stretch in dBFS.
    pub rms_dbfs: f32,
}

/// Longest voiced stretch of mono `samples`, or null when no stretch has at
/// least 10 voiced frames.
pub fn max_phonation(samples: Vec<f32>, sample_rate: u32) -> Result<Option<Phonation>, String> {
    voice_core::phonation::max_phonation(&samples, sample_rate)
        .map(|o| {
            o.map(|p| Phonation {
                duration_s: p.duration_s,
                start_s: p.start_s,
                mean_f0_hz: p.mean_f0_hz,
                f0_sd_semitones: p.f0_sd_semitones,
                rms_dbfs: p.rms_dbfs,
            })
        })
        .map_err(|e| e.to_string())
}

/// One step of the level ladder (mirror of `voice_core::ladder::LadderStep`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LadderStep {
    pub start_s: f64,
    pub end_s: f64,
    /// Seconds of speech found in the window.
    pub speech_s: f32,
    /// Level of the speech in dBFS; null with under 0.2 s of speech.
    pub level_dbfs: Option<f32>,
    /// Level minus the baseline in dB; null when the step or baseline has no level.
    pub relative_db: Option<f32>,
}

/// Levels of the `steps` given as `(start_s, end_s)` windows in seconds. The
/// baseline is the first step with speech unless `baseline_dbfs` is given.
pub fn level_ladder(
    samples: Vec<f32>,
    sample_rate: u32,
    steps: Vec<(f64, f64)>,
    baseline_dbfs: Option<f32>,
) -> Result<Vec<LadderStep>, String> {
    voice_core::ladder::level_ladder(&samples, sample_rate, &steps, baseline_dbfs)
        .map(|v| {
            v.into_iter()
                .map(|s| LadderStep {
                    start_s: s.start_s,
                    end_s: s.end_s,
                    speech_s: s.speech_s,
                    level_dbfs: s.level_dbfs,
                    relative_db: s.relative_db,
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

/// Statistics of one window (mirror of `voice_core::window_stats::WindowStats`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowStats {
    pub start_s: f64,
    pub end_s: f64,
    pub level_dbfs: Option<f32>,
    /// Level minus the recording's speech level, in dB.
    pub relative_level_db: Option<f32>,
    pub voiced_frames: u32,
    pub mean_f0_hz: Option<f32>,
    /// 10th to 90th percentile range of f0 in semitones.
    pub f0_range_semitones: Option<f32>,
    /// Mean f0 against the recording's median f0, in semitones.
    pub relative_pitch_semitones: Option<f32>,
}

/// Statistics of the `windows` given as `(start_s, end_s)` (for example word
/// timestamps), in the order given.
pub fn window_stats(
    samples: Vec<f32>,
    sample_rate: u32,
    windows: Vec<(f64, f64)>,
) -> Result<Vec<WindowStats>, String> {
    voice_core::window_stats::window_stats(&samples, sample_rate, &windows)
        .map(|v| {
            v.into_iter()
                .map(|w| WindowStats {
                    start_s: w.start_s,
                    end_s: w.end_s,
                    level_dbfs: w.level_dbfs,
                    relative_level_db: w.relative_level_db,
                    voiced_frames: w.voiced_frames,
                    mean_f0_hz: w.mean_f0_hz,
                    f0_range_semitones: w.f0_range_semitones,
                    relative_pitch_semitones: w.relative_pitch_semitones,
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

/// The middle half of a value over a recording's ordinary speech (mirror of
/// `voice_core::emphasis::Band`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmphasisBand {
    /// 25th percentile.
    pub low: f32,
    pub median: f32,
    /// 75th percentile.
    pub high: f32,
}

/// The typical range of a recording's ordinary speech (mirror of
/// `voice_core::emphasis::Typical`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypicalRange {
    /// Number of 0.4 s windows inside speech segments that were measured.
    pub windows: u32,
    /// Level relative to the speech level in dB; `None` with under 5 windows.
    pub level_db: Option<EmphasisBand>,
    /// Mean f0 relative to the median f0 in semitones; `None` with under 5 windows.
    pub pitch_semitones: Option<EmphasisBand>,
}

/// How one word stood out (mirror of `voice_core::emphasis::WordEmphasis`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WordEmphasis {
    pub start_s: f64,
    pub end_s: f64,
    pub relative_level_db: Option<f32>,
    pub relative_pitch_semitones: Option<f32>,
    /// Silent gap ending within 0.2 s of the word's start, 0 when none.
    pub pause_before_s: f32,
    /// Silent gap starting within 0.2 s of the word's end, 0 when none.
    pub pause_after_s: f32,
}

/// Emphasis of the given words and the typical range they are judged against.
#[derive(Debug, Clone, PartialEq)]
pub struct EmphasisReport {
    pub typical: TypicalRange,
    pub words: Vec<WordEmphasis>,
}

fn band(b: voice_core::emphasis::Band) -> EmphasisBand {
    EmphasisBand {
        low: b.low,
        median: b.median,
        high: b.high,
    }
}

/// How much each of the `words` (given as `(start_s, end_s)`) stood out in
/// level and pitch against the rest of the recording, and whether a pause sat
/// next to it.
pub fn emphasis(
    samples: Vec<f32>,
    sample_rate: u32,
    words: Vec<(f64, f64)>,
) -> Result<EmphasisReport, String> {
    voice_core::emphasis::emphasis(&samples, sample_rate, &words)
        .map(|r| EmphasisReport {
            typical: TypicalRange {
                windows: r.typical.windows,
                level_db: r.typical.level_db.map(band),
                pitch_semitones: r.typical.pitch_semitones.map(band),
            },
            words: r
                .words
                .into_iter()
                .map(|w| WordEmphasis {
                    start_s: w.start_s,
                    end_s: w.end_s,
                    relative_level_db: w.relative_level_db,
                    relative_pitch_semitones: w.relative_pitch_semitones,
                    pause_before_s: w.pause_before_s,
                    pause_after_s: w.pause_after_s,
                })
                .collect(),
        })
        .map_err(|e| e.to_string())
}

/// What kind of hesitation an event is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HesitationKind {
    /// A silent gap between two stretches of speech.
    Pause,
    /// A voiced sound held at a steady pitch.
    Held,
}

/// One hesitation, in seconds from the start of the recording.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hesitation {
    pub kind: HesitationKind,
    pub start_s: f64,
    pub end_s: f64,
}

/// Hesitations of mono `samples` in time order, with voice-core's default
/// thresholds (0.5 s pause, 0.4 s held sound, 1 semitone, 8 dB). Pauses match
/// Praat; held sounds are only checked on fluent speech so far.
pub fn hesitations(samples: Vec<f32>, sample_rate: u32) -> Result<Vec<Hesitation>, String> {
    use voice_core::hesitations::HesitationKind as K;
    voice_core::hesitations::hesitations(
        &samples,
        sample_rate,
        &voice_core::hesitations::HesitationConfig::default(),
    )
    .map(|v| {
        v.into_iter()
            .map(|h| Hesitation {
                kind: match h.kind {
                    K::Pause => HesitationKind::Pause,
                    K::Held => HesitationKind::Held,
                },
                start_s: h.start_s,
                end_s: h.end_s,
            })
            .collect()
    })
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(hz: f32, secs: f32, sr: u32) -> Vec<f32> {
        (0..(secs * sr as f32) as usize)
            .map(|i| 0.3 * (2.0 * std::f32::consts::PI * hz * i as f32 / sr as f32).sin())
            .collect()
    }

    #[test]
    fn phonation_of_a_held_tone() {
        let p = max_phonation(sine(200.0, 2.0, 16_000), 16_000)
            .unwrap()
            .expect("voiced");
        assert!(p.duration_s > 1.5, "{}", p.duration_s);
        assert!((p.mean_f0_hz - 200.0).abs() < 3.0);
    }

    #[test]
    fn silence_has_no_phonation() {
        assert_eq!(max_phonation(vec![0.0; 32_000], 16_000).unwrap(), None);
    }

    #[test]
    fn ladder_gives_one_step_per_window_and_relative_levels() {
        let mut s = sine(200.0, 1.0, 16_000);
        s.extend(sine(200.0, 1.0, 16_000).iter().map(|x| x * 0.5));
        let steps = level_ladder(s, 16_000, vec![(0.0, 1.0), (1.0, 2.0)], None).unwrap();
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].relative_db, Some(0.0));
        let r = steps[1].relative_db.expect("second step has speech");
        assert!((r + 6.0).abs() < 1.0, "{r}");
    }

    #[test]
    fn bad_ladder_step_is_an_error_message() {
        let e = level_ladder(vec![0.1; 1_600], 16_000, vec![(1.0, 0.5)], None).unwrap_err();
        assert!(e.contains("start < end"), "{e}");
    }

    #[test]
    fn window_stats_keep_the_given_order() {
        let w = window_stats(
            sine(200.0, 2.0, 16_000),
            16_000,
            vec![(0.5, 1.0), (0.0, 0.5)],
        )
        .unwrap();
        assert_eq!(w.len(), 2);
        assert_eq!(w[0].start_s, 0.5);
        let f0 = w[0].mean_f0_hz.expect("voiced");
        assert!((f0 - 200.0).abs() < 3.0);
    }

    #[test]
    fn emphasis_keeps_the_given_words() {
        let mut s = sine(150.0, 0.5, 16_000);
        s.extend(vec![0.0; 8_000]);
        let words = vec![(0.0, 0.4), (0.1, 0.5)];
        let r = emphasis(s, 16_000, words).unwrap();
        assert_eq!(r.words.len(), 2);
        assert_eq!(r.words[1].start_s, 0.1);
        assert!(
            r.typical.level_db.is_none(),
            "too little speech for a range"
        );
    }

    #[test]
    fn a_gap_is_a_pause() {
        let mut s = sine(200.0, 1.0, 16_000);
        s.extend(vec![0.0; 16_000]);
        s.extend(sine(200.0, 1.0, 16_000));
        let h = hesitations(s, 16_000).unwrap();
        assert!(h.iter().any(|e| e.kind == HesitationKind::Pause), "{h:?}");
    }

    #[test]
    fn empty_input_is_an_error_message() {
        assert!(hesitations(vec![], 16_000).is_err());
        assert!(window_stats(vec![], 16_000, vec![(0.0, 1.0)]).is_err());
        assert!(emphasis(vec![], 16_000, vec![(0.0, 1.0)]).is_err());
    }
}
