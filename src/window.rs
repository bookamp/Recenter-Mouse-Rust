use windows_sys::Win32::Foundation::{HWND, RECT};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetDesktopWindow, GetForegroundWindow, GetShellWindow, IsZoomed, SetWindowPos, ShowWindow,
    HWND_TOP, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SW_RESTORE,
};

/// Move the currently active foreground window to the primary display.
/// Unmaximizes the window if it was maximized, preserving its size.
pub fn move_active_window_to_monitor(target_rect: &RECT) -> bool {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.is_null() {
            return false;
        }

        // Avoid touching the desktop window or shell window (taskbar)
        let desktop = GetDesktopWindow();
        let shell = GetShellWindow();
        if hwnd == desktop || hwnd == shell {
            return false;
        }

        // If the window is maximized, restore it first
        if IsZoomed(hwnd) != 0 {
            ShowWindow(hwnd, SW_RESTORE);
        }

        // Position window at the top-left of target monitor without resizing or altering z-order
        let result = SetWindowPos(
            hwnd,
            HWND_TOP,
            target_rect.left,
            target_rect.top,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        );

        result != 0
    }
}
