//! Predictive Transcoding Engine (PTE) — first measurement-and-forecast prototype.
//! This module does not launch codecs and does not alter the existing streaming path.
//! It turns measured stage rates into an explainable playback-risk decision.

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Demux,
    Decode,
    Scale,
    Encode,
    Mux,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StageRates {
    pub demux_fps: f64,
    pub decode_fps: f64,
    pub scale_fps: f64,
    pub encode_fps: f64,
    pub mux_fps: f64,
}

impl StageRates {
    pub fn bottleneck(self) -> (Stage, f64) {
        let stages = [
            (Stage::Demux, self.demux_fps),
            (Stage::Decode, self.decode_fps),
            (Stage::Scale, self.scale_fps),
            (Stage::Encode, self.encode_fps),
            (Stage::Mux, self.mux_fps),
        ];
        stages.into_iter()
            .filter(|(_, fps)| fps.is_finite() && *fps >= 0.0)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap_or((Stage::Encode, 0.0))
    }

    pub fn valid(self) -> bool {
        [self.demux_fps, self.decode_fps, self.scale_fps,
         self.encode_fps, self.mux_fps]
            .into_iter().all(|v| v.is_finite() && v >= 0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Observation {
    /// Source media frame rate, not wall-clock sampling rate.
    pub source_fps: f64,
    pub stages: StageRates,
    /// Amount of already-produced media waiting for the client, in media seconds.
    pub buffer_seconds: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Forecast {
    pub bottleneck: Stage,
    pub bottleneck_fps: f64,
    pub realtime_ratio: f64,
    pub buffer_seconds: f64,
    pub estimated_underrun_seconds: Option<f64>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskState {
    WarmingUp,
    Healthy,
    Watch,
    AtRisk,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    KeepCurrentPath,
    PreferPretranscode,
    CannotMeetRealtimeContract,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Decision {
    pub state: RiskState,
    pub action: Action,
    pub forecast: Forecast,
}

#[derive(Debug)]
pub struct PteController {
    alpha: f64,
    smoothed_ratio: Option<f64>,
    history: VecDeque<f64>,
    state: RiskState,
    bad_windows: u8,
    good_windows: u8,
    history_limit: usize,
}

impl Default for PteController {
    fn default() -> Self {
        Self::new(0.25, 12)
    }
}

impl PteController {
    pub fn new(alpha: f64, history_limit: usize) -> Self {
        Self {
            alpha: alpha.clamp(0.01, 1.0),
            smoothed_ratio: None,
            history: VecDeque::with_capacity(history_limit.max(1)),
            state: RiskState::WarmingUp,
            bad_windows: 0,
            good_windows: 0,
            history_limit: history_limit.max(1),
        }
    }

    pub fn observe(
        &mut self,
        observation: Observation,
        direct_play_compatible: bool,
        pretranscode_allowed: bool,
    ) -> Decision {
        let (bottleneck, bottleneck_fps) = observation.stages.bottleneck();
        let source_fps = observation.source_fps.max(0.001);
        let ratio = bottleneck_fps / source_fps;

        let smoothed = match self.smoothed_ratio {
            Some(previous) => self.alpha * ratio + (1.0 - self.alpha) * previous,
            None => ratio,
        };
        self.smoothed_ratio = Some(smoothed);

        self.history.push_back(smoothed);
        if self.history.len() > self.history_limit {
            self.history.pop_front();
        }

        self.update_state(smoothed, observation.buffer_seconds);
        let underrun = if smoothed < 1.0 {
            Some(observation.buffer_seconds.max(0.0) / (1.0 - smoothed))
        } else {
            None
        };

        let confidence = (self.history.len() as f64 / 6.0).min(1.0);
        let forecast = Forecast {
            bottleneck,
            bottleneck_fps,
            realtime_ratio: smoothed,
            buffer_seconds: observation.buffer_seconds.max(0.0),
            estimated_underrun_seconds: underrun,
            confidence,
        };

        let action = if direct_play_compatible {
            Action::KeepCurrentPath
        } else if smoothed >= 1.0 && self.state != RiskState::Critical {
            Action::KeepCurrentPath
        } else if pretranscode_allowed {
            Action::PreferPretranscode
        } else {
            Action::CannotMeetRealtimeContract
        };

        Decision { state: self.state, action, forecast }
    }

    fn update_state(&mut self, ratio: f64, buffer_seconds: f64) {
        if ratio < 0.90 || (ratio < 1.0 && buffer_seconds < 2.0) {
            self.bad_windows = self.bad_windows.saturating_add(1);
        } else {
            self.bad_windows = 0;
        }

        if ratio >= 1.15 && buffer_seconds >= 3.0 {
            self.good_windows = self.good_windows.saturating_add(1);
        } else {
            self.good_windows = 0;
        }

        self.state = match self.state {
            RiskState::WarmingUp => {
                if self.history.len() < 3 { RiskState::WarmingUp }
                else if ratio < 1.0 { RiskState::AtRisk }
                else { RiskState::Healthy }
            }
            RiskState::Healthy | RiskState::Watch => {
                if self.bad_windows >= 2 { RiskState::AtRisk }
                else if ratio < 1.10 { RiskState::Watch }
                else { RiskState::Healthy }
            }
            RiskState::AtRisk => {
                if ratio < 0.90 || buffer_seconds < 1.0 { RiskState::Critical }
                else if self.good_windows >= 3 { RiskState::Healthy }
                else { RiskState::AtRisk }
            }
            RiskState::Critical => {
                if self.good_windows >= 4 { RiskState::Healthy }
                else { RiskState::Critical }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation(fps: f64, buffer: f64) -> Observation {
        Observation {
            source_fps: 30.0,
            stages: StageRates {
                demux_fps: 300.0,
                decode_fps: 60.0,
                scale_fps: 55.0,
                encode_fps: fps,
                mux_fps: 120.0,
            },
            buffer_seconds: buffer,
        }
    }

    #[test]
    fn finds_the_actual_slowest_stage() {
        let (stage, fps) = observation(24.0, 5.0).stages.bottleneck();
        assert_eq!(stage, Stage::Encode);
        assert_eq!(fps, 24.0);
    }

    #[test]
    fn forecasts_buffer_drain_when_pipeline_is_slow() {
        let mut pte = PteController::default();
        let decision = pte.observe(observation(24.0, 3.0), false, true);
        // 24/30 = 0.8; a 3-second media buffer drains in about 15 seconds.
        assert!((decision.forecast.estimated_underrun_seconds.unwrap() - 15.0).abs() < 0.01);
        assert_eq!(decision.action, Action::PreferPretranscode);
    }

    #[test]
    fn keeps_live_path_when_throughput_is_sufficient() {
        let mut pte = PteController::default();
        let decision = pte.observe(observation(45.0, 5.0), false, true);
        assert_eq!(decision.action, Action::KeepCurrentPath);
        assert!(decision.forecast.realtime_ratio > 1.0);
    }

    #[test]
    fn direct_play_bypasses_transcoding_recommendation() {
        let mut pte = PteController::default();
        let decision = pte.observe(observation(5.0, 0.5), true, false);
        assert_eq!(decision.action, Action::KeepCurrentPath);
    }
}
