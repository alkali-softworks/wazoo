/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Platform Integration
 * 
 * Provides Linux desktop environment helpers, detecting cursor size and theme settings via
 * gsettings and managing window activation and focus via wmctrl and xdotool.
 */

#[cfg(target_os = "linux")]
pub fn init_linux_cursor_env() {
    if std::env::var_os("XCURSOR_SIZE").is_none() {
        let size = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "cursor-size"])
            .output()
            .or_else(|_| {
                std::process::Command::new("gsettings")
                    .args(["get", "org.cinnamon.desktop.interface", "cursor-size"])
                    .output()
            })
            .ok()
            .and_then(|out| {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                s.parse::<u32>().ok()
            });

        if let Some(sz) = size {
            if sz > 0 {
                std::env::set_var("XCURSOR_SIZE", sz.to_string());
            }
        }
    }

    if std::env::var_os("XCURSOR_THEME").is_none() {
        let theme = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "cursor-theme"])
            .output()
            .or_else(|_| {
                std::process::Command::new("gsettings")
                    .args(["get", "org.cinnamon.desktop.interface", "cursor-theme"])
                    .output()
            })
            .ok()
            .and_then(|out| {
                let s = String::from_utf8_lossy(&out.stdout)
                    .trim()
                    .trim_matches('\'')
                    .to_string();
                if s.is_empty() { None } else { Some(s) }
            });

        if let Some(th) = theme {
            std::env::set_var("XCURSOR_THEME", th);
        }
    }
}

#[cfg(target_os = "linux")]
pub async fn ensure_window_focused_linux() {
    let my_pid = std::process::id();
    for _ in 0..25 {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        let mut activated = false;
        if let Ok(output) = std::process::Command::new("wmctrl")
            .args(["-l", "-p"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        if let Ok(pid) = parts[2].parse::<u32>() {
                            if pid == my_pid {
                                let win_id = parts[0];
                                let s = std::process::Command::new("wmctrl")
                                    .args(["-i", "-a", win_id])
                                    .status();
                                if let Ok(status) = s {
                                    if status.success() {
                                        activated = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !activated {
            if let Ok(status) = std::process::Command::new("xdotool")
                .args(["search", "--pid", &my_pid.to_string(), "windowactivate"])
                .status()
            {
                if status.success() {
                    activated = true;
                }
            }
        }

        if activated {
            break;
        }
    }
}
