use crate::config::AppConfig;
use std::ffi::OsStr;
use std::iter::once;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use windows_sys::Win32::Foundation::{HWND, POINT};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CheckMenuItem, CreatePopupMenu, DestroyMenu, GetCursorPos, LoadIconW,
    PostMessageW, SetForegroundWindow, SetMenuDefaultItem, TrackPopupMenuEx, HMENU,
    IDI_APPLICATION, MF_CHECKED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED, TPM_BOTTOMALIGN,
    TPM_RIGHTALIGN, WM_APP, WM_NULL,
};

pub const WM_APP_TRAY_MSG: u32 = WM_APP + 1;
pub const WM_APP_RELOAD_CONFIG: u32 = WM_APP + 2;

pub const ID_TRAY_RECENTER: usize = 101;
pub const ID_TRAY_TOGGLE_WINDOW: usize = 102;
pub const ID_TRAY_SETTINGS: usize = 103;
pub const ID_TRAY_TOGGLE_STARTUP: usize = 104;
pub const ID_TRAY_EXIT: usize = 105;

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}

pub fn create_notify_icon_data(hwnd: HWND, cfg: &AppConfig) -> NOTIFYICONDATAW {
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    nid.uCallbackMessage = WM_APP_TRAY_MSG;
    nid.hIcon = unsafe { LoadIconW(0 as _, IDI_APPLICATION) };

    let tip = format!("Recenter Mouse ({})", cfg.hotkey_display());
    let tip_wide = to_wide(&tip);
    let copy_len = tip_wide.len().min(nid.szTip.len() - 1);
    nid.szTip[..copy_len].copy_from_slice(&tip_wide[..copy_len]);

    nid
}

pub fn add_tray_icon(hwnd: HWND, cfg: &AppConfig) {
    let mut nid = create_notify_icon_data(hwnd, cfg);
    unsafe {
        Shell_NotifyIconW(NIM_ADD, &mut nid);
    }
}

pub fn update_tray_icon(hwnd: HWND, cfg: &AppConfig) {
    let mut nid = create_notify_icon_data(hwnd, cfg);
    unsafe {
        Shell_NotifyIconW(NIM_MODIFY, &mut nid);
    }
}

pub fn remove_tray_icon(hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    unsafe {
        Shell_NotifyIconW(NIM_DELETE, &mut nid);
    }
}

pub fn show_tray_menu(hwnd: HWND, cfg: &AppConfig) {
    unsafe {
        let menu: HMENU = CreatePopupMenu();

        let item_recenter = to_wide(&format!("Recenter Now ({})", cfg.hotkey_display()));
        let item_toggle_win = to_wide("Move Active Window");
        let item_settings = to_wide("Settings & Key Bindings...");
        let item_startup = to_wide("Start with Windows");
        let item_exit = to_wide("Exit");

        AppendMenuW(menu, MF_STRING, ID_TRAY_RECENTER, item_recenter.as_ptr());
        // Set recenter as default bold action
        SetMenuDefaultItem(menu, ID_TRAY_RECENTER as u32, 0);

        AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null());
        AppendMenuW(menu, MF_STRING, ID_TRAY_TOGGLE_WINDOW, item_toggle_win.as_ptr());
        CheckMenuItem(
            menu,
            ID_TRAY_TOGGLE_WINDOW as u32,
            if cfg.move_window { MF_CHECKED } else { MF_UNCHECKED },
        );

        AppendMenuW(menu, MF_STRING, ID_TRAY_SETTINGS, item_settings.as_ptr());
        AppendMenuW(menu, MF_STRING, ID_TRAY_TOGGLE_STARTUP, item_startup.as_ptr());
        CheckMenuItem(
            menu,
            ID_TRAY_TOGGLE_STARTUP as u32,
            if cfg.start_with_windows { MF_CHECKED } else { MF_UNCHECKED },
        );

        AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null());
        AppendMenuW(menu, MF_STRING, ID_TRAY_EXIT, item_exit.as_ptr());

        let mut pt: POINT = std::mem::zeroed();
        GetCursorPos(&mut pt);

        // Required Win32 sequence for tray context menus
        SetForegroundWindow(hwnd);
        TrackPopupMenuEx(
            menu,
            TPM_RIGHTALIGN | TPM_BOTTOMALIGN,
            pt.x,
            pt.y,
            hwnd,
            ptr::null(),
        );
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(menu);
    }
}
