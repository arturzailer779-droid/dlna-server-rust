use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    #[serde(alias = "server_name")]
    pub name: String,
    pub icon: PathBuf,
    pub port: u16,
    pub uuid: String,
    pub playback_mode: String,
    pub transcoding_enabled: bool,
    pub transcode_mode: String,
    pub language: String,
    pub locale: String,
    pub autostart: bool,
    pub anti_sleep: bool,
    #[serde(alias = "video_roots")]
    pub roots: Vec<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "DLNA VIDEO SERVER 2.0".into(),
            icon: PathBuf::from("/usr/share/dlna-server/assets/logo_tv.png"),
            port: 8208,
            uuid: generate_fallback_uuid(),
            playback_mode: "automatic".into(),
            transcoding_enabled: false,
            transcode_mode: "off".into(),
            language: "ru".into(),
            locale: "ru_RU".into(),
            autostart: false,
            anti_sleep: true,
            roots: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            let mut config = Self::default();
            config.try_migrate_legacy();
            config.save(path)?;
            return Ok(config);
        }

        let source = fs::read_to_string(path)
            .with_context(|| format!("Не удалось прочитать {}", path.display()))?;
        toml::from_str(&source)
            .with_context(|| format!("Некорректный TOML в {}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Не удалось создать {}", parent.display()))?;
        }

        let serialized = toml::to_string_pretty(self)?;
        let temporary = path.with_extension(format!("toml.tmp-{}", std::process::id()));
        fs::write(&temporary, serialized)
            .with_context(|| format!("Не удалось записать {}", temporary.display()))?;
        fs::rename(&temporary, path)
            .with_context(|| format!("Не удалось обновить {}", path.display()))?;
        Ok(())
    }

    fn try_migrate_legacy(&mut self) {
        let Some(home) = std::env::var_os("HOME") else { return };
        let legacy_dir = PathBuf::from(home).join(".dlna_server_1_1");
        if !legacy_dir.is_dir() {
            return;
        }
        if let Ok(uuid) = fs::read_to_string(legacy_dir.join("uuid.txt")) {
            let trimmed = uuid.trim();
            if !trimmed.is_empty() {
                self.uuid = trimmed.to_string();
            }
        }
        if let Ok(lang) = fs::read_to_string(legacy_dir.join("lang.txt")) {
            let trimmed = lang.trim();
            if !trimmed.is_empty() {
                self.language = trimmed.to_string();
            }
        }
        if let Ok(loc) = fs::read_to_string(legacy_dir.join("locale.txt")) {
            let trimmed = loc.trim();
            if !trimmed.is_empty() {
                self.locale = trimmed.to_string();
            }
        }
        if let Ok(core_conf) = fs::read_to_string(legacy_dir.join("core.conf")) {
            for line in core_conf.lines() {
                if let Some((k, v)) = line.trim().split_once('=') {
                    match k.trim() {
                        "port" => {
                            if let Ok(p) = v.trim().parse::<u16>()
                                && p >= 1024 { self.port = p; }
                        }
                        "name" => {
                            let n = v.trim();
                            if !n.is_empty() { self.name = n.to_string(); }
                        }
                        "root" => {
                            let p = PathBuf::from(v.trim());
                            if p.is_dir() && !self.roots.contains(&p) {
                                self.roots.push(p);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

pub fn default_config_dir() -> Result<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg).join("dlna-video-server-2"));
    }
    let home = std::env::var_os("HOME").context("Не найдена переменная HOME")?;
    Ok(PathBuf::from(home).join(".config").join("dlna-video-server-2"))
}

pub fn default_config_path() -> Result<PathBuf> {
    Ok(default_config_dir()?.join("config.toml"))
}

pub fn default_index_cache_path() -> Result<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(xdg).join("dlna-video-server-2").join("media-index.tsv"));
    }
    let home = std::env::var_os("HOME").context("Не найдена переменная HOME")?;
    Ok(PathBuf::from(home).join(".cache").join("dlna-video-server-2").join("media-index.tsv"))
}

fn generate_fallback_uuid() -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let pid = std::process::id() as u128;
    let mixed = nanos ^ (pid << 64) ^ 0x4d69_6e69_444c_4e41_5275_7374_3230_3236;
    format!(
        "{:08x}-{:04x}-4{:03x}-a{:03x}-{:012x}",
        (mixed >> 96) as u32,
        ((mixed >> 80) & 0xffff) as u16,
        ((mixed >> 64) & 0x0fff) as u16,
        ((mixed >> 48) & 0x0fff) as u16,
        (mixed & 0xffff_ffff_ffff) as u64
    )
}
