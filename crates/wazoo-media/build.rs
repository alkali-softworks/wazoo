/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Native Build Script
 *
 * Configures library search paths and embedding for libmpv.
 * On Windows, downloads prebuilt libmpv binaries if needed, and pre-compresses
 * libmpv-2.dll so that it is embedded directly into the executable for single-file distribution.
 */

fn main() {
    println!("cargo::rustc-check-cfg=cfg(wazoo_embed_mpv)");

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-search=native=/opt/homebrew/lib");
        println!("cargo:rustc-link-search=native=/usr/local/lib");
    }

    #[cfg(target_os = "linux")]
    {
        linux::setup_linux_mpv();
    }

    #[cfg(target_os = "windows")]
    {
        windows::setup_windows_mpv();
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    const MPV_WIN_DOWNLOAD_URL: &str = "https://github.com/shinchiro/mpv-winbuild-cmake/releases/download/20260903/mpv-dev-x86_64-20260903-git-69e63f425a.7z";
    const MPV_WIN_DOWNLOAD_SHA256: &str =
        "fac135c68a35b7639e39d72c0c365104edbaebdea39a0dfdd8c36e8c8e80faef";

    fn verify_sha256(path: &Path, expected_hex: &str) -> bool {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("(Get-FileHash -Algorithm SHA256 '{}').Hash", path.display()),
            ])
            .output()
            .or_else(|_| {
                Command::new("certutil")
                    .args(["-hashfile", path.to_str().unwrap_or_default(), "SHA256"])
                    .output()
            });

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout).to_lowercase();
                return stdout.contains(&expected_hex.to_lowercase());
            }
        }
        false
    }

    pub fn setup_windows_mpv() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
        let root = Path::new(&manifest_dir).join("../..");

        let candidates = [
            PathBuf::from("."),
            root.clone(),
            root.join("target/mpv-win64"),
        ];

        let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string()));
        let mpv_dir = out_dir.join("mpv-win64");
        let _ = std::fs::create_dir_all(&mpv_dir);

        let mut found_dll: Option<PathBuf> = None;
        for dir in &candidates {
            let p = dir.join("libmpv-2.dll");
            if p.exists() {
                found_dll = Some(p);
                break;
            }
        }

        if found_dll.is_none() && mpv_dir.join("libmpv-2.dll").exists() {
            found_dll = Some(mpv_dir.join("libmpv-2.dll"));
        }

        // Auto-download if libmpv files aren't found yet
        if found_dll.is_none() {
            let archive_path = mpv_dir.join("mpv-dev.7z");
            if !archive_path.exists() {
                println!(
                    "cargo:warning=libmpv not found on Windows. Downloading prebuilt mpv-dev binaries..."
                );

                let downloaded = Command::new("curl.exe")
                    .args(["-sL", MPV_WIN_DOWNLOAD_URL, "-o", archive_path.to_str().unwrap()])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                    || Command::new("powershell")
                        .args([
                            "-NoProfile",
                            "-Command",
                            &format!(
                                "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; (New-Object Net.WebClient).DownloadFile('{}', '{}')",
                                MPV_WIN_DOWNLOAD_URL,
                                archive_path.display()
                            ),
                        ])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);

                if !downloaded || !archive_path.exists() {
                    println!("cargo:warning=Failed to auto-download mpv-dev archive.");
                }
            }

            if archive_path.exists() {
                if !verify_sha256(&archive_path, MPV_WIN_DOWNLOAD_SHA256) {
                    let _ = std::fs::remove_file(&archive_path);
                    panic!(
                        "SECURITY ERROR: SHA-256 checksum mismatch for downloaded mpv archive: {}",
                        archive_path.display()
                    );
                }

                let extracted = Command::new("tar.exe")
                    .args([
                        "-xf",
                        archive_path.to_str().unwrap(),
                        "-C",
                        mpv_dir.to_str().unwrap(),
                    ])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                    || Command::new("7z.exe")
                        .args([
                            "x",
                            archive_path.to_str().unwrap(),
                            &format!("-o{}", mpv_dir.display()),
                            "-y",
                        ])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);

                if extracted && mpv_dir.join("libmpv-2.dll").exists() {
                    found_dll = Some(mpv_dir.join("libmpv-2.dll"));
                }
            }
        }

        // Copy runtime DLL to target profile directory for convenience
        if let Some(ref src) = found_dll {
            if let Some(target_profile_dir) = out_dir.ancestors().nth(3) {
                for dll_name in &["libmpv-2.dll", "mpv-2.dll"] {
                    let _ = std::fs::copy(src, target_profile_dir.join(dll_name));
                    let _ = std::fs::copy(src, root.join(dll_name));
                }
            }

            // Pre-compress libmpv-2.dll for embedding into single-file executable
            let deflate_path = out_dir.join("libmpv-2.dll.deflate");
            if !deflate_path.exists()
                || std::fs::metadata(&deflate_path)
                    .map(|m| m.len())
                    .unwrap_or(0)
                    == 0
            {
                println!(
                    "cargo:warning=Compressing libmpv-2.dll for embedding into single-file executable..."
                );
                if let Ok(bytes) = std::fs::read(src) {
                    let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&bytes, 6);
                    let _ = std::fs::write(&deflate_path, &compressed);
                }
            }

            if deflate_path.exists() {
                println!(
                    "cargo:rustc-env=WAZOO_EMBED_MPV_PATH={}",
                    deflate_path.display()
                );
                println!("cargo:rustc-cfg=wazoo_embed_mpv");
            }
        }

        println!("cargo:rustc-link-search=native={}", mpv_dir.display());
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::path::Path;

    pub fn setup_linux_mpv() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
        let root = Path::new(&manifest_dir).join("../..");
        let sysroot = root.join(".sysroot/usr/lib/x86_64-linux-gnu");

        // If sysroot already exists, add it to link search path
        if sysroot.exists() {
            println!("cargo:rustc-link-search=native={}", sysroot.display());
        }
    }
}
