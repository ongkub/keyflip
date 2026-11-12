use keyflip_core::tracker::Lang;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
        OnceLock,
    },
    thread,
    time::Duration,
};
use windows::{
    core::w,
    Win32::{
        Foundation::{HANDLE, HGLOBAL, HWND, LPARAM, WPARAM},
        System::{DataExchange::*, Memory::*, Ole::*},
        UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
};

pub static FLOW_ACTIVE: AtomicBool = AtomicBool::new(false);
thread_local! {
    // Hidden message-only window that owns the clipboard while a flow runs: Microsoft documents that
    // EmptyClipboard after OpenClipboard(NULL) leaves no owner and SetClipboardData may then fail.
    static OWNER: std::cell::Cell<HWND> = std::cell::Cell::new(HWND::default());
}
static CONVERT: OnceLock<Sender<(String, Sender<Option<(String, usize, Lang)>>)>> = OnceLock::new();
pub fn configure(tx: Sender<(String, Sender<Option<(String, usize, Lang)>>)>) {
    let _ = CONVERT.set(tx);
}
pub fn start(
    thread_id: u32,
    hwnd: HWND,
    token: u64,
    en: HKL,
    th: HKL,
    privacy: std::sync::Arc<crate::uia::Privacy>,
) {
    let hwnd_bits = hwnd.0 as usize;
    let en_bits = en.0 as usize;
    let th_bits = th.0 as usize;
    thread::spawn(move || unsafe {
        let hwnd = HWND(hwnd_bits as *mut _);
        let en = HKL(en_bits as *mut _);
        let th = HKL(th_bits as *mut _);
        let owner = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("STATIC"),
            w!(""),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            None,
            None,
        )
        .unwrap_or_default();
        OWNER.with(|o| o.set(owner));
        for _ in 0..60 {
            if GetAsyncKeyState(VK_CONTROL.0 as i32) >= 0 && GetAsyncKeyState(VK_MENU.0 as i32) >= 0
            {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        if GetAsyncKeyState(VK_CONTROL.0 as i32) < 0 || GetAsyncKeyState(VK_MENU.0 as i32) < 0 {
            finish();
            return;
        }
        if !permitted(hwnd, token, &privacy) {
            finish();
            return;
        }
        let Some(saved) = snapshot() else {
            finish();
            return;
        };
        let initial = GetClipboardSequenceNumber();
        if !send_chord(VK_C) {
            finish();
            return;
        }
        let mut changed = false;
        for _ in 0..40 {
            if GetClipboardSequenceNumber() != initial {
                changed = true;
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        if !changed {
            finish();
            return;
        }
        let Some(text) = read_text() else {
            restore(&saved);
            finish();
            return;
        };
        // From here on, give the user's clipboard back only while it still holds our copy: they may have
        // copied something else in the meantime.
        let copied_seq = GetClipboardSequenceNumber();
        if text.is_empty() || text.chars().count() > 5000 {
            restore_if(&saved, copied_seq);
            finish();
            return;
        }
        let original = text.clone();
        let (reply_tx, reply_rx) = mpsc::channel();
        let Some(tx) = CONVERT.get() else {
            restore_if(&saved, copied_seq);
            finish();
            return;
        };
        if tx.send((text, reply_tx)).is_err() {
            restore_if(&saved, copied_seq);
            finish();
            return;
        }
        let _ = PostThreadMessageW(thread_id, crate::hook::SELECT_CONVERT, WPARAM(0), LPARAM(0));
        let Some((converted, count, lang)) =
            reply_rx.recv_timeout(Duration::from_secs(2)).ok().flatten()
        else {
            restore_if(&saved, copied_seq);
            finish();
            return;
        };
        if count == 0 || converted == original || !permitted(hwnd, token, &privacy) {
            restore_if(&saved, copied_seq);
            finish();
            return;
        }
        if GetClipboardSequenceNumber() != copied_seq {
            finish();
            return;
        }
        if !write_text(&converted) {
            restore(&saved);
            finish();
            return;
        }
        let written_seq = GetClipboardSequenceNumber();
        if !permitted(hwnd, token, &privacy) {
            if GetClipboardSequenceNumber() == written_seq {
                restore(&saved);
            }
            finish();
            return;
        }
        if !send_chord(VK_V) {
            restore(&saved);
            finish();
            return;
        }
        let _ = PostThreadMessageW(thread_id, crate::hook::SELECT_COUNT, WPARAM(0), LPARAM(0));
        // Slow apps (Word) read the clipboard late; restoring early would paste the old content.
        thread::sleep(Duration::from_millis(500));
        if GetClipboardSequenceNumber() == written_seq {
            restore(&saved);
        }
        if permitted(hwnd, token, &privacy) {
            let target = if lang == Lang::Th { th } else { en };
            let _ = PostMessageW(
                Some(hwnd),
                WM_INPUTLANGCHANGEREQUEST,
                WPARAM(0),
                LPARAM(target.0 as isize),
            );
        }
        finish();
    });
}
unsafe fn permitted(hwnd: HWND, token: u64, privacy: &crate::uia::Privacy) -> bool {
    GetForegroundWindow() == hwnd
        && privacy.gate.valid(token)
        && privacy.allowed(hwnd) == Some(token)
}
fn finish() {
    OWNER.with(|o| {
        let owner = o.replace(HWND::default());
        if !owner.is_invalid() {
            let _ = unsafe { DestroyWindow(owner) };
        }
    });
    FLOW_ACTIVE.store(false, Ordering::SeqCst);
}
unsafe fn send_chord(key: VIRTUAL_KEY) -> bool {
    let events = [
        key_event(VK_CONTROL, false),
        key_event(key, false),
        key_event(key, true),
        key_event(VK_CONTROL, true),
    ];
    SendInput(&events, std::mem::size_of::<INPUT>() as i32) == events.len() as u32
}
unsafe fn key_event(key: VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                dwFlags: if up {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS(0)
                },
                dwExtraInfo: 0x4b46,
                ..Default::default()
            },
        },
    }
}
unsafe fn open() -> bool {
    for _ in 0..5 {
        let owner = OWNER.with(|o| o.get());
        if OpenClipboard((!owner.is_invalid()).then_some(owner)).is_ok() {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    false
}
unsafe fn skip(fmt: u32) -> bool {
    [
        CF_BITMAP.0 as u32,
        CF_PALETTE.0 as u32,
        CF_METAFILEPICT.0 as u32,
        CF_ENHMETAFILE.0 as u32,
        CF_OWNERDISPLAY.0 as u32,
        CF_DSPBITMAP.0 as u32,
        CF_DSPMETAFILEPICT.0 as u32,
        CF_DSPTEXT.0 as u32,
        CF_DSPENHMETAFILE.0 as u32,
    ]
    .contains(&fmt)
}
unsafe fn snapshot() -> Option<Vec<(u32, Vec<u8>)>> {
    if !open() {
        return None;
    }
    let mut all = Vec::new();
    let mut f = 0;
    loop {
        f = EnumClipboardFormats(f);
        if f == 0 {
            break;
        }
        if skip(f) {
            continue;
        }
        let Ok(handle) = GetClipboardData(f) else {
            continue;
        };
        let h = HGLOBAL(handle.0);
        let n = GlobalSize(h);
        if n == 0 || n > 64 * 1024 * 1024 {
            continue;
        }
        let p = GlobalLock(h);
        if p.is_null() {
            continue;
        }
        all.push((f, std::slice::from_raw_parts(p as *const u8, n).to_vec()));
        let _ = GlobalUnlock(h);
    }
    let _ = CloseClipboard();
    Some(all)
}
unsafe fn read_text() -> Option<String> {
    if !open() {
        return None;
    }
    let result = (|| {
        let h = GetClipboardData(CF_UNICODETEXT.0 as u32).ok()?;
        let p = GlobalLock(HGLOBAL(h.0)) as *const u16;
        if p.is_null() {
            return None;
        }
        let mut n = 0;
        while n <= 10000 && *p.add(n) != 0 {
            n += 1;
        }
        let out = if n > 5000 {
            None
        } else {
            String::from_utf16(std::slice::from_raw_parts(p, n)).ok()
        };
        let _ = GlobalUnlock(HGLOBAL(h.0));
        out
    })();
    let _ = CloseClipboard();
    result
}
unsafe fn global(bytes: &[u8]) -> Option<HGLOBAL> {
    let h = GlobalAlloc(GMEM_MOVEABLE, bytes.len()).ok()?;
    let p = GlobalLock(h);
    if p.is_null() {
        return None;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), p as *mut u8, bytes.len());
    let _ = GlobalUnlock(h);
    Some(h)
}
unsafe fn write_text(text: &str) -> bool {
    if !open() {
        return false;
    }
    let mut units: Vec<u16> = text.encode_utf16().collect();
    units.push(0);
    let bytes = std::slice::from_raw_parts(units.as_ptr() as *const u8, units.len() * 2);
    let ok = EmptyClipboard().is_ok()
        && global(bytes)
            .and_then(|h| SetClipboardData(CF_UNICODETEXT.0 as u32, Some(HANDLE(h.0))).ok())
            .is_some();
    for (name, data) in [
        (
            w!("ExcludeClipboardContentFromMonitorProcessing"),
            &[1u8][..],
        ),
        (w!("CanIncludeInClipboardHistory"), &0u32.to_le_bytes()[..]),
        (w!("CanUploadToCloudClipboard"), &0u32.to_le_bytes()[..]),
    ] {
        let id = RegisterClipboardFormatW(name);
        if id != 0 {
            if let Some(h) = global(data) {
                let _ = SetClipboardData(id, Some(HANDLE(h.0)));
            }
        }
    }
    let _ = CloseClipboard();
    ok
}
unsafe fn restore_if(saved: &[(u32, Vec<u8>)], expected: u32) {
    if GetClipboardSequenceNumber() == expected {
        restore(saved);
    }
}
unsafe fn restore(saved: &[(u32, Vec<u8>)]) {
    if !open() {
        return;
    }
    if EmptyClipboard().is_ok() {
        for (fmt, data) in saved {
            if let Some(h) = global(data) {
                let _ = SetClipboardData(*fmt, Some(HANDLE(h.0)));
            }
        }
    }
    let _ = CloseClipboard();
}
