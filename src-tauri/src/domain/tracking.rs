use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TrackingState {
    pub onboarding_completed: bool,
    pub auto_capture_enabled: bool,
}
