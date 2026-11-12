use crate::{
    hook::{self, Hook, Shared, Snapshot},
    store::{self, Store},
};
use crate::{http, report};
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
        Graphics::Gdi::*,
        System::{LibraryLoader::GetModuleHandleW, Threading::CreateMutexW},
        UI::{Input::KeyboardAndMouse::EnableWindow, Shell::*, WindowsAndMessaging::*},
    },
};

const CALLBACK: u32 = WM_APP + 20;
const TOGGLE: usize = 1;
const SELECTION_TOGGLE: usize = 7;
const CLEAR: usize = 2;
const OPEN: usize = 3;
const EXIT: usize = 4;
const REPORT: usize = 5;
const REPORT_SEND: usize = 601;
const REPORT_CANCEL: usize = 602;
const REPORT_ATTACH: usize = 603;
const REPORT_MSG: usize = 604;
const REPORT_CONTACT: usize = 605;
const REPORT_PREVIEW: usize = 606;
const REPORT_DONE: u32 = WM_APP + 21;
#[derive(Clone)]
struct ReportSeed {
    root: PathBuf,
    settings: store::Settings,
    stats: store::Stats,
    never: std::collections::BTreeSet<String>,
    model: String,
    url: String,
    os: String,
}
struct Tray {
    hook: Hook,
    shared: Shared,
    root: PathBuf,
    restart_message: u32,
    last_notice: Option<String>,
    report: ReportSeed,
}
thread_local! { static TRAY: RefCell<Option<Tray>> = const { RefCell::new(None) }; }
thread_local! { static REPORT_HWND: RefCell<Option<HWND>> = const { RefCell::new(None) }; }
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
unsafe fn child(
    parent: HWND,
    class: PCWSTR,
    text: &str,
    style: WINDOW_STYLE,
    id: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> HWND {
    let text = wide(text);
    CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        class,
        PCWSTR(text.as_ptr()),
        style | WS_CHILD | WS_VISIBLE,
        x,
        y,
        w,
        h,
        Some(parent),
        Some(HMENU(id as *mut _)),
        None,
        None,
    )
    .unwrap_or_default()
}
unsafe fn control_text(hwnd: HWND) -> String {
    let n = GetWindowTextLengthW(hwnd).max(0) as usize;
    let mut text = vec![0u16; n + 1];
    let got = GetWindowTextW(hwnd, &mut text) as usize;
    String::from_utf16_lossy(&text[..got])
}
unsafe fn dlg_item(hwnd: HWND, id: usize) -> HWND {
    GetDlgItem(Some(hwnd), id as i32).unwrap_or_default()
}
unsafe fn send_msg(hwnd: HWND, msg: u32, wp: usize, lp: isize) -> LRESULT {
    SendMessageW(hwnd, msg, Some(WPARAM(wp)), Some(LPARAM(lp)))
}
unsafe fn make_report(seed: &ReportSeed, hwnd: HWND) -> report::Report {
    let mut data = Store {
        root: seed.root.clone(),
        settings: seed.settings.clone(),
        never: seed.never.clone(),
        stats: seed.stats.clone(),
    };
    if let Some((enabled, counts)) = TRAY.with(|t| {
        t.try_borrow().ok().and_then(|t| {
            t.as_ref().map(|t| {
                let s = t.shared.lock().unwrap();
                (s.enabled, s.counts)
            })
        })
    }) {
        data.settings.enabled = enabled;
        let day = store::timestamp(store::now()).0;
        data.stats.insert(
            day,
            [
                ("runs_th".into(), counts.0),
                ("auto_typing".into(), counts.1),
                ("undo_backspace".into(), counts.2),
            ]
            .into(),
        );
    }
    let message = control_text(dlg_item(hwnd, REPORT_MSG));
    let contact = control_text(dlg_item(hwnd, REPORT_CONTACT));
    let attach = send_msg(dlg_item(hwnd, REPORT_ATTACH), 0x00F0, 0, 0).0 == 1;
    report::build(&data, &seed.os, &seed.model, &message, &contact, attach)
}
unsafe fn update_preview(hwnd: HWND) {
    let seed = TRAY.with(|t| {
        t.try_borrow()
            .ok()
            .and_then(|t| t.as_ref().map(|t| t.report.clone()))
    });
    let Some(seed) = seed else { return };
    let message_control = dlg_item(hwnd, REPORT_MSG);
    let message = control_text(message_control);
    if message.chars().count() > 2000 {
        let clipped: String = message.chars().take(2000).collect();
        let text = wide(&clipped);
        let _ = SetWindowTextW(message_control, PCWSTR(text.as_ptr()));
        return;
    }
    let contact_control = dlg_item(hwnd, REPORT_CONTACT);
    let contact = control_text(contact_control);
    if contact.chars().count() > 200 {
        let clipped: String = contact.chars().take(200).collect();
        let text = wide(&clipped);
        let _ = SetWindowTextW(contact_control, PCWSTR(text.as_ptr()));
        return;
    }
    let report = make_report(&seed, hwnd);
    let preview = serde_json::to_string(&report).unwrap_or_default();
    let wide = wide(&preview);
    let _ = SetWindowTextW(dlg_item(hwnd, REPORT_PREVIEW), PCWSTR(wide.as_ptr()));
    let enabled = report::validate(&report).is_ok();
    let _ = EnableWindow(dlg_item(hwnd, REPORT_SEND), enabled);
}
unsafe fn send_report(hwnd: HWND) {
    let Some(seed) = TRAY.with(|t| {
        t.try_borrow()
            .ok()
            .and_then(|t| t.as_ref().map(|t| t.report.clone()))
    }) else {
        return;
    };
    let report = make_report(&seed, hwnd);
    let body = match report::validate(&report) {
        Ok(v) => v,
        Err(e) => {
            let w = wide(&e);
            let _ = MessageBoxW(
                Some(hwnd),
                PCWSTR(w.as_ptr()),
                w!("KeyFlip"),
                MB_OK | MB_ICONWARNING,
            );
            return;
        }
    };
    let _ = EnableWindow(hwnd, false);
    let parent = hwnd.0 as usize;
    std::thread::spawn(move || {
        let result = match http::post(&seed.url, &body) {
            Ok((status, response)) => report::response_message(status, &response),
            Err(e) => format!("ส่งรายงานไม่สำเร็จ: {e}"),
        };
        let ptr = Box::into_raw(Box::new(result));
        unsafe {
            let _ = PostMessageW(
                Some(HWND(parent as *mut _)),
                REPORT_DONE,
                WPARAM(0),
                LPARAM(ptr as isize),
            );
        }
    });
}
unsafe fn open_report(owner: HWND) {
    if let Some(existing) = REPORT_HWND.with(|h| *h.borrow()) {
        let _ = SetForegroundWindow(existing);
        return;
    }
    let instance = GetModuleHandleW(None).unwrap_or_default();
    let class = wide("KeyFlipReportDialog");
    let wc = WNDCLASSW {
        lpfnWndProc: Some(wndproc),
        hInstance: instance.into(),
        hIcon: app_icon(false),
        lpszClassName: PCWSTR(class.as_ptr()),
        hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
        ..Default::default()
    };
    let _ = RegisterClassW(&wc);
    let title = wide("แจ้งปัญหา");
    let hwnd = CreateWindowExW(
        WS_EX_DLGMODALFRAME,
        PCWSTR(class.as_ptr()),
        PCWSTR(title.as_ptr()),
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        580,
        670,
        Some(owner),
        None,
        Some(instance.into()),
        None,
    )
    .unwrap_or_default();
    if hwnd.0.is_null() {
        return;
    }
    let font = CreateFontW(
        18,
        0,
        0,
        0,
        FW_NORMAL.0 as i32,
        0,
        0,
        0,
        DEFAULT_CHARSET,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        DEFAULT_QUALITY,
        DEFAULT_PITCH.0 as u32 | FF_DONTCARE.0 as u32,
        w!("Segoe UI"),
    );
    let label_style = WINDOW_STYLE(0);
    let labels = [
        ("รายละเอียดปัญหา (จำเป็น)", 12, 10),
        ("ข้อมูลติดต่อ (ไม่บังคับ)", 12, 94),
        ("ตัวอย่าง JSON ที่จะส่ง", 12, 207),
    ];
    for (s, x, y) in labels {
        let c = child(hwnd, w!("STATIC"), s, label_style, 0, x, y, 520, 22);
        let _ = send_msg(c, WM_SETFONT, font.0 as usize, 1);
    }
    let msg = child(
        hwnd,
        w!("EDIT"),
        "",
        WS_BORDER | WS_VSCROLL | WINDOW_STYLE(0x0004 | 0x0040 | 0x1000),
        REPORT_MSG,
        12,
        34,
        536,
        54,
    );
    let contact = child(
        hwnd,
        w!("EDIT"),
        "",
        WS_BORDER | WINDOW_STYLE(0x0080),
        REPORT_CONTACT,
        12,
        118,
        536,
        25,
    );
    let check = child(
        hwnd,
        w!("BUTTON"),
        "แนบคำที่แอปแปลงล่าสุด (ไม่เกิน 10 คำ)",
        WINDOW_STYLE(3),
        REPORT_ATTACH,
        12,
        151,
        536,
        28,
    );
    let _ = send_msg(msg, 0x00C5, 8000, 0);
    let _ = send_msg(contact, 0x00C5, 800, 0);
    let preview = child(
        hwnd,
        w!("EDIT"),
        "",
        WS_BORDER | WS_VSCROLL | WINDOW_STYLE(0x0004 | 0x0040 | 0x0800),
        REPORT_PREVIEW,
        12,
        231,
        536,
        355,
    );
    let cancel = child(
        hwnd,
        w!("BUTTON"),
        "ยกเลิก",
        WINDOW_STYLE(0),
        REPORT_CANCEL,
        345,
        600,
        95,
        30,
    );
    let send = child(
        hwnd,
        w!("BUTTON"),
        "ส่ง",
        WINDOW_STYLE(1),
        REPORT_SEND,
        453,
        600,
        95,
        30,
    );
    for c in [msg, contact, check, preview, cancel, send] {
        let _ = send_msg(c, WM_SETFONT, font.0 as usize, 1);
    }
    REPORT_HWND.with(|h| *h.borrow_mut() = Some(hwnd));
    update_preview(hwnd);
    let _ = ShowWindow(hwnd, SW_SHOW);
    let _ = SetForegroundWindow(hwnd);
}
fn windows_os_version() -> String {
    #[repr(C)]
    struct OsVersion {
        size: u32,
        major: u32,
        minor: u32,
        build: u32,
        platform: u32,
        name: [u16; 128],
    }
    #[link(name = "ntdll")]
    extern "system" {
        fn RtlGetVersion(info: *mut OsVersion) -> i32;
    }
    let mut v = OsVersion {
        size: std::mem::size_of::<OsVersion>() as u32,
        major: 0,
        minor: 0,
        build: 0,
        platform: 0,
        name: [0; 128],
    };
    unsafe {
        if RtlGetVersion(&mut v) == 0 {
            format!("Windows {}.{}.{}", v.major, v.minor, v.build)
        } else {
            "Windows".into()
        }
    }
}
unsafe fn icon(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut data = NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: CALLBACK,
        hIcon: app_icon(true),
        ..Default::default()
    };
    copy_text(&mut data.szTip, "KeyFlip · ก ⇄ A");
    data
}
unsafe fn app_icon(small: bool) -> HICON {
    use std::sync::OnceLock;

    // Cached as the handle's address: HICON wraps a raw pointer, which a static cannot hold.
    static SMALL_ICON: OnceLock<usize> = OnceLock::new();
    static LARGE_ICON: OnceLock<usize> = OnceLock::new();
    let cache = if small { &SMALL_ICON } else { &LARGE_ICON };
    let handle = *cache.get_or_init(|| {
        let (width, height) = if small {
            (GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON))
        } else {
            (GetSystemMetrics(SM_CXICON), GetSystemMetrics(SM_CYICON))
        };
        GetModuleHandleW(None)
            .ok()
            .and_then(|module| {
                LoadImageW(
                    Some(module.into()),
                    PCWSTR(1usize as *const u16),
                    IMAGE_ICON,
                    width,
                    height,
                    LR_SHARED,
                )
                .ok()
            })
            .map(|handle| handle.0 as usize)
            .unwrap_or_else(|| LoadIconW(None, IDI_APPLICATION).unwrap_or_default().0 as usize)
    });
    HICON(handle as *mut _)
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
    let selection_checked = if snapshot.selection_enabled {
        MF_CHECKED
    } else {
        MF_UNCHECKED
    };
    let _ = AppendMenuW(
        menu,
        MF_STRING | selection_checked,
        SELECTION_TOGGLE,
        w!("แปลงข้อความที่เลือก (Ctrl+Alt+Space)"),
    );
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
    let _ = AppendMenuW(menu, MF_STRING, REPORT, w!("แจ้งปัญหา…"));
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
        SELECTION_TOGGLE => {
            let _ = PostThreadMessageW(thread_id, hook::SELECTION_TOGGLE, WPARAM(0), LPARAM(0));
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
        REPORT => open_report(hwnd),
        _ => {}
    }
}
unsafe extern "system" fn wndproc(hwnd: HWND, message: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match message {
        WM_COMMAND => {
            match wp.0 as u16 as usize {
                REPORT_CANCEL => {
                    let _ = DestroyWindow(hwnd);
                    REPORT_HWND.with(|h| *h.borrow_mut() = None);
                }
                REPORT_SEND => send_report(hwnd),
                REPORT_ATTACH => update_preview(hwnd),
                _ if (wp.0 >> 16) as u16 == EN_CHANGE as u16 => update_preview(hwnd),
                _ => {}
            }
            return LRESULT(0);
        }
        REPORT_DONE => {
            let text = Box::from_raw(lp.0 as *mut String);
            let wide = wide(&text);
            let _ = MessageBoxW(Some(hwnd), PCWSTR(wide.as_ptr()), w!("KeyFlip"), MB_OK);
            let _ = EnableWindow(hwnd, true);
            return LRESULT(0);
        }
        WM_CLOSE if REPORT_HWND.with(|h| *h.borrow() == Some(hwnd)) => {
            REPORT_HWND.with(|h| *h.borrow_mut() = None);
            let _ = DestroyWindow(hwnd);
            return LRESULT(0);
        }
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
        let model = if root.join("models/th.kfm").is_file() && root.join("models/en.kfm").is_file()
        {
            "personal"
        } else if store.settings.models_dir.is_some() {
            "open"
        } else {
            "generic"
        };
        let report_seed = ReportSeed {
            root: root.clone(),
            settings: store.settings.clone(),
            stats: store.stats.clone(),
            never: store.never.clone(),
            model: model.into(),
            url: store
                .settings
                .extra
                .get("reportURL")
                .and_then(|v| v.as_str())
                .unwrap_or(report::ENDPOINT)
                .to_string(),
            os: windows_os_version(),
        };
        let shared = Arc::new(Mutex::new(Snapshot {
            enabled: store.settings.enabled,
            selection_enabled: store.settings.selection_hotkey,
            counts: store.counts(&store::timestamp(store::now()).0),
            day: store::timestamp(store::now()).0,
            never: store.never.len(),
            notice: None,
        }));
        let instance = GetModuleHandleW(None).map_err(|e| e.to_string())?;
        let class = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            hIcon: app_icon(false),
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
                report: report_seed,
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
