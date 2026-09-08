use std::mem::size_of;
use std::ptr;
use windows_sys::Win32::Foundation::{BOOL, LPARAM, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SetCursorPos, MONITORINFOF_PRIMARY, SM_CXSCREEN, SM_CYSCREEN,
};

/// Retrieve the bounding rectangle of the primary display monitor.
pub fn get_primary_monitor_rect() -> RECT {
    unsafe extern "system" fn monitor_enum_proc(
        hmonitor: HMONITOR,
        _hdc: HDC,
        _rect: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        let mut mi: MONITORINFO = std::mem::zeroed();
        mi.cbSize = size_of::<MONITORINFO>() as u32;

        if GetMonitorInfoW(hmonitor, &mut mi) != 0 {
            if (mi.dwFlags & MONITORINFOF_PRIMARY) != 0 {
                let out_rect = lparam as *mut RECT;
                *out_rect = mi.rcMonitor;
                return 0; // Stop enumeration once primary monitor is found
            }
        }
        1 // Continue enumeration
    }

    let mut primary_rect: RECT = unsafe { std::mem::zeroed() };
    let success = unsafe {
        EnumDisplayMonitors(
            ptr::null_mut(),
            ptr::null(),
            Some(monitor_enum_proc),
            &mut primary_rect as *mut RECT as LPARAM,
        )
    };

    // If enumeration failed or returned empty rectangle, fallback to primary screen metrics
    if success != 0 && (primary_rect.right > primary_rect.left && primary_rect.bottom > primary_rect.top) {
        primary_rect
    } else {
        unsafe {
            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);
            RECT {
                left: 0,
                top: 0,
                right: width,
                bottom: height,
            }
        }
    }
}

/// Calculate the center coordinate (X, Y) of a given monitor RECT.
pub fn get_rect_center(rect: &RECT) -> (i32, i32) {
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    (rect.left + width / 2, rect.top + height / 2)
}

/// Move the mouse cursor to the center of the primary display.
pub fn center_cursor_on_primary() -> (i32, i32) {
    let rect = get_primary_monitor_rect();
    let (center_x, center_y) = get_rect_center(&rect);
    unsafe {
        SetCursorPos(center_x, center_y);
    }
    (center_x, center_y)
}
