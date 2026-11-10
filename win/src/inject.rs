use crate::input::{plan, Stroke};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::{
        Input::KeyboardAndMouse::*,
        WindowsAndMessaging::{PostMessageW, WM_INPUTLANGCHANGEREQUEST},
    },
};

/// Called only by the hook thread's posted-message handler, never its callback.
/// A short SendInput result includes UIPI failures. The caller drops events and resets.
pub unsafe fn replace(hwnd: HWND, delete: usize, text: &str, layout: HKL) -> bool {
    let inputs: Vec<INPUT> = plan(delete, text)
        .into_iter()
        .map(|stroke| {
            let (vk, scan, flags) = match stroke {
                Stroke::Backspace(down) => (
                    VK_BACK,
                    0,
                    if down {
                        KEYBD_EVENT_FLAGS(0)
                    } else {
                        KEYEVENTF_KEYUP
                    },
                ),
                Stroke::Unicode(unit, down) => (
                    VIRTUAL_KEY(0),
                    unit,
                    KEYEVENTF_UNICODE
                        | if down {
                            KEYBD_EVENT_FLAGS(0)
                        } else {
                            KEYEVENTF_KEYUP
                        },
                ),
            };
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk,
                        wScan: scan,
                        dwFlags: flags,
                        dwExtraInfo: 0x4b46,
                        ..Default::default()
                    },
                },
            }
        })
        .collect();
    if !inputs.is_empty() && SendInput(&inputs, size_of::<INPUT>() as i32) != inputs.len() as u32 {
        return false;
    }
    PostMessageW(
        Some(hwnd),
        WM_INPUTLANGCHANGEREQUEST,
        WPARAM(0),
        LPARAM(layout.0 as isize),
    )
    .is_ok()
}
