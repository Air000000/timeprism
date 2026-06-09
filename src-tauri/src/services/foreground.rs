#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::CloseHandle;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::SystemInformation::GetTickCount;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
};

#[cfg(target_os = "windows")]
pub(crate) fn capture_foreground_window() -> Result<Option<(String, String)>, String> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_null() {
        return Ok(None);
    }

    let mut pid: u32 = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut pid as *mut u32);
    }
    if pid == 0 {
        return Ok(None);
    }

    let title_len = unsafe { GetWindowTextLengthW(hwnd) };
    let mut title = String::new();
    if title_len > 0 {
        let mut buffer = vec![0u16; (title_len + 1) as usize];
        let copied = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), title_len + 1) };
        if copied > 0 {
            title = String::from_utf16_lossy(&buffer[..copied as usize]);
        }
    }
    if title.trim().is_empty() {
        title = "Untitled Window".to_string();
    }

    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process.is_null() {
        return Ok(Some((format!("pid-{pid}.exe"), title)));
    }

    let mut path_buf = vec![0u16; 1024];
    let mut size = path_buf.len() as u32;
    let query_ok =
        unsafe { QueryFullProcessImageNameW(process, 0, path_buf.as_mut_ptr(), &mut size) };
    unsafe {
        CloseHandle(process);
    }

    let process_name = if query_ok != 0 && size > 0 {
        String::from_utf16_lossy(&path_buf[..size as usize])
    } else {
        format!("pid-{pid}.exe")
    };

    Ok(Some((process_name, title)))
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn capture_foreground_window() -> Result<Option<(String, String)>, String> {
    Err("foreground capture is currently supported only on Windows".to_string())
}

#[cfg(target_os = "windows")]
pub(crate) fn current_idle_millis() -> Result<i64, String> {
    let mut info = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };

    let ok = unsafe { GetLastInputInfo(&mut info as *mut LASTINPUTINFO) };
    if ok == 0 {
        return Err("GetLastInputInfo failed".to_string());
    }

    let now_tick = unsafe { GetTickCount() };
    let elapsed = now_tick.wrapping_sub(info.dwTime) as i64;
    Ok(elapsed.max(0))
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn current_idle_millis() -> Result<i64, String> {
    Ok(0)
}
