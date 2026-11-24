use dlna_core::{CapabilityConfidence, ClientCapability, ClientProfile, PlaybackMode};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct ClientRegistry {
    clients: Mutex<HashMap<String, ClientProfile>>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn observe(&self, ip: &str, user_agent: Option<&str>) -> ClientProfile {
        let ua = user_agent.unwrap_or("Unknown DLNA Client").trim();
        let (display_name, manufacturer, capabilities) = classify_user_agent(ua);

        let profile = ClientProfile {
            id: ip.to_string(),
            display_name,
            manufacturer,
            model: Some(ua.chars().take(64).collect()),
            capabilities,
        };

        if let Ok(mut guard) = self.clients.lock() {
            guard.insert(ip.to_string(), profile.clone());
        }
        profile
    }

    pub fn list(&self) -> Vec<ClientProfile> {
        self.clients
            .lock()
            .map(|g| g.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Никогда не блокирует Direct Play при неполных данных о клиенте (ТЗ п. 4)
    pub fn can_direct_play(&self, mode: PlaybackMode, _mime: &str) -> bool {
        match mode {
            PlaybackMode::DirectPlayOnly | PlaybackMode::Automatic => true,
            PlaybackMode::Transcoding => false,
        }
    }
}

fn classify_user_agent(ua: &str) -> (String, Option<String>, Vec<ClientCapability>) {
    let lower = ua.to_ascii_lowercase();
    if lower.contains("samsung") || lower.contains("tizen") {
        (
            "Samsung Smart TV".into(),
            Some("Samsung".into()),
            vec![ClientCapability {
                mime_type: "video/x-matroska".into(),
                codec: None,
                confidence: CapabilityConfidence::Inferred,
                evidence: format!("User-Agent: {ua}"),
            }],
        )
    } else if lower.contains("lg") || lower.contains("webos") {
        (
            "LG webOS TV".into(),
            Some("LG".into()),
            vec![ClientCapability {
                mime_type: "video/mp4".into(),
                codec: None,
                confidence: CapabilityConfidence::Inferred,
                evidence: format!("User-Agent: {ua}"),
            }],
        )
    } else if lower.contains("vlc") || lower.contains("lavf") {
        (
            "VLC Media Player".into(),
            Some("VideoLAN".into()),
            vec![ClientCapability {
                mime_type: "video/*".into(),
                codec: None,
                confidence: CapabilityConfidence::Reported,
                evidence: format!("User-Agent: {ua}"),
            }],
        )
    } else {
        (
            format!("DLNA-устройство ({})", ua.chars().take(28).collect::<String>()),
            None,
            vec![ClientCapability {
                mime_type: "video/mp4".into(),
                codec: None,
                confidence: CapabilityConfidence::Unknown,
                evidence: format!("Неполные данные User-Agent: {ua}"),
            }],
        )
    }
}
