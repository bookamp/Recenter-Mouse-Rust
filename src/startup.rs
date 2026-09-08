use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

const RUN_SUBKEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const APP_VALUE_NAME: &str = "RecenterMouse";

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}

pub fn is_startup_enabled() -> bool {
    let subkey = to_wide(RUN_SUBKEY);
    let val_name = to_wide(APP_VALUE_NAME);
    let mut hkey: HKEY = ptr::null_mut();

    unsafe {
        let res = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_READ,
            &mut hkey,
        );
        if res != ERROR_SUCCESS {
            return false;
        }

        let mut data_type: u32 = 0;
        let mut data_len: u32 = 0;
        let query_res = RegQueryValueExW(
            hkey,
            val_name.as_ptr(),
            ptr::null_mut(),
            &mut data_type,
            ptr::null_mut(),
            &mut data_len,
        );
        RegCloseKey(hkey);

        query_res == ERROR_SUCCESS && data_len > 0
    }
}

pub fn set_startup_enabled(enable: bool) -> Result<(), String> {
    let subkey = to_wide(RUN_SUBKEY);
    let val_name = to_wide(APP_VALUE_NAME);
    let mut hkey: HKEY = ptr::null_mut();

    unsafe {
        let res = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_WRITE | KEY_READ,
            &mut hkey,
        );
        if res != ERROR_SUCCESS {
            return Err(format!("Failed to open registry key: error code {}", res));
        }

        let op_res = if enable {
            let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
            let exe_str = format!("\"{}\"", exe_path.to_string_lossy());
            let wide_val = to_wide(&exe_str);
            let byte_len = (wide_val.len() * std::mem::size_of::<u16>()) as u32;

            RegSetValueExW(
                hkey,
                val_name.as_ptr(),
                0,
                REG_SZ,
                wide_val.as_ptr() as *const u8,
                byte_len,
            )
        } else {
            RegDeleteValueW(hkey, val_name.as_ptr())
        };

        RegCloseKey(hkey);

        if op_res == ERROR_SUCCESS || (!enable && op_res == 2) {
            // Error 2 is ERROR_FILE_NOT_FOUND, which means it wasn't registered anyway
            Ok(())
        } else {
            Err(format!("Registry operation failed with code {}", op_res))
        }
    }
}
