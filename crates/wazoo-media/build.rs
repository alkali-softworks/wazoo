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

    use sha2::{Digest, Sha256};
    use std::io::Read;

    const MPV_WIN_DOWNLOAD_URL: &str = "https://github.com/shinchiro/mpv-winbuild-cmake/releases/download/20260903/mpv-dev-x86_64-20260903-git-69e63f425a.7z";
    const MPV_WIN_DOWNLOAD_SHA256: &str =
        "fac135c68a35b7639e39d72c0c365104edbaebdea39a0dfdd8c36e8c8e80faef";

    fn verify_sha256(path: &Path, expected_hex: &str) -> Result<(), String> {
        let mut file = std::fs::File::open(path)
            .map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
        let metadata = file
            .metadata()
            .map_err(|e| format!("Failed to read metadata for {}: {}", path.display(), e))?;
        let len = metadata.len();

        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536];
        loop {
            let count = file
                .read(&mut buffer)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        let hash = hasher.finalize();
        let actual_hex = format!("{:x}", hash);

        if actual_hex.eq_ignore_ascii_case(expected_hex) {
            Ok(())
        } else {
            let snippet = if len < 4096 {
                std::fs::read_to_string(path).unwrap_or_default()
            } else {
                String::new()
            };
            Err(format!(
                "Checksum mismatch for {}\nSize: {} bytes\nExpected: {}\nActual:   {}\nContent snippet: {}",
                path.display(),
                len,
                expected_hex,
                actual_hex,
                snippet
            ))
        }
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
                    .args([
                        "-fSL",
                        "--retry",
                        "3",
                        "--retry-delay",
                        "2",
                        "-H",
                        "User-Agent: WazooBuild/1.0",
                        MPV_WIN_DOWNLOAD_URL,
                        "-o",
                        archive_path.to_str().unwrap(),
                    ])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                    || Command::new("powershell")
                        .args([
                            "-NoProfile",
                            "-Command",
                            &format!(
                                "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; $wc = New-Object Net.WebClient; $wc.Headers.Add('User-Agent', 'WazooBuild/1.0'); $wc.DownloadFile('{}', '{}')",
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
                if let Err(err) = verify_sha256(&archive_path, MPV_WIN_DOWNLOAD_SHA256) {
                    let _ = std::fs::remove_file(&archive_path);
                    panic!("SECURITY ERROR: {}", err);
                }

                let extracted = Command::new("7z.exe")
                    .args([
                        "x",
                        archive_path.to_str().unwrap(),
                        &format!("-o{}", mpv_dir.display()),
                        "-y",
                    ])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                    || Command::new("tar.exe")
                        .args([
                            "-xf",
                            archive_path.to_str().unwrap(),
                            "-C",
                            mpv_dir.to_str().unwrap(),
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
                let deps_dir = target_profile_dir.join("deps");
                let _ = std::fs::create_dir_all(&deps_dir);
                for dll_name in &["libmpv-2.dll", "mpv-2.dll"] {
                    let _ = std::fs::copy(src, target_profile_dir.join(dll_name));
                    let _ = std::fs::copy(src, deps_dir.join(dll_name));
                    let _ = std::fs::copy(src, root.join(dll_name));
                }

                // If vulkan-1.dll exists on the system or in SDK, copy it alongside libmpv for tests
                let vulkan_candidates = [
                    std::path::PathBuf::from("C:\\Windows\\System32\\vulkan-1.dll"),
                    std::env::var("VULKAN_SDK")
                        .map(|p| std::path::PathBuf::from(p).join("bin\\vulkan-1.dll"))
                        .unwrap_or_default(),
                    std::env::var("VULKAN_SDK")
                        .map(|p| std::path::PathBuf::from(p).join("runtime\\x64\\vulkan-1.dll"))
                        .unwrap_or_default(),
                ];
                for vpath in &vulkan_candidates {
                    if vpath.exists() {
                        let _ = std::fs::copy(vpath, target_profile_dir.join("vulkan-1.dll"));
                        let _ = std::fs::copy(vpath, deps_dir.join("vulkan-1.dll"));
                        let _ = std::fs::copy(vpath, root.join("vulkan-1.dll"));
                        break;
                    }
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
