use crate::config::AppConfig;
use crate::hotkey::parse_key_str;
use crate::startup::set_startup_enabled;
use std::cell::RefCell;
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{GetStockObject, COLOR_BTNFACE, DEFAULT_GUI_FONT, HFONT};
use windows_sys::Win32::UI::Controls::{BST_CHECKED, BST_UNCHECKED};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetSystemMetrics, MessageBoxW,
    RegisterClassExW, SendMessageW, SetForegroundWindow, ShowWindow, BM_GETCHECK, BM_SETCHECK,
    BS_AUTOCHECKBOX, BS_DEFPUSHBUTTON, BS_GROUPBOX, BS_PUSHBUTTON, CBS_DROPDOWNLIST, CB_ADDSTRING,
    CB_GETCURSEL, CB_SETCURSEL, CS_HREDRAW, CS_VREDRAW, HMENU, MB_ICONERROR, MB_OK,
    SM_CXSCREEN, SM_CYSCREEN, SW_SHOW, WM_CLOSE, WM_COMMAND, WM_DESTROY, WM_SETFONT,
    WNDCLASSEXW, WS_CAPTION, WS_CHILD, WS_EX_DLGMODALFRAME, WS_MINIMIZEBOX, WS_OVERLAPPED,
    WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};

const ID_CHK_CTRL: isize = 201;
const ID_CHK_ALT: isize = 202;
const ID_CHK_SHIFT: isize = 203;
const ID_CHK_WIN: isize = 204;
const ID_CMB_KEY: isize = 205;
const ID_CHK_MOVE_WINDOW: isize = 206;
const ID_CHK_STARTUP: isize = 207;
const ID_BTN_SAVE: isize = 208;
const ID_BTN_CANCEL: isize = 209;

const KEYS_LIST: &[&str] = &[
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R",
    "S", "T", "U", "V", "W", "X", "Y", "Z", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
    "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "Space",
    "Enter", "Tab", "Esc", "Backspace", "Insert", "Delete", "Home", "End", "PageUp", "PageDown",
];

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}

pub struct SettingsDialog {
    pub hwnd: HWND,
    chk_ctrl: HWND,
    chk_alt: HWND,
    chk_shift: HWND,
    chk_win: HWND,
    cmb_key: HWND,
    chk_move_window: HWND,
    chk_startup: HWND,
    parent_notify_hwnd: HWND,
}

thread_local! {
    static SETTINGS_STATE: RefCell<Option<SettingsDialog>> = RefCell::new(None);
}

pub fn show_settings_window(parent_notify_hwnd: HWND) {
    // If settings window already exists, bring it to front
    let mut already_open = false;
    SETTINGS_STATE.with(|cell| {
        if let Some(ref dlg) = *cell.borrow() {
            unsafe {
                ShowWindow(dlg.hwnd, SW_SHOW);
                SetForegroundWindow(dlg.hwnd);
            }
            already_open = true;
        }
    });

    if already_open {
        return;
    }

    let class_name = to_wide("RecenterMouseSettingsWindow");
    let window_title = to_wide("Recenter Mouse - Settings & Key Bindings");

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_COMMAND => {
                let id = (wparam & 0xFFFF) as isize;
                if id == ID_BTN_SAVE {
                    handle_save(hwnd);
                    return 0;
                } else if id == ID_BTN_CANCEL {
                    DestroyWindow(hwnd);
                    return 0;
                }
            }
            WM_CLOSE => {
                DestroyWindow(hwnd);
                return 0;
            }
            WM_DESTROY => {
                SETTINGS_STATE.with(|cell| {
                    *cell.borrow_mut() = None;
                });
                return 0;
            }
            _ => {}
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: 0 as _,
        hIcon: crate::tray::load_app_icon(false),
        hCursor: 0 as _,
        hbrBackground: (COLOR_BTNFACE + 1) as _,
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: crate::tray::load_app_icon(true),
    };

    unsafe {
        RegisterClassExW(&wc);

        let width = 420;
        let height = 370;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let pos_x = (screen_w - width) / 2;
        let pos_y = (screen_h - height) / 2;

        let hwnd = CreateWindowExW(
            WS_EX_DLGMODALFRAME,
            class_name.as_ptr(),
            window_title.as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            pos_x,
            pos_y,
            width,
            height,
            0 as _,
            0 as _,
            0 as _,
            ptr::null(),
        );

        if hwnd.is_null() {
            return;
        }

        let font = GetStockObject(DEFAULT_GUI_FONT as _) as HFONT;
        let btn_class = to_wide("BUTTON");
        let static_class = to_wide("STATIC");
        let combo_class = to_wide("COMBOBOX");

        let make_label = |text: &str, x: i32, y: i32, w: i32, h: i32| {
            let t = to_wide(text);
            let h_ctl = CreateWindowExW(
                0,
                static_class.as_ptr(),
                t.as_ptr(),
                WS_CHILD | WS_VISIBLE,
                x,
                y,
                w,
                h,
                hwnd,
                0 as _,
                0 as _,
                ptr::null(),
            );
            SendMessageW(h_ctl, WM_SETFONT, font as usize, 1);
            h_ctl
        };

        let make_group = |text: &str, x: i32, y: i32, w: i32, h: i32| {
            let t = to_wide(text);
            let h_ctl = CreateWindowExW(
                0,
                btn_class.as_ptr(),
                t.as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_GROUPBOX as u32,
                x,
                y,
                w,
                h,
                hwnd,
                0 as _,
                0 as _,
                ptr::null(),
            );
            SendMessageW(h_ctl, WM_SETFONT, font as usize, 1);
            h_ctl
        };

        let make_checkbox = |text: &str, x: i32, y: i32, w: i32, h: i32, id: isize| {
            let t = to_wide(text);
            let h_ctl = CreateWindowExW(
                0,
                btn_class.as_ptr(),
                t.as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_AUTOCHECKBOX as u32,
                x,
                y,
                w,
                h,
                hwnd,
                id as HMENU,
                0 as _,
                ptr::null(),
            );
            SendMessageW(h_ctl, WM_SETFONT, font as usize, 1);
            h_ctl
        };

        let make_button = |text: &str, x: i32, y: i32, w: i32, h: i32, id: isize, is_def: bool| {
            let t = to_wide(text);
            let style = if is_def {
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_DEFPUSHBUTTON as u32
            } else {
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON as u32
            };
            let h_ctl = CreateWindowExW(
                0,
                btn_class.as_ptr(),
                t.as_ptr(),
                style,
                x,
                y,
                w,
                h,
                hwnd,
                id as HMENU,
                0 as _,
                ptr::null(),
            );
            SendMessageW(h_ctl, WM_SETFONT, font as usize, 1);
            h_ctl
        };

        // Load current config
        let cfg = AppConfig::load();

        // 1. Hotkey Group
        make_group("Hotkey Combination", 15, 10, 375, 120);
        let chk_ctrl = make_checkbox("Ctrl", 30, 35, 60, 22, ID_CHK_CTRL);
        let chk_alt = make_checkbox("Alt", 95, 35, 60, 22, ID_CHK_ALT);
        let chk_shift = make_checkbox("Shift", 160, 35, 65, 22, ID_CHK_SHIFT);
        let chk_win = make_checkbox("Win", 230, 35, 60, 22, ID_CHK_WIN);

        SendMessageW(chk_ctrl, BM_SETCHECK, if cfg.ctrl { BST_CHECKED } else { BST_UNCHECKED } as _, 0);
        SendMessageW(chk_alt, BM_SETCHECK, if cfg.alt { BST_CHECKED } else { BST_UNCHECKED } as _, 0);
        SendMessageW(chk_shift, BM_SETCHECK, if cfg.shift { BST_CHECKED } else { BST_UNCHECKED } as _, 0);
        SendMessageW(chk_win, BM_SETCHECK, if cfg.win { BST_CHECKED } else { BST_UNCHECKED } as _, 0);

        make_label("Trigger Key:", 30, 75, 80, 20);
        let cmb_key = CreateWindowExW(
            0,
            combo_class.as_ptr(),
            ptr::null(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | CBS_DROPDOWNLIST as u32,
            115,
            72,
            120,
            200,
            hwnd,
            ID_CMB_KEY as HMENU,
            0 as _,
            ptr::null(),
        );
        SendMessageW(cmb_key, WM_SETFONT, font as usize, 1);

        // Populate combobox with keys
        let mut selected_idx = 0;
        for (i, &key_name) in KEYS_LIST.iter().enumerate() {
            let kw = to_wide(key_name);
            SendMessageW(cmb_key, CB_ADDSTRING, 0, kw.as_ptr() as LPARAM);
            if key_name.eq_ignore_ascii_case(&cfg.key) {
                selected_idx = i;
            }
        }
        SendMessageW(cmb_key, CB_SETCURSEL, selected_idx, 0);

        // 2. Behavior Group
        make_group("Behavior", 15, 140, 375, 60);
        let chk_move_window = make_checkbox(
            "Move active window to primary display",
            30,
            165,
            340,
            22,
            ID_CHK_MOVE_WINDOW,
        );
        SendMessageW(
            chk_move_window,
            BM_SETCHECK,
            if cfg.move_window { BST_CHECKED } else { BST_UNCHECKED } as _,
            0,
        );

        // 3. System Startup Group
        make_group("Startup", 15, 210, 375, 60);
        let chk_startup = make_checkbox(
            "Start Recenter Mouse automatically with Windows",
            30,
            235,
            340,
            22,
            ID_CHK_STARTUP,
        );
        let startup_enabled = crate::startup::is_startup_enabled() || cfg.start_with_windows;
        SendMessageW(
            chk_startup,
            BM_SETCHECK,
            if startup_enabled { BST_CHECKED } else { BST_UNCHECKED } as _,
            0,
        );

        // Buttons
        make_button("Save & Apply", 175, 285, 105, 30, ID_BTN_SAVE, true);
        make_button("Cancel", 290, 285, 100, 30, ID_BTN_CANCEL, false);

        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);

        let dialog = SettingsDialog {
            hwnd,
            chk_ctrl,
            chk_alt,
            chk_shift,
            chk_win,
            cmb_key,
            chk_move_window,
            chk_startup,
            parent_notify_hwnd,
        };

        SETTINGS_STATE.with(|cell| {
            *cell.borrow_mut() = Some(dialog);
        });
    }
}

fn handle_save(hwnd: HWND) {
    SETTINGS_STATE.with(|cell| {
        let borrow = cell.borrow();
        let dlg = match borrow.as_ref() {
            Some(d) => d,
            None => return,
        };

        unsafe {
            let ctrl = SendMessageW(dlg.chk_ctrl, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
            let alt = SendMessageW(dlg.chk_alt, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
            let shift = SendMessageW(dlg.chk_shift, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
            let win = SendMessageW(dlg.chk_win, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;

            if !ctrl && !alt && !shift && !win {
                let err_msg = to_wide("Please select at least one modifier key (Ctrl, Alt, Shift, or Win).");
                let err_title = to_wide("Invalid Hotkey");
                MessageBoxW(hwnd, err_msg.as_ptr(), err_title.as_ptr(), MB_OK | MB_ICONERROR);
                return;
            }

            let sel_idx = SendMessageW(dlg.cmb_key, CB_GETCURSEL, 0, 0);
            if sel_idx < 0 || (sel_idx as usize) >= KEYS_LIST.len() {
                let err_msg = to_wide("Please select a trigger key.");
                let err_title = to_wide("Invalid Key");
                MessageBoxW(hwnd, err_msg.as_ptr(), err_title.as_ptr(), MB_OK | MB_ICONERROR);
                return;
            }

            let key_str = KEYS_LIST[sel_idx as usize].to_string();
            let move_window = SendMessageW(dlg.chk_move_window, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
            let start_with_windows = SendMessageW(dlg.chk_startup, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;

            let updated_cfg = AppConfig {
                ctrl,
                alt,
                shift,
                win,
                key: key_str,
                move_window,
                start_with_windows,
            };

            // Verify that key string parses
            if parse_key_str(&updated_cfg.key).is_none() {
                let err_msg = to_wide("Selected key cannot be parsed as a valid Windows key.");
                let err_title = to_wide("Invalid Key");
                MessageBoxW(hwnd, err_msg.as_ptr(), err_title.as_ptr(), MB_OK | MB_ICONERROR);
                return;
            }

            // Save config
            if let Err(e) = updated_cfg.save() {
                let err_msg = to_wide(&format!("Failed to save configuration file:\n{}", e));
                let err_title = to_wide("Save Error");
                MessageBoxW(hwnd, err_msg.as_ptr(), err_title.as_ptr(), MB_OK | MB_ICONERROR);
                return;
            }

            // Update startup registry
            let _ = set_startup_enabled(start_with_windows);

            // Notify main window to reload configuration and re-register hotkey
            if !dlg.parent_notify_hwnd.is_null() {
                windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                    dlg.parent_notify_hwnd,
                    crate::tray::WM_APP_RELOAD_CONFIG,
                    0,
                    0,
                );
            }

            DestroyWindow(hwnd);
        }
    });
}
