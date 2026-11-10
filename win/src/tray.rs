use crate::{
    hook::{self, Hook, Shared, Snapshot},
    store::{self, Store},
};
use std::{
    cell::RefCell,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{
            CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, HWND, LPARAM, LRESULT, POINT,
            WPARAM,
        },
        System::{LibraryLoader::GetModuleHandleW, Threading::CreateMutexW},
        UI::{Shell::*, WindowsAndMessaging::*},
    },
};

const CALLBACK: u32 = WM_APP + 20;
const TOGGLE: usize = 1;
const CLEAR: usize = 2;
const OPEN: usize = 3;
const EXIT: usize = 4;
struct Tray {
    hook: Hook,
    shared: Shared,
    root: PathBuf,
    restart_message: u32,
    last_notice: Option<String>,
}
thread_local! { static TRAY: RefCell<Option<Tray>> = const { RefCell::new(None) }; }
struct OwnedHandle(HANDLE);
impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
fn copy_text(out: &mut [u16], text: &str) {
    // UTF-16 truncation is only used for short tray labels/notices.
    let n = out.len().saturating_sub(1);
    for (dst, unit) in out.iter_mut().take(n).zip(text.encode_utf16()) {
        *dst = unit;
    }
}
unsafe fn icon(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut data = NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: CALLBACK,
        hIcon: LoadIconW(None, IDI_APPLICATION).unwrap_or_default(),
        ..Default::default()
    };
    copy_text(&mut data.szTip, "KeyFlip · ก ⇄ A");
    data
}
unsafe fn notice(hwnd: HWND, text: &str) {
    let mut data = icon(hwnd);
    data.uFlags = NIF_INFO;
    data.dwInfoFlags = NIIF_WARNING;
    copy_text(&mut data.szInfoTitle, "KeyFlip");
    copy_text(&mut data.szInfo, text);
    let _ = Shell_NotifyIconW(NIM_MODIFY, &data);
}
unsafe fn menu(hwnd: HWND, thread_id: u32, root: PathBuf, mut snapshot: Snapshot) {
    if snapshot.day != store::timestamp(store::now()).0 {
        snapshot.counts = (0, 0, 0);
    }
    let Ok(menu) = CreatePopupMenu() else { return };
    let checked = if snapshot.enabled {
        MF_CHECKED
    } else {
        MF_UNCHECKED
    };
    let _ = AppendMenuW(menu, MF_STRING | checked, TOGGLE, w!("แก้ภาษาอัตโนมัติ"));
    let info = wide(&format!(
        "วันนี้: พิมพ์ {} คำ · แก้ให้ {} · ย้อน {}",
        snapshot.counts.0, snapshot.counts.1, snapshot.counts.2
    ));
    let _ = AppendMenuW(menu, MF_STRING | MF_GRAYED, 0, PCWSTR(info.as_ptr()));
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    let clear = wide(&format!("ล้างคำที่ตั้งไม่ให้แปลง ({})", snapshot.never));
    let _ = AppendMenuW(
        menu,
        MF_STRING
            | if snapshot.never == 0 {
                MF_GRAYED
            } else {
                MF_ENABLED
            },
        CLEAR,
        PCWSTR(clear.as_ptr()),
    );
    let _ = AppendMenuW(menu, MF_STRING, OPEN, w!("เปิดโฟลเดอร์ข้อมูล"));
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    let _ = AppendMenuW(menu, MF_STRING, EXIT, w!("ออก"));
    let mut point = POINT::default();
    let _ = GetCursorPos(&mut point);
    let _ = SetForegroundWindow(hwnd);
    let command = TrackPopupMenu(
        menu,
        TPM_RETURNCMD | TPM_RIGHTBUTTON,
        point.x,
        point.y,
        None,
        hwnd,
        None,
    )
    .0 as usize;
    let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
    let _ = DestroyMenu(menu);
    match command {
        TOGGLE => {
            let _ = PostThreadMessageW(thread_id, hook::TOGGLE, WPARAM(0), LPARAM(0));
        }
        CLEAR => {
            let _ = PostThreadMessageW(thread_id, hook::CLEAR, WPARAM(0), LPARAM(0));
        }
        OPEN => {
            use std::os::windows::ffi::OsStrExt;
            let root: Vec<u16> = root.as_os_str().encode_wide().chain(Some(0)).collect();
            ShellExecuteW(
                Some(hwnd),
                w!("open"),
                PCWSTR(root.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );
        }
        EXIT => PostQuitMessage(0),
        _ => {}
    }
}
unsafe extern "system" fn wndproc(hwnd: HWND, message: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match message {
        CALLBACK if matches!(lp.0 as u32, WM_RBUTTONUP | WM_LBUTTONUP | WM_CONTEXTMENU) => {
            // Do not retain the RefCell borrow across TrackPopupMenu's nested message loop.
            let context = TRAY.with(|tray| {
                tray.try_borrow().ok().and_then(|tray| {
                    tray.as_ref()
                        .map(|t| (t.hook.id, t.root.clone(), t.shared.lock().unwrap().clone()))
                })
            });
            if let Some((id, root, snapshot)) = context {
                menu(hwnd, id, root, snapshot);
            }
            return LRESULT(0);
        }
        WM_TIMER => {
            TRAY.with(|tray| {
                if let Ok(mut tray) = tray.try_borrow_mut() {
                    if let Some(tray) = tray.as_mut() {
                        let text = tray.shared.lock().unwrap().notice.clone();
                        if text != tray.last_notice {
                            if let Some(text) = &text {
                                notice(hwnd, text);
                            }
                            tray.last_notice = text;
                        }
                    }
                }
            });
            return LRESULT(0);
        }
        WM_CLOSE => {
            PostQuitMessage(0);
            return LRESULT(0);
        }
        _ => {}
    }
    TRAY.with(|tray| {
        if let Ok(tray) = tray.try_borrow() {
            if tray.as_ref().is_some_and(|t| message == t.restart_message) {
                let _ = Shell_NotifyIconW(NIM_ADD, &icon(hwnd));
            }
        }
    });
    DefWindowProcW(hwnd, message, wp, lp)
}

pub fn error(text: &str) {
    let text = wide(text);
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(text.as_ptr()),
            w!("KeyFlip"),
            MB_OK | MB_ICONWARNING,
        );
    }
}
pub fn run() -> Result<(), String> {
    unsafe {
        let mutex = CreateMutexW(None, false, w!("Local\\KeyFlip.Tray.Singleton"))
            .map_err(|e| e.to_string())?;
        let duplicate = GetLastError() == ERROR_ALREADY_EXISTS;
        let _mutex = OwnedHandle(mutex);
        if duplicate {
            return Ok(());
        }
        let root = store::data_root(std::env::var_os("APPDATA").ok_or("ไม่พบโฟลเดอร์ APPDATA")?);
        let store = Store::open(root.clone()).map_err(|e| e.to_string())?;
        let shared = Arc::new(Mutex::new(Snapshot {
            enabled: store.settings.enabled,
            counts: store.counts(&store::timestamp(store::now()).0),
            day: store::timestamp(store::now()).0,
            never: store.never.len(),
            notice: None,
        }));
        let instance = GetModuleHandleW(None).map_err(|e| e.to_string())?;
        let class = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: w!("KeyFlipTrayWindow"),
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 {
            return Err("สร้างหน้าต่างถาดระบบไม่สำเร็จ".into());
        }
        // An invisible top-level window receives TaskbarCreated after Explorer restarts.
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class.lpszClassName,
            w!("KeyFlip"),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(instance.into()),
            None,
        )
        .map_err(|e| e.to_string())?;
        let hook = match hook::start(store, shared.clone()) {
            Ok(hook) => hook,
            Err(e) => {
                let _ = DestroyWindow(hwnd);
                return Err(e);
            }
        };
        TRAY.with(|tray| {
            *tray.borrow_mut() = Some(Tray {
                hook,
                shared,
                root,
                restart_message: RegisterWindowMessageW(w!("TaskbarCreated")),
                last_notice: None,
            })
        });
        let added = Shell_NotifyIconW(NIM_ADD, &icon(hwnd)).as_bool();
        let timer = SetTimer(Some(hwnd), 1, 1000, None);
        let mut message = MSG::default();
        if added && timer != 0 {
            loop {
                let result = GetMessageW(&mut message, None, 0, 0).0;
                if result <= 0 {
                    break;
                }
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        let _ = KillTimer(Some(hwnd), 1);
        let _ = Shell_NotifyIconW(NIM_DELETE, &icon(hwnd));
        let _ = DestroyWindow(hwnd);
        if let Some(tray) = TRAY.with(|tray| tray.borrow_mut().take()) {
            tray.hook.stop();
        }
        if !added || timer == 0 {
            return Err("สร้างไอคอนถาดระบบไม่สำเร็จ".into());
        }
        Ok(())
    }
}
