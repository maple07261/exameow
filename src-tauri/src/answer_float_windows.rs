//! Maintain the answer window's Z-order even while another topmost app is active.
use tauri::WebviewWindow;
use windows_sys::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        IsIconic, IsWindowVisible, KillTimer, SetTimer, SetWindowPos, HWND_TOPMOST, HWND_BOTTOM,
        GetWindowLongPtrW, GWL_EXSTYLE, WS_EX_TOPMOST,
        SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE,
    },
};

const TOPMOST_TIMER_ID: usize = 0x45584d46;
const TOPMOST_INTERVAL_MS: u32 = 500;

// Called only on the HWND's owning UI thread. Do not show hidden/minimized
// windows, change geometry, or activate the window (VMware keeps keyboard focus).
unsafe fn raise_without_activation(hwnd: HWND) -> Result<(), std::io::Error> {
    if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
        return Ok(());
    }
    if SetWindowPos(
        hwnd, HWND_TOPMOST, 0, 0, 0, 0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
    ) == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

unsafe extern "system" fn maintain_topmost(hwnd: HWND, _: u32, _: usize, _: u32) {
    // Never panic across a native callback boundary. A later tick can retry a
    // transient failure, such as a desktop switch, without stealing focus.
    // A queued WM_TIMER can arrive after KillTimer; never promote a lowered window.
    if GetWindowLongPtrW(hwnd, GWL_EXSTYLE) & WS_EX_TOPMOST as isize != 0 {
        let _ = raise_without_activation(hwnd);
    }
}

pub async fn install(window: &WebviewWindow) -> Result<(), String> {
    let window_on_ui = window.clone();
    let (send, receive) = tokio::sync::oneshot::channel();
    window.run_on_main_thread(move || {
        let result = (|| {
            let hwnd = window_on_ui.hwnd().map_err(|e| e.to_string())?.0 as HWND;
            unsafe {
                raise_without_activation(hwnd).map_err(|e| e.to_string())?;
                // Binding the timer to this HWND makes Windows destroy it when
                // the float closes. Reopening creates a fresh timer, no worker
                // thread or stale HWND survives the old window's destruction.
                if SetTimer(hwnd, TOPMOST_TIMER_ID, TOPMOST_INTERVAL_MS, Some(maintain_topmost)) == 0 {
                    return Err(std::io::Error::last_os_error().to_string());
                }
            }
            Ok(())
        })();
        let _ = send.send(result);
    }).map_err(|e| e.to_string())?;
    receive.await.map_err(|e| e.to_string())?
}

// Must run on the UI thread, serialized with the timer callback.
pub fn set_topmost(window: &WebviewWindow, topmost: bool) -> Result<(), String> {
    let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as HWND;
    unsafe {
        KillTimer(hwnd, TOPMOST_TIMER_ID);
        // Keep Tauri's cached flag in sync with the native change.
        window.set_always_on_top(topmost).map_err(|e| e.to_string())?;
        if topmost {
            raise_without_activation(hwnd).map_err(|e| e.to_string())?;
            if SetTimer(hwnd, TOPMOST_TIMER_ID, TOPMOST_INTERVAL_MS, Some(maintain_topmost)) == 0 {
                return Err(std::io::Error::last_os_error().to_string());
            }
        } else if SetWindowPos(hwnd, HWND_BOTTOM, 0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER) == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
    }
    Ok(())
}
