/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * libmpv Foreign Function Interface (Dynamic Loading)
 *
 * Dynamically resolves the libmpv client API and render context symbols at runtime
 * across Windows, Linux, and macOS without hard link-time dependencies.
 * On Windows, supports automatic unpack and loading of the embedded compressed
 * libmpv-2.dll for a completely self-contained single-executable experience.
 */

#![allow(clippy::missing_safety_doc)]

use libloading::Library;
use std::ffi::{c_char, c_double, c_int, c_void};
use std::sync::OnceLock;

#[repr(C)]
pub struct MpvHandle(c_void);

#[repr(C)]
pub struct MpvRenderContext(c_void);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MpvRenderParam {
    pub type_: c_int,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct MpvEvent {
    pub event_id: c_int,
    pub error: c_int,
    pub reply_userdata: u64,
    pub data: *mut c_void,
}

// Event IDs
pub const MPV_EVENT_NONE: c_int = 0;
pub const MPV_EVENT_SHUTDOWN: c_int = 1;
pub const MPV_EVENT_LOG_MESSAGE: c_int = 2;
pub const MPV_EVENT_GET_PROPERTY_REPLY: c_int = 3;
pub const MPV_EVENT_SET_PROPERTY_REPLY: c_int = 4;
pub const MPV_EVENT_COMMAND_REPLY: c_int = 5;
pub const MPV_EVENT_START_FILE: c_int = 6;
pub const MPV_EVENT_END_FILE: c_int = 7;
pub const MPV_EVENT_FILE_LOADED: c_int = 8;
pub const MPV_EVENT_IDLE: c_int = 11;
pub const MPV_EVENT_TRACKS_CHANGED: c_int = 18;
pub const MPV_EVENT_VIDEO_RECONFIG: c_int = 19;
pub const MPV_EVENT_PLAYBACK_RESTART: c_int = 21;
pub const MPV_EVENT_PROPERTY_CHANGE: c_int = 22;

// Render param types
pub const MPV_RENDER_PARAM_INVALID: c_int = 0;
pub const MPV_RENDER_PARAM_API_TYPE: c_int = 1;
pub const MPV_RENDER_PARAM_BLOCK_FOR_TARGET_TIME: c_int = 12;
pub const MPV_RENDER_PARAM_SW_SIZE: c_int = 17;
pub const MPV_RENDER_PARAM_SW_FORMAT: c_int = 18;
pub const MPV_RENDER_PARAM_SW_STRIDE: c_int = 19;
pub const MPV_RENDER_PARAM_SW_POINTER: c_int = 20;

pub const MPV_RENDER_UPDATE_FRAME: u64 = 1 << 0;

// Format types
pub const MPV_FORMAT_NONE: c_int = 0;
pub const MPV_FORMAT_STRING: c_int = 1;
pub const MPV_FORMAT_OSD_STRING: c_int = 2;
pub const MPV_FORMAT_FLAG: c_int = 3;
pub const MPV_FORMAT_INT64: c_int = 4;
pub const MPV_FORMAT_DOUBLE: c_int = 5;
pub const MPV_FORMAT_NODE: c_int = 6;

pub struct MpvApi {
    pub mpv_create: unsafe extern "C" fn() -> *mut MpvHandle,
    pub mpv_initialize: unsafe extern "C" fn(ctx: *mut MpvHandle) -> c_int,
    pub mpv_terminate_destroy: unsafe extern "C" fn(ctx: *mut MpvHandle),
    pub mpv_set_option_string: unsafe extern "C" fn(
        ctx: *mut MpvHandle,
        name: *const c_char,
        data: *const c_char,
    ) -> c_int,
    pub mpv_set_property: unsafe extern "C" fn(
        ctx: *mut MpvHandle,
        name: *const c_char,
        format: c_int,
        data: *mut c_void,
    ) -> c_int,
    pub mpv_set_property_string: unsafe extern "C" fn(
        ctx: *mut MpvHandle,
        name: *const c_char,
        data: *const c_char,
    ) -> c_int,
    pub mpv_get_property: unsafe extern "C" fn(
        ctx: *mut MpvHandle,
        name: *const c_char,
        format: c_int,
        data: *mut c_void,
    ) -> c_int,
    pub mpv_get_property_string:
        unsafe extern "C" fn(ctx: *mut MpvHandle, name: *const c_char) -> *mut c_char,
    pub mpv_free: unsafe extern "C" fn(data: *mut c_void),
    pub mpv_command: unsafe extern "C" fn(ctx: *mut MpvHandle, args: *mut *const c_char) -> c_int,
    pub mpv_command_string: unsafe extern "C" fn(ctx: *mut MpvHandle, args: *const c_char) -> c_int,
    pub mpv_wait_event:
        unsafe extern "C" fn(ctx: *mut MpvHandle, timeout: c_double) -> *mut MpvEvent,

    pub mpv_render_context_create: unsafe extern "C" fn(
        res: *mut *mut MpvRenderContext,
        mpv: *mut MpvHandle,
        params: *mut MpvRenderParam,
    ) -> c_int,
    pub mpv_render_context_render:
        unsafe extern "C" fn(ctx: *mut MpvRenderContext, params: *mut MpvRenderParam) -> c_int,
    pub mpv_render_context_report_swap: unsafe extern "C" fn(ctx: *mut MpvRenderContext),
    pub mpv_render_context_update: unsafe extern "C" fn(ctx: *mut MpvRenderContext) -> u64,
    pub mpv_render_context_free: unsafe extern "C" fn(ctx: *mut MpvRenderContext),
    _lib: Library,
}

static MPV_API: OnceLock<Option<MpvApi>> = OnceLock::new();

pub fn get_mpv_api() -> Option<&'static MpvApi> {
    MPV_API.get_or_init(load_mpv_api).as_ref()
}

unsafe fn load_symbols(lib: Library) -> Option<MpvApi> {
    unsafe {
        macro_rules! get_sym {
            ($name:ident) => {
                match lib.get(concat!(stringify!($name), "\0").as_bytes()) {
                    Ok(sym) => *sym,
                    Err(err) => {
                        log::error!(
                            "Failed to resolve libmpv symbol {}: {}",
                            stringify!($name),
                            err
                        );
                        return None;
                    }
                }
            };
        }

        Some(MpvApi {
            mpv_create: get_sym!(mpv_create),
            mpv_initialize: get_sym!(mpv_initialize),
            mpv_terminate_destroy: get_sym!(mpv_terminate_destroy),
            mpv_set_option_string: get_sym!(mpv_set_option_string),
            mpv_set_property: get_sym!(mpv_set_property),
            mpv_set_property_string: get_sym!(mpv_set_property_string),
            mpv_get_property: get_sym!(mpv_get_property),
            mpv_get_property_string: get_sym!(mpv_get_property_string),
            mpv_free: get_sym!(mpv_free),
            mpv_command: get_sym!(mpv_command),
            mpv_command_string: get_sym!(mpv_command_string),
            mpv_wait_event: get_sym!(mpv_wait_event),
            mpv_render_context_create: get_sym!(mpv_render_context_create),
            mpv_render_context_render: get_sym!(mpv_render_context_render),
            mpv_render_context_report_swap: get_sym!(mpv_render_context_report_swap),
            mpv_render_context_update: get_sym!(mpv_render_context_update),
            mpv_render_context_free: get_sym!(mpv_render_context_free),
            _lib: lib,
        })
    }
}

#[cfg(target_os = "windows")]
unsafe fn set_dll_directory(dir: &std::path::Path) {
    use std::os::windows::ffi::OsStrExt;
    let mut wide: Vec<u16> = dir.as_os_str().encode_wide().collect();
    wide.push(0);
    unsafe extern "system" {
        fn SetDllDirectoryW(lpPathName: *const u16) -> i32;
    }
    unsafe {
        let _ = SetDllDirectoryW(wide.as_ptr());
    }
}

#[cfg(target_os = "windows")]
fn load_windows_mpv() -> Option<MpvApi> {
    let dll_names = ["libmpv-2.dll", "mpv-2.dll", "mpv.dll"];

    let try_load = |path: &std::path::Path| -> Option<MpvApi> {
        if !path.exists() {
            return None;
        }
        if let Some(parent) = path.parent() {
            unsafe {
                set_dll_directory(parent);
            }
        }
        match unsafe { Library::new(path) } {
            Ok(lib) => {
                if let Some(api) = unsafe { load_symbols(lib) } {
                    log::info!("Loaded libmpv from: {}", path.display());
                    return Some(api);
                }
            }
            Err(err) => {
                log::warn!(
                    "Found libmpv candidate at {} but failed to load (check missing dependencies like vulkan-1.dll): {}",
                    path.display(),
                    err
                );
            }
        }
        None
    };

    // 1. Check beside executable and in parent directory (for cargo test binaries in target/debug/deps)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in &dll_names {
                if let Some(api) = try_load(&dir.join(name)) {
                    return Some(api);
                }
            }
            if let Some(parent_dir) = dir.parent() {
                for name in &dll_names {
                    if let Some(api) = try_load(&parent_dir.join(name)) {
                        return Some(api);
                    }
                }
            }
        }
    }

    // 2. Check current working directory & known workspace directories
    if let Ok(cwd) = std::env::current_dir() {
        let search_dirs = [
            cwd.clone(),
            cwd.join("target/mpv-win64"),
            cwd.join("target/debug"),
            cwd.join("target/debug/deps"),
            cwd.join("target/release"),
            cwd.join("target/release/deps"),
        ];
        for search_dir in search_dirs {
            for name in &dll_names {
                if let Some(api) = try_load(&search_dir.join(name)) {
                    return Some(api);
                }
            }
        }
    }

    // 3. Check local app data cache & fallback temp cache
    let fallback_data_dirs = [
        directories::ProjectDirs::from("com", "Alkali Softworks", "Wazoo")
            .map(|p| p.data_local_dir().to_path_buf()),
        Some(std::env::temp_dir().join("wazoo-mpv-cache")),
    ];

    for data_dir_opt in fallback_data_dirs {
        if let Some(data_dir) = data_dir_opt {
            let bin_dir = data_dir.join("bin");
            let cached_dll = bin_dir.join("libmpv-2.dll");
            if let Some(api) = try_load(&cached_dll) {
                return Some(api);
            }

            // Extract embedded compressed DLL if present
            #[cfg(wazoo_embed_mpv)]
            {
                static EMBEDDED_DLL: &[u8] = include_bytes!(env!("WAZOO_EMBED_MPV_PATH"));
                let _ = std::fs::create_dir_all(&bin_dir);
                log::info!("Unpacking embedded libmpv to {}", cached_dll.display());
                if let Ok(decompressed) = miniz_oxide::inflate::decompress_to_vec_zlib(EMBEDDED_DLL)
                {
                    if std::fs::write(&cached_dll, &decompressed).is_ok() {
                        if let Ok(lib) = unsafe { Library::new(&cached_dll) } {
                            if let Some(api) = unsafe { load_symbols(lib) } {
                                log::info!(
                                    "Loaded unpacked embedded libmpv: {}",
                                    cached_dll.display()
                                );
                                return Some(api);
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. System DLL search
    for name in &dll_names {
        if let Ok(lib) = unsafe { Library::new(name) } {
            if let Some(api) = unsafe { load_symbols(lib) } {
                log::info!("Loaded system libmpv: {}", name);
                return Some(api);
            }
        }
    }

    None
}

#[cfg(target_os = "linux")]
fn load_linux_mpv() -> Option<MpvApi> {
    // 1. Beside executable or in ./lib
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in &["libmpv.so.2", "libmpv.so"] {
                let path = dir.join(name);
                if path.exists() {
                    if let Ok(lib) = unsafe { Library::new(&path) } {
                        if let Some(api) = unsafe { load_symbols(lib) } {
                            log::info!(
                                "Loaded libmpv from executable directory: {}",
                                path.display()
                            );
                            return Some(api);
                        }
                    }
                }
                let lib_path = dir.join("lib").join(name);
                if lib_path.exists() {
                    if let Ok(lib) = unsafe { Library::new(&lib_path) } {
                        if let Some(api) = unsafe { load_symbols(lib) } {
                            log::info!(
                                "Loaded libmpv from lib subdirectory: {}",
                                lib_path.display()
                            );
                            return Some(api);
                        }
                    }
                }
            }
        }
    }

    // 2. Standard system locations
    let candidates = [
        "libmpv.so.2",
        "libmpv.so",
        "/usr/lib/x86_64-linux-gnu/libmpv.so.2",
        "/usr/lib64/libmpv.so.2",
        "/usr/lib/libmpv.so.2",
        "/usr/local/lib/libmpv.so.2",
        "/usr/lib/x86_64-linux-gnu/libmpv.so",
        "/usr/lib64/libmpv.so",
        "/usr/lib/libmpv.so",
        "/usr/local/lib/libmpv.so",
    ];

    for c in &candidates {
        if let Ok(lib) = unsafe { Library::new(c) } {
            if let Some(api) = unsafe { load_symbols(lib) } {
                log::info!("Loaded libmpv from candidate: {}", c);
                return Some(api);
            }
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn load_macos_mpv() -> Option<MpvApi> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in &["libmpv.2.dylib", "libmpv.dylib"] {
                let path = dir.join(name);
                if path.exists() {
                    if let Ok(lib) = unsafe { Library::new(&path) } {
                        if let Some(api) = unsafe { load_symbols(lib) } {
                            log::info!(
                                "Loaded libmpv from executable directory: {}",
                                path.display()
                            );
                            return Some(api);
                        }
                    }
                }
                let fw = dir.join("../Frameworks").join(name);
                if fw.exists() {
                    if let Ok(lib) = unsafe { Library::new(&fw) } {
                        if let Some(api) = unsafe { load_symbols(lib) } {
                            log::info!("Loaded libmpv from Frameworks: {}", fw.display());
                            return Some(api);
                        }
                    }
                }
            }
        }
    }

    let candidates = [
        "/opt/homebrew/lib/libmpv.2.dylib",
        "/opt/homebrew/lib/libmpv.dylib",
        "/usr/local/lib/libmpv.2.dylib",
        "/usr/local/lib/libmpv.dylib",
        "libmpv.2.dylib",
        "libmpv.dylib",
    ];

    for c in &candidates {
        if let Ok(lib) = unsafe { Library::new(c) } {
            if let Some(api) = unsafe { load_symbols(lib) } {
                log::info!("Loaded libmpv on macOS from candidate: {}", c);
                return Some(api);
            }
        }
    }

    None
}

fn load_mpv_api() -> Option<MpvApi> {
    #[cfg(target_os = "windows")]
    {
        if let Some(api) = load_windows_mpv() {
            return Some(api);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(api) = load_linux_mpv() {
            return Some(api);
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(api) = load_macos_mpv() {
            return Some(api);
        }
    }

    for fallback in &["libmpv.so.2", "libmpv.so", "mpv", "libmpv"] {
        if let Ok(lib) = unsafe { Library::new(fallback) } {
            if let Some(api) = unsafe { load_symbols(lib) } {
                log::info!("Loaded libmpv via generic fallback: {}", fallback);
                return Some(api);
            }
        }
    }

    log::error!("CRITICAL: Unable to locate or dynamically load libmpv runtime library");
    None
}

// ---------------------------------------------------------------------------
// Public API Functions (matching original FFI declarations exactly)
// ---------------------------------------------------------------------------

pub unsafe fn mpv_create() -> *mut MpvHandle {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_create)()
        } else {
            std::ptr::null_mut()
        }
    }
}

pub unsafe fn mpv_initialize(ctx: *mut MpvHandle) -> c_int {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_initialize)(ctx)
        } else {
            -1
        }
    }
}

pub unsafe fn mpv_terminate_destroy(ctx: *mut MpvHandle) {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_terminate_destroy)(ctx);
        }
    }
}

pub unsafe fn mpv_set_option_string(
    ctx: *mut MpvHandle,
    name: *const c_char,
    data: *const c_char,
) -> c_int {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_set_option_string)(ctx, name, data)
        } else {
            -1
        }
    }
}

pub unsafe fn mpv_set_property(
    ctx: *mut MpvHandle,
    name: *const c_char,
    format: c_int,
    data: *mut c_void,
) -> c_int {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_set_property)(ctx, name, format, data)
        } else {
            -1
        }
    }
}

pub unsafe fn mpv_set_property_string(
    ctx: *mut MpvHandle,
    name: *const c_char,
    data: *const c_char,
) -> c_int {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_set_property_string)(ctx, name, data)
        } else {
            -1
        }
    }
}

pub unsafe fn mpv_get_property(
    ctx: *mut MpvHandle,
    name: *const c_char,
    format: c_int,
    data: *mut c_void,
) -> c_int {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_get_property)(ctx, name, format, data)
        } else {
            -1
        }
    }
}

pub unsafe fn mpv_get_property_string(ctx: *mut MpvHandle, name: *const c_char) -> *mut c_char {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_get_property_string)(ctx, name)
        } else {
            std::ptr::null_mut()
        }
    }
}

pub unsafe fn mpv_free(data: *mut c_void) {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_free)(data);
        }
    }
}

pub unsafe fn mpv_command(ctx: *mut MpvHandle, args: *mut *const c_char) -> c_int {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_command)(ctx, args)
        } else {
            -1
        }
    }
}

pub unsafe fn mpv_command_string(ctx: *mut MpvHandle, args: *const c_char) -> c_int {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_command_string)(ctx, args)
        } else {
            -1
        }
    }
}

pub unsafe fn mpv_wait_event(ctx: *mut MpvHandle, timeout: c_double) -> *mut MpvEvent {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_wait_event)(ctx, timeout)
        } else {
            std::ptr::null_mut()
        }
    }
}

pub unsafe fn mpv_render_context_create(
    res: *mut *mut MpvRenderContext,
    mpv: *mut MpvHandle,
    params: *mut MpvRenderParam,
) -> c_int {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_render_context_create)(res, mpv, params)
        } else {
            -1
        }
    }
}

pub unsafe fn mpv_render_context_render(
    ctx: *mut MpvRenderContext,
    params: *mut MpvRenderParam,
) -> c_int {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_render_context_render)(ctx, params)
        } else {
            -1
        }
    }
}

pub unsafe fn mpv_render_context_report_swap(ctx: *mut MpvRenderContext) {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_render_context_report_swap)(ctx);
        }
    }
}

pub unsafe fn mpv_render_context_update(ctx: *mut MpvRenderContext) -> u64 {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_render_context_update)(ctx)
        } else {
            0
        }
    }
}

pub unsafe fn mpv_render_context_free(ctx: *mut MpvRenderContext) {
    unsafe {
        if let Some(api) = get_mpv_api() {
            (api.mpv_render_context_free)(ctx);
        }
    }
}
