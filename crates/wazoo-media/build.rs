/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Native Build Script
 * 
 * Detects libmpv dependencies on Linux/macOS and configured search paths.
 * On Windows, automatically detects or downloads prebuilt libmpv binaries
 * so builds succeed out-of-the-box without manual file hunting.
 */

fn main() {
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-search=native=/opt/homebrew/lib");
        println!("cargo:rustc-link-search=native=/usr/local/lib");
    }

    #[cfg(target_os = "linux")]
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
        let sysroot = std::path::Path::new(&manifest_dir).join("../../.sysroot/usr/lib/x86_64-linux-gnu");
        if sysroot.exists() {
            println!("cargo:rustc-link-search=native={}", sysroot.display());
        }
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

    const MPV_WIN_DOWNLOAD_URL: &str =
        "https://github.com/shinchiro/mpv-winbuild-cmake/releases/download/20260903/mpv-dev-x86_64-20260903-git-69e63f425a.7z";

    pub fn setup_windows_mpv() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
        let root = Path::new(&manifest_dir).join("../..");

        // 1. Check if user or environment already provided an mpv library
        let candidates = [
            PathBuf::from("."),
            root.clone(),
            root.join("target/mpv-win64"),
        ];

        for dir in &candidates {
            if dir.join("mpv.lib").exists() || dir.join("libmpv.dll.a").exists() {
                println!("cargo:rustc-link-search=native={}", dir.display());
                return;
            }
        }

        // Check VCPKG_ROOT if set
        if let Ok(vcpkg_root) = std::env::var("VCPKG_ROOT") {
            let vcpkg_lib = Path::new(&vcpkg_root).join("installed/x64-windows/lib");
            if vcpkg_lib.join("mpv.lib").exists() {
                println!("cargo:rustc-link-search=native={}", vcpkg_lib.display());
                return;
            }
        }

        // 2. Not found: auto-download and extract mpv-dev package into target
        let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string()));
        let mpv_dir = out_dir.join("mpv-win64");
        let _ = std::fs::create_dir_all(&mpv_dir);

        let archive_path = mpv_dir.join("mpv-dev.7z");

        // Download if libmpv files aren't extracted yet
        if !mpv_dir.join("libmpv-2.dll").exists() && !mpv_dir.join("mpv.lib").exists() {
            println!("cargo:warning=libmpv not found on Windows. Downloading prebuilt mpv-dev binaries...");

            // Download using curl.exe or powershell.exe (both built into Windows 10/11)
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

            if downloaded && archive_path.exists() {
                // Extract using tar.exe (bsdtar included with Windows 10/11) or 7z.exe
                let extracted = Command::new("tar.exe")
                    .args(["-xf", archive_path.to_str().unwrap(), "-C", mpv_dir.to_str().unwrap()])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                    || Command::new("7z.exe")
                        .args(["x", archive_path.to_str().unwrap(), &format!("-o{}", mpv_dir.display()), "-y"])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);

                if extracted {
                    // Create mpv.lib from libmpv.dll.a for MSVC linker
                    let dll_a = mpv_dir.join("libmpv.dll.a");
                    let mpv_lib = mpv_dir.join("mpv.lib");
                    if dll_a.exists() && !mpv_lib.exists() {
                        let _ = std::fs::copy(&dll_a, &mpv_lib);
                    }

                    // Copy runtime DLL to target profile directory (target/debug or target/release)
                    if let Some(target_profile_dir) = out_dir.ancestors().nth(3) {
                        for dll_name in &["libmpv-2.dll", "mpv-2.dll"] {
                            let src = mpv_dir.join("libmpv-2.dll");
                            if src.exists() {
                                let _ = std::fs::copy(&src, target_profile_dir.join(dll_name));
                                let _ = std::fs::copy(&src, root.join(dll_name));
                            }
                        }
                    }
                }
            }
        }

        println!("cargo:rustc-link-search=native={}", mpv_dir.display());
    }
}
