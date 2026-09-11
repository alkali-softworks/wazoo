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
        println!("cargo:rustc-link-search=native=.");
    }
}
