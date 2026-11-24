use std::process::Command;

pub const SUBSYSTEM: &str = "transcoding";
pub const ENABLED_BY_DEFAULT: bool = false;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscoderStatus {
    pub available: bool,
    pub engine_name: Option<String>,
    pub version_line: Option<String>,
}

/// Проверяет наличие внешнего движка только по запросу пользователя (ТЗ п. 3.3).
/// Оригинальные файлы никогда не изменяются.
pub fn probe_engine() -> TranscoderStatus {
    if let Ok(out) = Command::new("ffmpeg").arg("-version").output()
        && out.status.success() {
            let first_line = String::from_utf8_lossy(&out.stdout)
                .lines()
                .next()
                .map(|s| s.trim().to_string());
            return TranscoderStatus {
                available: true,
                engine_name: Some("ffmpeg".into()),
                version_line: first_line,
            };
        }
    TranscoderStatus {
        available: false,
        engine_name: None,
        version_line: None,
    }
}
