//! Plain, serialisable mirrors of voice-core's report types. No signal
//! processing lives here; every number comes from `voice_core::analyze`.

/// Something that makes a recording's measurements less trustworthy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceWarning {
    TooShort,
    Clipped,
    TooQuiet,
}

impl From<voice_core::quality::Warning> for VoiceWarning {
    fn from(w: voice_core::quality::Warning) -> Self {
        use voice_core::quality::Warning as W;
        match w {
            W::TooShort => VoiceWarning::TooShort,
            W::Clipped => VoiceWarning::Clipped,
            W::TooQuiet => VoiceWarning::TooQuiet,
        }
    }
}

/// Measurements for one recording (mirror of `voice_core::VoiceReport`).
#[derive(Debug, Clone)]
pub struct VoiceReport {
    pub duration_s: f64,
    pub sample_rate_hz: u32,
    pub rms_dbfs: f32,
    pub lufs: f64,
    pub f0_median_hz: Option<f32>,
    pub voiced_fraction: f32,
    pub syllable_count: u32,
    pub syllables_per_second: f32,
    pub pause_count: u32,
    pub pause_total_s: f32,
    pub longest_pause_s: f32,
    pub warnings: Vec<VoiceWarning>,
}

impl From<voice_core::VoiceReport> for VoiceReport {
    fn from(r: voice_core::VoiceReport) -> Self {
        VoiceReport {
            duration_s: r.duration_s,
            sample_rate_hz: r.sample_rate_hz,
            rms_dbfs: r.rms_dbfs,
            lufs: r.lufs,
            f0_median_hz: r.f0_median_hz,
            voiced_fraction: r.voiced_fraction,
            syllable_count: r.syllable_count,
            syllables_per_second: r.syllables_per_second,
            pause_count: r.pause_count,
            pause_total_s: r.pause_total_s,
            longest_pause_s: r.longest_pause_s,
            warnings: r.warnings.into_iter().map(Into::into).collect(),
        }
    }
}

/// Analyses mono `samples` (floats in -1..1) recorded at `sample_rate` Hz.
///
/// Errors are returned as the voice-core message, which Dart receives as an
/// exception.
pub fn analyze_samples(samples: Vec<f32>, sample_rate: u32) -> Result<VoiceReport, String> {
    voice_core::analyze(&samples, sample_rate)
        .map(Into::into)
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
    fn sine_reports_expected_pitch_and_duration() {
        let r = analyze_samples(sine(200.0, 2.0, 16_000), 16_000).unwrap();
        assert!((r.duration_s - 2.0).abs() < 1e-6);
        assert_eq!(r.sample_rate_hz, 16_000);
        let f0 = r.f0_median_hz.expect("voiced");
        assert!((f0 - 200.0).abs() < 2.0, "f0 {f0}");
        assert!(r.warnings.is_empty());
    }

    #[test]
    fn empty_input_is_an_error_message() {
        let e = analyze_samples(vec![], 16_000).unwrap_err();
        assert!(e.contains("no samples"), "{e}");
    }

    #[test]
    fn silence_maps_quality_warning() {
        let r = analyze_samples(vec![0.0; 32_000], 16_000).unwrap();
        assert!(r.warnings.contains(&VoiceWarning::TooQuiet));
    }

    #[test]
    fn zero_rate_is_an_error() {
        assert!(analyze_samples(vec![0.1; 100], 0).is_err());
    }
}
