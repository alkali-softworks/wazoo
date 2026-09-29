/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Platform Integration
 *
 * Provides Linux desktop environment helpers, detecting cursor size and theme settings via
 * gsettings and managing window activation and focus via wmctrl and xdotool.
 */

#[cfg(target_os = "linux")]
pub fn init_linux_cursor_env(is_default_player: bool) {
    init_linux_desktop_entry(is_default_player);
    if is_default_player {
        let _ = set_as_default_video_player();
    }

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

pub const SUPPORTED_VIDEO_MIMETYPES: &[&str] = &[
    "video/x-matroska",
    "video/mp4",
    "video/webm",
    "video/x-msvideo",
    "video/quicktime",
];

#[cfg(target_os = "linux")]
pub fn init_linux_desktop_entry(is_default_player: bool) {
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

        let target_exe = std::env::var("APPIMAGE")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(std::path::PathBuf::from)
            .or_else(|| std::env::current_exe().ok());

        if let Some(current_exe) = target_exe {
            let exe_str = current_exe.to_string_lossy();
            let mime_line = if is_default_player {
                let mimetypes = format!("{};", SUPPORTED_VIDEO_MIMETYPES.join(";"));
                format!("MimeType={}\n", mimetypes)
            } else {
                String::new()
            };
            let desktop_content = format!(
                "[Desktop Entry]\n\
                Type=Application\n\
                Name=Wazoo\n\
                GenericName=Ambient Media Engine\n\
                Comment=Native Rust ambient media engine for non-stop viewing\n\
                Exec=\"{}\" %U\n\
                Icon=wazoo\n\
                Terminal=false\n\
                Categories=AudioVideo;Video;Player;\n\
                StartupWMClass=wazoo\n\
                {}",
                exe_str, mime_line
            );
            let _ = std::fs::write(&desktop_path, desktop_content);
            let _ = std::process::Command::new("update-desktop-database")
                .arg(&apps_dir)
                .output();
        }
    }
}

#[cfg(target_os = "linux")]
pub fn set_as_default_video_player() -> Result<(), String> {
    init_linux_desktop_entry(true);
    let mut failed = Vec::new();
    for mime in SUPPORTED_VIDEO_MIMETYPES {
        let status = std::process::Command::new("xdg-mime")
            .args(["default", "wazoo.desktop", mime])
            .status();
        match status {
            Ok(s) if s.success() => {}
            _ => failed.push(*mime),
        }
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!("Failed to set default for: {}", failed.join(", ")))
    }
}

#[cfg(not(target_os = "linux"))]
pub fn set_as_default_video_player() -> Result<(), String> {
    Err("Setting default video player is only supported on Linux".to_string())
}

#[cfg(target_os = "linux")]
pub fn unset_as_default_video_player() -> Result<(), String> {
    init_linux_desktop_entry(false);

    if let Some(home) = std::env::var_os("HOME") {
        let home = std::path::PathBuf::from(home);
        let mimeapps_path = home.join(".config/mimeapps.list");
        if mimeapps_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&mimeapps_path) {
                let mut new_lines = Vec::new();
                let mut in_default_section = false;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with('[') {
                        in_default_section = trimmed.eq_ignore_ascii_case("[Default Applications]");
                        new_lines.push(line.to_string());
                    } else if in_default_section && trimmed.contains("wazoo.desktop") {
                        continue;
                    } else {
                        new_lines.push(line.to_string());
                    }
                }
                let _ = std::fs::write(&mimeapps_path, new_lines.join("\n") + "\n");
            }
        }
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn unset_as_default_video_player() -> Result<(), String> {
    Ok(())
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

/**
 * Detaches the process from the controlling terminal / console.
 *
 * Spawns a detached grandchild process in a new session with redirected standard I/O,
 * allowing the launching console to be closed without terminating the application.
 */
#[cfg(unix)]
pub fn detach_from_console() {
    if cfg!(test) {
        return;
    }

    unsafe {
        // First fork: creates a child process
        let pid = libc::fork();
        if pid < 0 {
            return;
        }
        if pid > 0 {
            // Parent process exits immediately with success status (0),
            // freeing the console and returning the shell prompt.
            libc::_exit(0);
        }

        // Child process: create a new session and become session leader
        if libc::setsid() < 0 {
            // setsid failed, continue
        }

        // Ignore SIGHUP so terminal closure doesn't kill the child
        libc::signal(libc::SIGHUP, libc::SIG_IGN);

        // Second fork: ensures child is not a session leader,
        // so it cannot acquire a controlling terminal.
        let pid2 = libc::fork();
        if pid2 < 0 {
            return;
        }
        if pid2 > 0 {
            libc::_exit(0);
        }

        // Redirect standard I/O file descriptors (0, 1, 2) to /dev/null
        let devnull = libc::open(b"/dev/null\0".as_ptr() as *const libc::c_char, libc::O_RDWR);
        if devnull >= 0 {
            libc::dup2(devnull, libc::STDIN_FILENO);
            libc::dup2(devnull, libc::STDOUT_FILENO);
            libc::dup2(devnull, libc::STDERR_FILENO);
            if devnull > 2 {
                libc::close(devnull);
            }
        }
    }
}

#[cfg(windows)]
pub fn detach_from_console() {
    if cfg!(test) {
        return;
    }
    extern "system" {
        fn FreeConsole() -> i32;
    }
    unsafe {
        FreeConsole();
    }
}

#[cfg(not(any(unix, windows)))]
pub fn detach_from_console() {}

