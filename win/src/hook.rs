use crate::{
    inject,
    keys::{self, LayoutHistory, Modifiers},
    pairs::{self, Layout},
    shift_tap::ShiftTap,
    store::{self, Store},
    uia::{self, Privacy},
};
use keyflip_core::{
    engine::Engine,
    tracker::{Action, Key, Lang, TrackerEvent},
};
use std::{
    cell::RefCell,
    collections::{BTreeSet, VecDeque},
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Instant,
};
use windows::{
    core::w,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
        UI::{Accessibility::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
};

pub const WORK: u32 = WM_APP + 1;
pub const TOGGLE: u32 = WM_APP + 2;
pub const CLEAR: u32 = WM_APP + 3;
pub const STOP: u32 = WM_APP + 4;

#[derive(Default, Clone)]
pub struct Snapshot {
    pub enabled: bool,
    pub counts: (u64, u64, u64),
    pub day: String,
    pub never: usize,
    pub notice: Option<String>,
}
pub type Shared = Arc<Mutex<Snapshot>>;
enum Save {
    Record(Vec<TrackerEvent>, Vec<String>, String),
    Enabled(bool),
    Clear,
}
fn storage(mut store: Store, shared: Shared) -> (Sender<Save>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let join = std::thread::spawn(move || {
        while let Ok(command) = rx.recv() {
            let result = match command {
                Save::Record(events, learned, app) => {
                    let mut never = store.never.clone();
                    never.extend(learned);
                    store
                        .record(events, &app, store::now())
                        .and_then(|()| store.set_never(never))
                }
                Save::Enabled(enabled) => {
                    store.settings.enabled = enabled;
                    store.save_settings()
                }
                Save::Clear => store.set_never(BTreeSet::new()),
            };
            let mut snapshot = shared.lock().unwrap();
            snapshot.day = store::timestamp(store::now()).0;
            snapshot.counts = store.counts(&snapshot.day);
            snapshot.never = store.never.len();
            if let Err(error) = result {
                snapshot.notice = Some(format!("บันทึกข้อมูลไม่สำเร็จ: {error}"));
            }
        }
        let _ = store.flush();
    });
    (tx, join)
}

struct Job {
    action: Action,
    events: Vec<TrackerEvent>,
    learned: Vec<String>,
    token: u64,
    hwnd: HWND,
}
struct Runtime {
    engine: Option<Engine>,
    enabled: bool,
    en: HKL,
    th: HKL,
    layouts: LayoutHistory,
    shift: ShiftTap,
    clock: Instant,
    privacy: Arc<Privacy>,
    query: Sender<u64>,
    save: Sender<Save>,
    jobs: VecDeque<Job>,
    thread_id: u32,
}
thread_local! { static STATE: RefCell<Option<Runtime>> = const { RefCell::new(None) }; }
impl Runtime {
    fn reset(&mut self) {
        if let Some(engine) = &mut self.engine {
            engine.handle(Key::Reset);
            engine.take_events();
        }
        self.shift.clear();
        self.jobs.clear();
        self.layouts.clear();
    }
    fn focus(&mut self) {
        let generation = self.privacy.gate.invalidate();
        self.reset();
        let _ = self.query.send(generation);
    }
    unsafe fn enqueue(&mut self, key: Key, token: u64, hwnd: HWND) -> bool {
        if key == Key::Reset {
            self.reset();
            return false;
        }
        let Some(engine) = &mut self.engine else {
            return false;
        };
        let action = engine.handle(key);
        let swallow = !matches!(action, Action::Pass);
        let events = engine.take_events();
        if swallow || !events.is_empty() {
            let learned = events
                .iter()
                .filter_map(|event| match event {
                    TrackerEvent::Reverted { result, .. } if engine.never().contains(result) => {
                        Some(result.clone())
                    }
                    _ => None,
                })
                .collect();
            self.jobs.push_back(Job {
                action,
                events,
                learned,
                token,
                hwnd,
            });
            if PostThreadMessageW(self.thread_id, WORK, WPARAM(0), LPARAM(0)).is_err() {
                self.reset();
                return false;
            }
        }
        swallow
    }
    unsafe fn keyboard(&mut self, h: &KBDLLHOOKSTRUCT, down: bool) -> bool {
        let vk = h.vkCode;
        let hwnd = GetForegroundWindow();
        let token = self.privacy.allowed(hwnd);
        if !self.enabled || token.is_none() {
            self.reset();
            return false;
        }
        let layout = GetKeyboardLayout(GetWindowThreadProcessId(hwnd, None));
        let language = layout.0 as usize as u16;
        if !keys::supported_language(language) || (language == 0x041e && layout != self.th) {
            self.reset();
            return false;
        }
        if self.layouts.observe(layout.0 as usize) {
            self.reset();
        }
        let modifiers = Modifiers {
            ctrl: pressed(VK_CONTROL) || (down && matches!(vk, 0x11 | 0xa2 | 0xa3)),
            alt: pressed(VK_MENU) || (down && matches!(vk, 0x12 | 0xa4 | 0xa5)),
            win: pressed(VK_LWIN) || pressed(VK_RWIN) || (down && matches!(vk, 0x5b | 0x5c)),
        };
        if keys::is_shift(vk) {
            if modifiers.ctrl || modifiers.alt || modifiers.win {
                self.reset();
                return false;
            }
            if self
                .shift
                .event(vk, down, self.clock.elapsed().as_millis() as u64)
            {
                self.enqueue(Key::Undo, token.unwrap(), hwnd);
                // The gesture triggers on key-up, which carries no text. Pass
                // the release so the preceding passed Shift-down stays balanced.
                return false;
            }
            return false;
        }
        if !down {
            return false;
        }
        self.shift.clear();
        let key = if modifiers.ctrl || modifiers.alt || modifiers.win {
            Key::Reset
        } else {
            let mut state = [0u8; 256];
            if pressed(VK_SHIFT) {
                state[VK_SHIFT.0 as usize] = 0x80;
            }
            state[VK_CAPITAL.0 as usize] = (GetKeyState(VK_CAPITAL.0 as i32) & 1) as u8;
            state[VK_NUMLOCK.0 as usize] = (GetKeyState(VK_NUMLOCK.0 as i32) & 1) as u8;
            state[(vk as usize).min(255)] |= 0x80;
            let text = translate(vk, h.scanCode, &state, layout);
            keys::classify(vk, modifiers, text)
        };
        self.enqueue(key, token.unwrap(), hwnd)
    }
    unsafe fn work(&mut self) {
        while let Some(job) = self.jobs.pop_front() {
            if !self.enabled
                || !self.privacy.gate.valid(job.token)
                || GetForegroundWindow() != job.hwnd
                || self.privacy.allowed(job.hwnd) != Some(job.token)
            {
                self.reset();
                break;
            }
            match &job.action {
                Action::Pass => {}
                Action::Replace {
                    delete,
                    insert,
                    switch_to,
                }
                | Action::Revert {
                    delete,
                    insert,
                    switch_to,
                } => {
                    let target = if *switch_to == Lang::Th {
                        self.th
                    } else {
                        self.en
                    };
                    if !inject::replace(job.hwnd, *delete, insert, target) {
                        self.reset(); // Includes UIPI / partially accepted input: never log a failed conversion.
                        break;
                    }
                    self.layouts.expect(target.0 as usize);
                }
            }
            let _ = self
                .save
                .send(Save::Record(job.events, job.learned, self.privacy.app()));
        }
    }
}
unsafe fn pressed(vk: VIRTUAL_KEY) -> bool {
    GetAsyncKeyState(vk.0 as i32) < 0
}
unsafe fn translate(vk: u32, scan: u32, state: &[u8; 256], layout: HKL) -> Option<String> {
    let mut text = [0u16; 16];
    let len = ToUnicodeEx(vk, scan, state, &mut text, 0x4, Some(layout));
    if len <= 0 || len as usize > text.len() {
        None
    } else {
        String::from_utf16(&text[..len as usize]).ok()
    }
}
unsafe extern "system" fn keyboard(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code >= 0 {
        let h = &*(lp.0 as *const KBDLLHOOKSTRUCT);
        // Filter BEFORE accessing state, including reentrant SendInput callbacks.
        if !h.flags.contains(LLKHF_INJECTED) {
            let message = wp.0 as u32;
            if matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP) {
                let swallow = STATE.with(|state| {
                    state
                        .try_borrow_mut()
                        .ok()
                        .and_then(|mut s| {
                            s.as_mut().map(|s| {
                                s.keyboard(h, matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN))
                            })
                        })
                        .unwrap_or(false)
                });
                if swallow {
                    return LRESULT(1);
                }
            }
        }
    }
    CallNextHookEx(None, code, wp, lp)
}
unsafe extern "system" fn mouse(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code >= 0
        && matches!(
            wp.0 as u32,
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN
        )
    {
        STATE.with(|state| {
            if let Ok(mut state) = state.try_borrow_mut() {
                if let Some(state) = state.as_mut() {
                    state.focus();
                }
            }
        });
    }
    CallNextHookEx(None, code, wp, lp)
}
unsafe extern "system" fn focus(_: HWINEVENTHOOK, _: u32, _: HWND, _: i32, _: i32, _: u32, _: u32) {
    STATE.with(|state| {
        if let Ok(mut state) = state.try_borrow_mut() {
            if let Some(state) = state.as_mut() {
                state.focus();
            }
        }
    });
}

unsafe fn layouts() -> Result<(HKL, HKL, String), String> {
    let count = GetKeyboardLayoutList(None);
    let mut loaded = vec![HKL::default(); count.max(0) as usize];
    GetKeyboardLayoutList(Some(&mut loaded));
    let th = LoadKeyboardLayoutW(w!("0000041E"), KLF_NOTELLSHELL).map_err(|_| {
        "ไม่พบแป้นพิมพ์ไทย Kedmanee กรุณาเพิ่มภาษาไทยใน Windows แล้วเปิด KeyFlip ใหม่".to_string()
    })?;
    let en = loaded
        .iter()
        .find(|h| (h.0 as usize as u16) & 0x03ff == 9)
        .copied()
        .or_else(|| LoadKeyboardLayoutW(w!("00000409"), KLF_NOTELLSHELL).ok())
        .ok_or("ไม่พบแป้นพิมพ์อังกฤษ")?;
    let pairs = pairs::build(|scan, shifted, layout| {
        let hkl = if layout == Layout::English { en } else { th };
        let vk = MapVirtualKeyExW(scan, MAPVK_VSC_TO_VK_EX, Some(hkl));
        if vk == 0 {
            return None;
        }
        let mut state = [0u8; 256];
        if shifted {
            state[VK_SHIFT.0 as usize] = 0x80;
        }
        let text = translate(vk, scan, &state, hkl)?;
        let mut chars = text.chars();
        let c = chars.next()?;
        chars.next().is_none().then_some(c)
    });
    if pairs.is_empty() {
        return Err("สร้างตารางแป้นพิมพ์ไม่ได้".into());
    }
    Ok((en, th, serde_json::to_string(&pairs).unwrap()))
}

pub struct Hook {
    pub id: u32,
    join: JoinHandle<()>,
}
impl Hook {
    pub fn post(&self, message: u32) {
        unsafe {
            let _ = PostThreadMessageW(self.id, message, WPARAM(0), LPARAM(0));
        }
    }
    pub fn stop(self) {
        self.post(STOP);
        let _ = self.join.join();
    }
}
pub fn start(store: Store, shared: Shared) -> Result<Hook, String> {
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let join = std::thread::spawn(move || unsafe {
        let id = GetCurrentThreadId();
        let mut message = MSG::default();
        let _ = PeekMessageW(&mut message, None, 0, 0, PM_NOREMOVE); // Create thread queue before publishing id.
        let executable = std::env::current_exe().unwrap_or_default();
        let models = store.models(executable.parent().unwrap_or(std::path::Path::new(".")));
        let setup = layouts().and_then(|(en, th, pairs)| {
            Engine::new(
                models,
                Some(&store.settings.engine_config(&store.never)),
                Some(&pairs),
            )
            .map(|engine| (engine, en, th))
        });
        let (engine, en, th, enabled) = match setup {
            Ok((engine, en, th)) => (Some(engine), en, th, store.settings.enabled),
            Err(error) => {
                shared.lock().unwrap().notice = Some(error);
                (None, HKL::default(), HKL::default(), false)
            }
        };
        shared.lock().unwrap().enabled = enabled;
        let privacy = Arc::new(Privacy::new());
        let query = uia::start(privacy.clone(), store.settings.excluded_processes.clone());
        let (save, writer) = storage(store, shared.clone());
        STATE.with(|state| {
            *state.borrow_mut() = Some(Runtime {
                engine,
                enabled,
                en,
                th,
                layouts: LayoutHistory::default(),
                shift: ShiftTap::default(),
                clock: Instant::now(),
                privacy,
                query,
                save,
                jobs: VecDeque::new(),
                thread_id: id,
            })
        });
        let module = GetModuleHandleW(None).ok().map(Into::into);
        let keyboard_hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard), module, 0);
        let mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse), module, 0);
        let foreground_hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(focus),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );
        let focus_hook = SetWinEventHook(
            EVENT_OBJECT_FOCUS,
            EVENT_OBJECT_FOCUS,
            None,
            Some(focus),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );
        let installed = keyboard_hook.is_ok()
            && mouse_hook.is_ok()
            && !foreground_hook.0.is_null()
            && !focus_hook.0.is_null();
        let _ = ready_tx.send(if installed {
            Ok(id)
        } else {
            Err("ติดตั้ง keyboard/focus hook ไม่สำเร็จ".to_string())
        });
        if installed {
            STATE.with(|state| state.borrow_mut().as_mut().unwrap().focus());
            loop {
                let result = GetMessageW(&mut message, None, 0, 0).0;
                if result <= 0 || message.message == STOP {
                    break;
                }
                STATE.with(|state| {
                    let mut state = state.borrow_mut();
                    let s = state.as_mut().unwrap();
                    match message.message {
                        WORK => s.work(),
                        TOGGLE => {
                            s.reset();
                            if s.engine.is_some() {
                                s.enabled = !s.enabled;
                                let _ = s.save.send(Save::Enabled(s.enabled));
                                shared.lock().unwrap().enabled = s.enabled;
                            }
                        }
                        CLEAR => {
                            s.reset();
                            if let Some(engine) = &mut s.engine {
                                engine.clear_never();
                            }
                            let _ = s.save.send(Save::Clear);
                        }
                        _ => {}
                    }
                });
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        if let Ok(h) = keyboard_hook {
            let _ = UnhookWindowsHookEx(h);
        }
        if let Ok(h) = mouse_hook {
            let _ = UnhookWindowsHookEx(h);
        }
        if !foreground_hook.0.is_null() {
            let _ = UnhookWinEvent(foreground_hook);
        }
        if !focus_hook.0.is_null() {
            let _ = UnhookWinEvent(focus_hook);
        }
        STATE.with(|state| {
            state.borrow_mut().take();
        }); // Closes writer channel; UIA worker may be in a slow provider.
        let _ = writer.join();
    });
    match ready_rx.recv() {
        Ok(Ok(id)) => Ok(Hook { id, join }),
        Ok(Err(error)) => {
            let _ = join.join();
            Err(error)
        }
        Err(error) => {
            let _ = join.join();
            Err(error.to_string())
        }
    }
}
