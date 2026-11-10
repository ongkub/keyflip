use crate::{exclude, privacy::Gate};
use std::sync::{
    atomic::{AtomicIsize, Ordering},
    mpsc::{self, Sender},
    Arc, Mutex,
};
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::{CloseHandle, HWND},
        System::{
            Com::{
                CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
                COINIT_MULTITHREADED,
            },
            Threading::{
                OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
                PROCESS_QUERY_LIMITED_INFORMATION,
            },
            Variant::{VariantClear, VariantToBoolean, VT_BOOL},
        },
        UI::{
            Accessibility::{CUIAutomation, IUIAutomation, UIA_IsPasswordPropertyId},
            WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
        },
    },
};

pub struct Privacy {
    pub gate: Gate,
    hwnd: AtomicIsize,
    app: Mutex<String>,
}
impl Privacy {
    pub fn new() -> Self {
        Self {
            gate: Gate::default(),
            hwnd: AtomicIsize::new(0),
            app: Mutex::new(String::new()),
        }
    }
    pub fn allowed(&self, hwnd: HWND) -> Option<u64> {
        let token = self.gate.token()?;
        (self.hwnd.load(Ordering::SeqCst) == hwnd.0 as isize).then_some(token)
    }
    pub fn app(&self) -> String {
        self.app.lock().unwrap().clone()
    }
}

pub fn start(privacy: Arc<Privacy>, exclusions: Vec<String>) -> Sender<u64> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || unsafe {
        if CoInitializeEx(None, COINIT_MULTITHREADED).is_err() {
            return;
        }
        let automation: windows::core::Result<IUIAutomation> =
            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER);
        if let Ok(automation) = automation {
            while let Ok(mut generation) = rx.recv() {
                // Only the newest focus matters. A slow or failed provider keeps us disabled.
                for newer in rx.try_iter() {
                    generation = newer;
                }
                if !privacy.gate.current(generation) {
                    continue;
                }
                let hwnd = GetForegroundWindow();
                let Some(app) = process_name(hwnd) else {
                    continue;
                };
                if exclude::excluded(&app, &exclusions) {
                    continue;
                }
                let safe = (|| -> windows::core::Result<bool> {
                    let element = automation.GetFocusedElement()?;
                    let mut pid = 0;
                    GetWindowThreadProcessId(hwnd, Some(&mut pid));
                    if element.CurrentProcessId()? as u32 != pid {
                        return Ok(false);
                    }
                    let mut property =
                        element.GetCurrentPropertyValueEx(UIA_IsPasswordPropertyId, true)?;
                    // A missing/unsupported property is not evidence of safety.
                    let password = if property.Anonymous.Anonymous.vt == VT_BOOL {
                        VariantToBoolean(&property).ok()
                    } else {
                        None
                    };
                    let _ = VariantClear(&mut property);
                    Ok(password.is_some_and(|value| !value.as_bool()))
                })()
                .unwrap_or(false);
                if safe && GetForegroundWindow() == hwnd && privacy.gate.current(generation) {
                    *privacy.app.lock().unwrap() = app;
                    privacy.hwnd.store(hwnd.0 as isize, Ordering::SeqCst);
                    privacy.gate.publish_safe(generation);
                }
            }
            drop(automation);
        }
        CoUninitialize();
    });
    tx
}

unsafe fn process_name(hwnd: HWND) -> Option<String> {
    if hwnd.0.is_null() {
        return None;
    }
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
    let mut path = vec![0u16; 32768];
    let mut size = path.len() as u32;
    let result = QueryFullProcessImageNameW(
        process,
        PROCESS_NAME_WIN32,
        PWSTR(path.as_mut_ptr()),
        &mut size,
    );
    let _ = CloseHandle(process);
    result.ok()?;
    let path = String::from_utf16(&path[..size as usize]).ok()?;
    Some(path.rsplit('\\').next()?.to_owned())
}
