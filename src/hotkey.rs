use crate::config::AppConfig;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN,
    VK_BACK, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1, VK_F10, VK_F11, VK_F12, VK_F2, VK_F3,
    VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_HOME, VK_INSERT, VK_LEFT, VK_NEXT, VK_PRIOR,
    VK_RETURN, VK_RIGHT, VK_SPACE, VK_TAB, VK_UP,
};

pub const RECENTER_HOTKEY_ID: i32 = 1001;

/// Convert a key string (e.g. "C", "F12", "Space") into a Windows Virtual Key code.
pub fn parse_key_str(key: &str) -> Option<u32> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Check single character
    if trimmed.len() == 1 {
        let ch = trimmed.chars().next().unwrap().to_ascii_uppercase();
        if ch.is_ascii_alphanumeric() {
            return Some(ch as u32);
        }
    }

    // Check special key names
    let upper = trimmed.to_ascii_uppercase();
    match upper.as_str() {
        "F1" => Some(VK_F1 as u32),
        "F2" => Some(VK_F2 as u32),
        "F3" => Some(VK_F3 as u32),
        "F4" => Some(VK_F4 as u32),
        "F5" => Some(VK_F5 as u32),
        "F6" => Some(VK_F6 as u32),
        "F7" => Some(VK_F7 as u32),
        "F8" => Some(VK_F8 as u32),
        "F9" => Some(VK_F9 as u32),
        "F10" => Some(VK_F10 as u32),
        "F11" => Some(VK_F11 as u32),
        "F12" => Some(VK_F12 as u32),
        "SPACE" => Some(VK_SPACE as u32),
        "ENTER" | "RETURN" => Some(VK_RETURN as u32),
        "TAB" => Some(VK_TAB as u32),
        "ESC" | "ESCAPE" => Some(VK_ESCAPE as u32),
        "BACKSPACE" | "BACK" => Some(VK_BACK as u32),
        "INSERT" => Some(VK_INSERT as u32),
        "DELETE" | "DEL" => Some(VK_DELETE as u32),
        "HOME" => Some(VK_HOME as u32),
        "END" => Some(VK_END as u32),
        "PAGEUP" | "PGUP" => Some(VK_PRIOR as u32),
        "PAGEDOWN" | "PGDN" => Some(VK_NEXT as u32),
        "LEFT" => Some(VK_LEFT as u32),
        "UP" => Some(VK_UP as u32),
        "RIGHT" => Some(VK_RIGHT as u32),
        "DOWN" => Some(VK_DOWN as u32),
        _ => None,
    }
}

/// Convert AppConfig modifiers into Win32 modifier bitmask.
pub fn get_modifiers_mask(cfg: &AppConfig) -> u32 {
    let mut mods = MOD_NOREPEAT;
    if cfg.ctrl {
        mods |= MOD_CONTROL;
    }
    if cfg.alt {
        mods |= MOD_ALT;
    }
    if cfg.shift {
        mods |= MOD_SHIFT;
    }
    if cfg.win {
        mods |= MOD_WIN;
    }
    mods
}

/// Register the configured hotkey to the specified window handle.
pub fn register_app_hotkey(hwnd: HWND, cfg: &AppConfig) -> Result<(), String> {
    let vk = parse_key_str(&cfg.key)
        .ok_or_else(|| format!("Unknown or invalid key name: '{}'", cfg.key))?;
    let mods = get_modifiers_mask(cfg);

    unsafe {
        // Unregister existing first to prevent conflicts
        UnregisterHotKey(hwnd, RECENTER_HOTKEY_ID);

        let success = RegisterHotKey(hwnd, RECENTER_HOTKEY_ID, mods, vk);
        if success == 0 {
            return Err(format!(
                "Failed to register hotkey '{}'. It might already be in use by another application.",
                cfg.hotkey_display()
            ));
        }
    }
    Ok(())
}

/// Unregister the hotkey from the specified window handle.
pub fn unregister_app_hotkey(hwnd: HWND) {
    unsafe {
        UnregisterHotKey(hwnd, RECENTER_HOTKEY_ID);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_alphanumeric_keys() {
        assert_eq!(parse_key_str("c"), Some('C' as u32));
        assert_eq!(parse_key_str("C"), Some('C' as u32));
        assert_eq!(parse_key_str("x"), Some('X' as u32));
        assert_eq!(parse_key_str("1"), Some('1' as u32));
    }

    #[test]
    fn test_parse_special_keys() {
        assert_eq!(parse_key_str("F12"), Some(VK_F12 as u32));
        assert_eq!(parse_key_str("space"), Some(VK_SPACE as u32));
        assert_eq!(parse_key_str("Enter"), Some(VK_RETURN as u32));
        assert_eq!(parse_key_str("ESC"), Some(VK_ESCAPE as u32));
    }

    #[test]
    fn test_modifiers_mask() {
        let mut cfg = AppConfig::default();
        cfg.ctrl = true;
        cfg.alt = true;
        cfg.shift = false;
        cfg.win = false;

        let mask = get_modifiers_mask(&cfg);
        assert_eq!(mask, MOD_NOREPEAT | MOD_CONTROL | MOD_ALT);
    }
}
