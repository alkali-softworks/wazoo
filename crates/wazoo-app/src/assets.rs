/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Static Vector & Icon Assets
 *
 * Embeds static resources including the application icon and Lucide-style vector SVGs
 * for window controls, playback buttons, and audio indicators.
 */

pub static APP_ICON_BYTES: &[u8] = include_bytes!("../resources/icon.png");

// Crisp vector SVGs for window controls matching modern desktop apps
pub static SVG_WINDOW_MINIMIZE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><line x1="0" y1="5" x2="10" y2="5" stroke="#cccccc" stroke-width="1.2"/></svg>"##;
pub static SVG_WINDOW_MAXIMIZE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10" fill="none"><rect x="1.5" y="1.5" width="7" height="7" stroke="#cccccc" stroke-width="1.2"/></svg>"##;
pub static SVG_WINDOW_CLOSE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><line x1="1.5" y1="1.5" x2="8.5" y2="8.5" stroke="#cccccc" stroke-width="1.2"/><line x1="8.5" y1="1.5" x2="1.5" y2="8.5" stroke="#cccccc" stroke-width="1.2"/></svg>"##;

// Lucide-style 24x24 vector playback SVGs for bottom player controls
pub static SVG_PLAYER_PLAY: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#ffffff"><polygon points="6 3 20 12 6 21 6 3"/></svg>"##;
pub static SVG_PLAYER_PAUSE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#ffffff"><rect x="5" y="3" width="4" height="18" rx="1"/><rect x="15" y="3" width="4" height="18" rx="1"/></svg>"##;
pub static SVG_PLAYER_PREV: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#ffffff"><polygon points="19 20 9 12 19 4 19 20"/><rect x="4" y="4" width="3" height="16" rx="1"/></svg>"##;
pub static SVG_PLAYER_NEXT: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#ffffff"><polygon points="5 4 15 12 5 20 5 4"/><rect x="17" y="4" width="3" height="16" rx="1"/></svg>"##;
pub static SVG_PLAYER_VOLUME: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" fill="#ffffff"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14"/></svg>"##;
pub static SVG_PLAYER_MUTE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" fill="#ffffff"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/></svg>"##;
pub static SVG_PLAYER_SHUFFLE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 18h1.4c1.3 0 2.5-.6 3.3-1.7l6.1-8.6c.7-1.1 2-1.7 3.3-1.7H22"/><path d="m18 2 4 4-4 4"/><path d="M2 6h1.9c1.5 0 2.9.9 3.6 2.2"/><path d="M22 18h-5.9c-1.3 0-2.6-.7-3.3-1.8l-.5-.8"/><path d="m18 14 4 4-4 4"/></svg>"##;
pub static SVG_PLAYER_REPEAT: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m17 2 4 4-4 4"/><path d="M3 11v-1a4 4 0 0 1 4-4h14"/><path d="m7 22-4-4 4-4"/><path d="M21 13v1a4 4 0 0 1-4 4H3"/></svg>"##;
pub static SVG_PLAYER_FLIP: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="#42b883" stroke="#42b883" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>"##;
