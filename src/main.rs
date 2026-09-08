#![windows_subsystem = "windows"]

mod config;
mod hotkey;
mod monitor;
mod settings_ui;
mod startup;
mod tray;
mod window;

use config::AppConfig;
use std::cell::RefCell;
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, MessageBoxW,
    PostQuitMessage, RegisterClassExW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, MB_ICONINFORMATION,
    MB_OK, MSG, WM_COMMAND, WM_CONTEXTMENU, WM_DESTROY, WM_HOTKEY, WM_LBUTTONDBLCLK,
    WM_LBUTTONUP, WM_RBUTTONUP, WNDCLASSEXW,
};

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}

thread_local! {
    static GLOBAL_CONFIG: RefCell<AppConfig> = RefCell::new(AppConfig::default());
    static HEADLESS_MODE: RefCell<bool> = const { RefCell::new(false) };
}

fn execute_recenter(cfg: &AppConfig) {
    let rect = monitor::get_primary_monitor_rect();
    monitor::center_cursor_on_primary();

    if cfg.move_window {
        window::move_active_window_to_monitor(&rect);
    }
}

fn main() {
    // 1. Enable modern Per-Monitor V2 DPI awareness so screen coordinates match physical pixels
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    // 2. Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut is_once = false;
    let mut is_settings = false;
    let mut is_headless = false;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--once" | "-o" => is_once = true,
            "--settings" | "-s" => is_settings = true,
            "--headless" => is_headless = true,
            "--help" | "-h" => {
                let help_text = to_wide(
                    "Recenter Mouse (Rust)\n\n\
                    Options:\n  \
                    --once, -o       Recenter cursor and active window once, then exit\n  \
                    --settings, -s   Open the Settings & Key Bindings interface\n  \
                    --headless       Run hotkey listener in background without tray icon\n  \
                    --help, -h       Show this information dialog",
                );
                let help_title = to_wide("Recenter Mouse - Help");
                unsafe {
                    MessageBoxW(0 as _, help_text.as_ptr(), help_title.as_ptr(), MB_OK | MB_ICONINFORMATION);
                }
                return;
            }
            _ => {}
        }
    }

    // 3. Load configuration
    let cfg = AppConfig::load();
    GLOBAL_CONFIG.with(|c| *c.borrow_mut() = cfg.clone());
    HEADLESS_MODE.with(|h| *h.borrow_mut() = is_headless);

    // 4. If --once was requested, perform recenter and exit immediately
    if is_once {
        execute_recenter(&cfg);
        return;
    }

    // 5. Ensure single running instance
    let mutex_name = to_wide("RecenterMouseRustSingletonMutex");
    let _mutex = unsafe { CreateMutexW(ptr::null(), 1, mutex_name.as_ptr()) };
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        if is_settings {
            // User requested settings while daemon is running, open standalone settings
            settings_ui::show_settings_window(0 as _);
            unsafe {
                let mut msg: MSG = std::mem::zeroed();
                while GetMessageW(&mut msg, 0 as _, 0, 0) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            return;
        }
        return;
    }

    // 6. Register hidden message-only window class
    let class_name = to_wide("RecenterMouseMessageWindowClass");

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_HOTKEY => {
                if wparam == hotkey::RECENTER_HOTKEY_ID as usize {
                    GLOBAL_CONFIG.with(|c| {
                        execute_recenter(&c.borrow());
                    });
                }
                0
            }
            tray::WM_APP_TRAY_MSG => {
                let event = lparam as u32;
                if event == WM_RBUTTONUP || event == WM_CONTEXTMENU {
                    GLOBAL_CONFIG.with(|c| {
                        tray::show_tray_menu(hwnd, &c.borrow());
                    });
                } else if event == WM_LBUTTONUP || event == WM_LBUTTONDBLCLK {
                    // Left click or double click on tray icon triggers quick recenter
                    GLOBAL_CONFIG.with(|c| {
                        execute_recenter(&c.borrow());
                    });
                }
                0
            }
            WM_COMMAND => {
                let item_id = (wparam & 0xFFFF) as usize;
                match item_id {
                    tray::ID_TRAY_RECENTER => {
                        GLOBAL_CONFIG.with(|c| {
                            execute_recenter(&c.borrow());
                        });
                    }
                    tray::ID_TRAY_TOGGLE_WINDOW => {
                        GLOBAL_CONFIG.with(|c| {
                            let mut cfg = c.borrow_mut();
                            cfg.move_window = !cfg.move_window;
                            let _ = cfg.save();
                        });
                    }
                    tray::ID_TRAY_SETTINGS => {
                        settings_ui::show_settings_window(hwnd);
                    }
                    tray::ID_TRAY_TOGGLE_STARTUP => {
                        GLOBAL_CONFIG.with(|c| {
                            let mut cfg = c.borrow_mut();
                            cfg.start_with_windows = !cfg.start_with_windows;
                            let _ = startup::set_startup_enabled(cfg.start_with_windows);
                            let _ = cfg.save();
                        });
                    }
                    tray::ID_TRAY_EXIT => {
                        DestroyWindow(hwnd);
                    }
                    _ => {}
                }
                0
            }
            tray::WM_APP_RELOAD_CONFIG => {
                // Reload configuration after Settings UI changes
                let updated = AppConfig::load();
                GLOBAL_CONFIG.with(|c| {
                    *c.borrow_mut() = updated.clone();
                });
                let _ = hotkey::register_app_hotkey(hwnd, &updated);

                HEADLESS_MODE.with(|h| {
                    if !*h.borrow() {
                        tray::update_tray_icon(hwnd, &updated);
                    }
                });
                0
            }
            WM_DESTROY => {
                HEADLESS_MODE.with(|h| {
                    if !*h.borrow() {
                        tray::remove_tray_icon(hwnd);
                    }
                });
                hotkey::unregister_app_hotkey(hwnd);
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: 0 as _,
        hIcon: tray::load_app_icon(false),
        hCursor: 0 as _,
        hbrBackground: 0 as _,
        lpszMenuName: ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: tray::load_app_icon(true),
    };

    unsafe {
        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            to_wide("RecenterMouseMessageWindow").as_ptr(),
            0,
            0,
            0,
            0,
            0,
            0 as _,
            0 as _,
            0 as _,
            ptr::null(),
        );

        if hwnd.is_null() {
            return;
        }

        // 7. Register Hotkey
        if let Err(err) = hotkey::register_app_hotkey(hwnd, &cfg) {
            let err_w = to_wide(&format!("{}\nYou can change the hotkey in Settings.", err));
            let title_w = to_wide("Recenter Mouse - Hotkey Warning");
            MessageBoxW(0 as _, err_w.as_ptr(), title_w.as_ptr(), MB_OK | MB_ICONINFORMATION);
        }

        // 8. Add Tray Icon unless headless
        if !is_headless {
            tray::add_tray_icon(hwnd, &cfg);
        }

        // 9. If started with --settings, open settings window immediately
        if is_settings {
            settings_ui::show_settings_window(hwnd);
        }

        // 10. Native Win32 Message Loop (Zero CPU utilization when idle)
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0 as _, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
