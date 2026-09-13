/*!
 * ALKALI SOFTWORKS - Wazoo
 * 
 * Windows Resource Build Script
 * 
 * Embeds high-resolution application icon, product metadata, and version
 * information directly into the Windows executable (.exe).
 */

fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("resources/icon.ico");
        res.set("ProductName", "Wazoo");
        res.set("FileDescription", "Wazoo - Ambient Media Engine");
        res.set("LegalCopyright", "Copyright (C) Alkali Softworks");
        let _ = res.compile();
    }
}
