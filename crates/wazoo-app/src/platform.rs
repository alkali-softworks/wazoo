/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Platform Integration
 *
 * Provides Linux desktop environment helpers, detecting cursor size and theme settings via
 * gsettings and managing window activation and focus via wmctrl and xdotool.
 */

#[cfg(target_os = "linux")]
pub fn init_linux_cursor_env() {
    init_linux_desktop_entry();

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
                // SAFETY: Setting environment variables at startup before background worker threads run.
                unsafe {
                    std::env::set_var("XCURSOR_SIZE", sz.to_string());
                }
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
            // SAFETY: Setting environment variables at startup before background worker threads run.
            unsafe {
                std::env::set_var("XCURSOR_THEME", th);
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub fn init_linux_desktop_entry() {
    // Install the .desktop file and icon into user's local XDG directories
    // so desktop environments (Cinnamon, GNOME, KDE) properly display the icon
    // in taskbar/panel, Alt+Tab, and application launchers.
    if let Some(home) = std::env::var_os("HOME") {
        let home = std::path::PathBuf::from(home);
        let icon_dir = home.join(".local/share/icons/hicolor/512x512/apps");
        let _ = std::fs::create_dir_all(&icon_dir);
        let icon_path = icon_dir.join("wazoo.png");
        let icon_bytes = include_bytes!("../resources/icon.png");

        let write_icon = match std::fs::metadata(&icon_path) {
            Ok(meta) => meta.len() != icon_bytes.len() as u64,
            Err(_) => true,
        };
        if write_icon {
            let _ = std::fs::write(&icon_path, icon_bytes);
        }

        let generic_icon_path = home.join(".local/share/icons/wazoo.png");
        let write_generic = match std::fs::metadata(&generic_icon_path) {
            Ok(meta) => meta.len() != icon_bytes.len() as u64,
            Err(_) => true,
        };
        if write_generic {
            let _ = std::fs::write(&generic_icon_path, icon_bytes);
        }

        let apps_dir = home.join(".local/share/applications");
        let _ = std::fs::create_dir_all(&apps_dir);
        let desktop_path = apps_dir.join("wazoo.desktop");

        if let Ok(current_exe) = std::env::current_exe() {
            let exe_str = current_exe.to_string_lossy();
            let desktop_content = format!(
                "[Desktop Entry]\n\
                Type=Application\n\
                Name=Wazoo\n\
                GenericName=Ambient Media Engine\n\
                Comment=Native Rust ambient media engine for non-stop viewing\n\
                Exec=\"{}\"\n\
                Icon=wazoo\n\
                Terminal=false\n\
                Categories=AudioVideo;Video;Player;\n\
                StartupWMClass=wazoo\n",
                exe_str
            );
            let _ = std::fs::write(&desktop_path, desktop_content);
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

#[cfg(target_os = "linux")]
pub fn is_hybrid_laptop() -> bool {
    // Detect hybrid laptops (e.g. Intel/AMD iGPU + NVIDIA dGPU driving an internal eDP panel).
    // On desktop PCs, monitors are plugged directly into the dedicated GPU via DP/HDMI
    // with no battery or eDP panel, so the dedicated GPU should be preferred directly.
    let has_battery = std::path::Path::new("/sys/class/power_supply/BAT0").exists()
        || std::path::Path::new("/sys/class/power_supply/BAT1").exists();

    let has_edp = std::fs::read_dir("/sys/class/drm")
        .map(|entries| {
            entries.filter_map(Result::ok).any(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                name.contains("eDP")
            })
        })
        .unwrap_or(false);

    let card_count = std::fs::read_dir("/sys/class/drm")
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|e| {
                    let name = e.file_name().to_string_lossy().into_owned();
                    name.starts_with("card") && !name.contains('-')
                })
                .count()
        })
        .unwrap_or(0);

    (has_edp || has_battery) && card_count > 1
}

pub fn open_url(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}
