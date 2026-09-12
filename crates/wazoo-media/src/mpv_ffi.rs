/**
 * ALKALI SOFTWORKS - Wazoo
 * 
 * libmpv Foreign Function Interface
 * 
 * Declares raw C bindings and data structures for interacting with the libmpv client API,
 * render contexts, and OpenGL/software frame extraction.
 */

use std::ffi::{c_char, c_double, c_int, c_void};

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

#[link(name = "mpv")]
extern "C" {
    pub fn mpv_create() -> *mut MpvHandle;
    pub fn mpv_initialize(ctx: *mut MpvHandle) -> c_int;
    pub fn mpv_terminate_destroy(ctx: *mut MpvHandle);
    pub fn mpv_set_option_string(ctx: *mut MpvHandle, name: *const c_char, data: *const c_char) -> c_int;
    pub fn mpv_set_property(
        ctx: *mut MpvHandle,
        name: *const c_char,
        format: c_int,
        data: *mut c_void,
    ) -> c_int;
    pub fn mpv_set_property_string(ctx: *mut MpvHandle, name: *const c_char, data: *const c_char) -> c_int;
    pub fn mpv_get_property(
        ctx: *mut MpvHandle,
        name: *const c_char,
        format: c_int,
        data: *mut c_void,
    ) -> c_int;
    pub fn mpv_get_property_string(ctx: *mut MpvHandle, name: *const c_char) -> *mut c_char;
    pub fn mpv_free(data: *mut c_void);
    pub fn mpv_command(ctx: *mut MpvHandle, args: *mut *const c_char) -> c_int;
    pub fn mpv_command_string(ctx: *mut MpvHandle, args: *const c_char) -> c_int;
    pub fn mpv_wait_event(ctx: *mut MpvHandle, timeout: c_double) -> *mut MpvEvent;

    pub fn mpv_render_context_create(
        res: *mut *mut MpvRenderContext,
        mpv: *mut MpvHandle,
        params: *mut MpvRenderParam,
    ) -> c_int;
    pub fn mpv_render_context_render(ctx: *mut MpvRenderContext, params: *mut MpvRenderParam) -> c_int;
    pub fn mpv_render_context_report_swap(ctx: *mut MpvRenderContext);
    pub fn mpv_render_context_update(ctx: *mut MpvRenderContext) -> u64;
    pub fn mpv_render_context_free(ctx: *mut MpvRenderContext);
}
