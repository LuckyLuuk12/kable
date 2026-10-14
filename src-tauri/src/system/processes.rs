use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::process::Command;
use std::sync::Mutex;

static TRACKED_PIDS: Lazy<Mutex<HashSet<u32>>> = Lazy::new(|| Mutex::new(HashSet::new()));

pub fn track_process(pid: u32) {
    if let Ok(mut pids) = TRACKED_PIDS.lock() {
        pids.insert(pid);
    }
}

pub fn untrack_process(pid: u32) {
    if let Ok(mut pids) = TRACKED_PIDS.lock() {
        pids.remove(&pid);
    }
}

pub fn is_tracked(pid: u32) -> bool {
    if let Ok(pids) = TRACKED_PIDS.lock() {
        pids.contains(&pid)
    } else {
        false
    }
}

pub async fn kill_process(process_id: u32) -> Result<(), String> {
    if !is_tracked(process_id) {
        return Err(format!("Process {} is not tracked", process_id));
    }

    #[cfg(target_os = "windows")]
    {
        let status = Command::new("taskkill")
            .args(["/PID", &process_id.to_string(), "/F"])
            .status()
            .map_err(|e| format!("Failed to run taskkill: {}", e))?;
        if status.success() {
            untrack_process(process_id);
            Ok(())
        } else {
            Err(format!("taskkill failed for PID {}", process_id))
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let status =
            Command::new("kill").args(["-9", &process_id.to_string()]).status().map_err(|e| format!("Failed to run kill: {}", e))?;
        if status.success() {
            untrack_process(process_id);
            Ok(())
        } else {
            Err(format!("kill failed for PID {}", process_id))
        }
    }
}

pub fn get_tracked_pids() -> Vec<u32> {
    if let Ok(pids) = TRACKED_PIDS.lock() {
        pids.iter().cloned().collect()
    } else {
        Vec::new()
    }
}

pub fn is_process_alive(pid: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
        const STILL_ACTIVE: u32 = 259;
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() {
                return false;
            }
            let mut code = 0u32;
            let ok = GetExitCodeProcess(handle, &mut code as *mut u32);
            CloseHandle(handle);
            ok != 0 && code == STILL_ACTIVE
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
}

pub fn get_process_exit_code(pid: u32) -> Option<i32> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() {
                return None;
            }
            let mut code = 0u32;
            let ok = GetExitCodeProcess(handle, &mut code as *mut u32);
            CloseHandle(handle);
            if ok != 0 {
                Some(code as i32)
            } else {
                None
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

pub fn hide_console_window() {
    // Hides the console window on Windows when running in CLI mode. This is a bit of a hack but it works.
    #[cfg(windows)]
    {
        extern "C" {
            fn GetConsoleWindow() -> *mut std::ffi::c_void;
            fn ShowWindow(hwnd: *mut std::ffi::c_void, n_cmd_show: i32) -> i32;
        }
        const SW_HIDE: i32 = 0;
        unsafe {
            let console_window = GetConsoleWindow();
            if !console_window.is_null() {
                ShowWindow(console_window, SW_HIDE);
            }
        }
    }
}
