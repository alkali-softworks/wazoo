/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Application Entry Point
 *
 * Boots the Wazoo application runtime.
 */

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub fn main() -> iced::Result {
    wazoo_app::run()
}
