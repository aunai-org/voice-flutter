//! Live analysis: feed audio in chunks as it arrives and get one `FrameInfo`
//! per 10 ms. Mirrors `voice_core::stream`; no signal processing lives here.

use std::sync::Mutex;

/// What is known about one 10 ms frame (mirror of `voice_core::stream::FrameInfo`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameInfo {
    /// Centre of the frame in seconds from the start of the stream.
    pub time_s: f64,
    /// Level in dBFS.
    pub level_dbfs: f32,
    /// Whether the frame counts as speech.
    pub speech: bool,
    /// Fundamental frequency in Hz, or null when the frame is unvoiced.
    pub f0_hz: Option<f32>,
    /// Whether the frame is a syllable nucleus.
    pub syllable_nucleus: bool,
    /// Syllables per second over the last 3 s, or null in the first second.
    pub syllable_rate_hz: Option<f32>,
}

impl From<voice_core::stream::FrameInfo> for FrameInfo {
    fn from(f: voice_core::stream::FrameInfo) -> Self {
        FrameInfo {
            time_s: f.time_s,
            level_dbfs: f.level_dbfs,
            speech: f.speech,
            f0_hz: f.f0_hz,
            syllable_nucleus: f.syllable_nucleus,
            syllable_rate_hz: f.syllable_rate_hz,
        }
    }
}

/// How far behind the audio the frames are: pitch, the speech flag and syllable
/// nuclei look 150 ms ahead, plus the 20 ms level window.
pub const STREAM_DELAY_S: f64 = 0.17;

/// Push-style analyzer for one mono stream (wraps `voice_core::stream::Analyzer`).
#[flutter_rust_bridge::frb(opaque)]
pub struct StreamAnalyzer {
    inner: Mutex<Option<voice_core::stream::Analyzer>>,
}

impl StreamAnalyzer {
    /// Creates an analyzer for audio at `sample_rate` Hz. `reference_level_dbfs`
    /// is the loudest level of an earlier recording, if known, so the first words
    /// are judged against a realistic speech level.
    #[flutter_rust_bridge::frb(sync)]
    pub fn new(
        sample_rate: u32,
        reference_level_dbfs: Option<f32>,
    ) -> Result<StreamAnalyzer, String> {
        let mut a = voice_core::stream::Analyzer::new(sample_rate).map_err(|e| e.to_string())?;
        if let Some(level) = reference_level_dbfs {
            a = a.with_reference_level(level);
        }
        Ok(StreamAnalyzer {
            inner: Mutex::new(Some(a)),
        })
    }

    /// Adds mono samples (floats in -1..1) and returns the frames that are now
    /// complete. Empty after `finish`.
    pub fn push(&self, samples: Vec<f32>) -> Vec<FrameInfo> {
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        match guard.as_mut() {
            Some(a) => a.push(&samples).into_iter().map(Into::into).collect(),
            None => Vec::new(),
        }
    }

    /// Ends the stream and returns the remaining frames. Later calls return none.
    pub fn finish(&self) -> Vec<FrameInfo> {
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        match guard.take() {
            Some(a) => a.finish().into_iter().map(Into::into).collect(),
            None => Vec::new(),
        }
    }
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
    fn chunks_give_pitch_and_speech_frames() {
        let a = StreamAnalyzer::new(16_000, None).unwrap();
        let mut frames = Vec::new();
        for chunk in sine(200.0, 2.0, 16_000).chunks(1_600) {
            frames.extend(a.push(chunk.to_vec()));
        }
        frames.extend(a.finish());
        assert!(frames.len() >= 195, "{} frames", frames.len());
        let mid = &frames[100];
        assert!(mid.speech);
        assert!((mid.f0_hz.expect("voiced") - 200.0).abs() < 3.0);
    }

    #[test]
    fn push_after_finish_is_empty() {
        let a = StreamAnalyzer::new(16_000, None).unwrap();
        a.push(vec![0.1; 3_200]);
        a.finish();
        assert!(a.push(vec![0.1; 1_600]).is_empty());
        assert!(a.finish().is_empty());
    }

    #[test]
    fn zero_rate_is_an_error() {
        assert!(StreamAnalyzer::new(0, None).is_err());
    }
}
