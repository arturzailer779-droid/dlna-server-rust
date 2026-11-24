use anyhow::{Context, Result};
use std::{
    fs,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

pub fn autostart_file_path() -> Result<PathBuf> {
    let base = if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(xdg)
    } else {
        let home = std::env::var_os("HOME").context("Не найдена переменная HOME")?;
        PathBuf::from(home).join(".config")
    };
    Ok(base.join("autostart").join("dlna-video-server-2.desktop"))
}

pub fn is_autostart_enabled() -> bool {
    autostart_file_path().map(|p| p.is_file()).unwrap_or(false)
}

pub fn set_autostart(enabled: bool) -> Result<()> {
    let target = autostart_file_path()?;
    if enabled {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        let exe = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("/usr/bin/dlna-video-server-2"));
        let entry = format!(
            "[Desktop Entry]\nType=Application\nName=DLNA VIDEO SERVER 2.0\n\
             Exec=\"{}\" --minimized\nIcon=dlna-server\nTerminal=false\n\
             X-GNOME-Autostart-enabled=true\n",
            exe.display()
        );
        fs::write(&target, entry)?;
    } else if target.exists() {
        fs::remove_file(&target)?;
    }
    Ok(())
}

pub struct SleepInhibitor {
    #[cfg(target_os = "linux")]
    child: Option<Child>,
}

impl SleepInhibitor {
    pub fn acquire() -> Self {
        #[cfg(target_os = "linux")]
        {
            let mut child = Command::new("systemd-inhibit")
                .args([
                    "--what=idle:sleep",
                    "--who=DLNA Video Server 2.6.0",
                    "--why=Streaming video over DLNA",
                    "--mode=block",
                    "sleep",
                    "infinity",
                ])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .ok();
            if let Some(ref mut c) = child
                && matches!(c.try_wait(), Ok(Some(_)) | Err(_))
            {
                child = None;
            }
            Self { child }
        }
        #[cfg(not(target_os = "linux"))]
        {
            Self {}
        }
    }

    pub fn is_active(&mut self) -> bool {
        #[cfg(target_os = "linux")]
        {
            if let Some(ref mut c) = self.child {
                return matches!(c.try_wait(), Ok(None));
            }
            false
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }
}

impl Drop for SleepInhibitor {
    fn drop(&mut self) {
        #[cfg(target_os = "linux")]
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub fn is_ufw_blocking() -> bool {
    #[cfg(target_os = "linux")]
    {
        let Ok(output) = Command::new("ufw").arg("status").output() else {
            return false;
        };
        let text = String::from_utf8_lossy(&output.stdout);
        if !output.status.success() || !text.contains("Status: active") {
            return false;
        }
        let mut tcp_ok = false;
        let mut udp_ok = false;
        for line in text.lines() {
            let upper = line.trim().to_ascii_uppercase();
            if !upper.contains("ALLOW") || upper.contains("DENY") || upper.contains("REJECT") {
                continue;
            }
            if upper.contains("8208/TCP") || (upper.contains("8208") && upper.contains("TCP")) {
                tcp_ok = true;
            }
            if upper.contains("1900/UDP") || (upper.contains("1900") && upper.contains("UDP")) {
                udp_ok = true;
            }
        }
        !(tcp_ok && udp_ok)
    }
    #[cfg(not(target_os = "linux"))]
    false
}
