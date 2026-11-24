use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PlaybackMode {
    DirectPlayOnly,
    #[default]
    Automatic,
    Transcoding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CapabilityConfidence {
    Confirmed,
    Reported,
    Inferred,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientCapability {
    pub mime_type: String,
    pub codec: Option<String>,
    pub confidence: CapabilityConfidence,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientProfile {
    pub id: String,
    pub display_name: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub capabilities: Vec<ClientCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerState {
    Stopped,
    Starting,
    Running,
    Degraded(String),
    Stopping,
}
