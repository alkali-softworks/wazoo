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
    let result = wazoo_app::run();
    let exit_code = if result.is_err() { 1 } else { 0 };
    std::process::exit(exit_code);
}
