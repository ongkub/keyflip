//! C99 ABI; see include/keyflip.h for ownership and JSON contracts.
//!
//! Each engine is single-threaded. Null pointers and invalid UTF-8 are handled;
//! non-null pointers must still point to valid objects/NUL-terminated strings.
//! Allocated strings must be released exactly once with `kf_string_free`.
use crate::{
    engine::Engine,
    keymap::Lang,
    tracker::{Action, Key},
};
use std::{
    collections::BTreeSet,
    ffi::{c_char, CStr, CString},
    marker::PhantomData,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr,
    rc::Rc,
};

pub struct KFEngine {
    engine: Engine,
    insert: Option<CString>,
    // Prevent safe Rust callers from moving/sharing the wrapper between threads.
    _single_threaded: PhantomData<Rc<()>>,
}
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KFAction {
    pub kind: u32,
    pub delete_count: u32,
    pub insert: *const c_char,
    pub switch_to: u32,
}
impl Default for KFAction {
    fn default() -> Self {
        Self {
            kind: 0,
            delete_count: 0,
            insert: ptr::null(),
            switch_to: 0,
        }
    }
}
fn guard<T>(fallback: T, f: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(fallback)
}
unsafe fn text<'a>(p: *const c_char) -> Result<&'a str, String> {
    if p.is_null() {
        return Err("null string pointer".into());
    }
    CStr::from_ptr(p)
        .to_str()
        .map_err(|e| format!("invalid UTF-8: {e}"))
}
unsafe fn optional_text<'a>(p: *const c_char) -> Result<Option<&'a str>, String> {
    if p.is_null() {
        Ok(None)
    } else {
        text(p).map(Some)
    }
}
fn owned_string(s: String) -> *mut c_char {
    CString::new(s)
        .map(CString::into_raw)
        .unwrap_or(ptr::null_mut())
}

/// # Safety
/// Strings must be valid NUL-terminated buffers; `error_out`, when non-null,
/// must be writable. The returned handle must be freed once, on the same thread.
#[no_mangle]
pub unsafe extern "C" fn kf_engine_new(
    models_dir: *const c_char,
    config_json: *const c_char,
    pairs_json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut KFEngine {
    guard(ptr::null_mut(), || {
        if !error_out.is_null() {
            *error_out = ptr::null_mut();
        }
        let result = catch_unwind(AssertUnwindSafe(|| {
            Engine::new(
                text(models_dir)?,
                optional_text(config_json)?,
                optional_text(pairs_json)?,
            )
        }))
        .unwrap_or_else(|_| Err("panic while creating KeyFlip engine".into()));
        match result {
            Ok(engine) => Box::into_raw(Box::new(KFEngine {
                engine,
                insert: None,
                _single_threaded: PhantomData,
            })),
            Err(error) => {
                if !error_out.is_null() {
                    *error_out = owned_string(error);
                }
                ptr::null_mut()
            }
        }
    })
}
/// # Safety
/// `e` must be null or a live handle from `kf_engine_new`, freed only once.
#[no_mangle]
pub unsafe extern "C" fn kf_engine_free(e: *mut KFEngine) {
    guard((), || {
        if !e.is_null() {
            drop(Box::from_raw(e));
        }
    });
}
/// # Safety
/// `e` must be null or live and exclusively accessed on its owning thread.
/// For character keys `text_ptr` must be null or a NUL-terminated buffer.
#[no_mangle]
pub unsafe extern "C" fn kf_key(
    e: *mut KFEngine,
    key_kind: u32,
    text_ptr: *const c_char,
) -> KFAction {
    guard(KFAction::default(), || {
        let Some(e) = e.as_mut() else {
            return KFAction::default();
        };
        e.insert = None;
        let key = match key_kind {
            0 => match text(text_ptr) {
                Ok(s) => Key::Char(s.into()),
                Err(_) => return KFAction::default(),
            },
            1 => Key::Space,
            2 => Key::Tab,
            3 => Key::Return,
            4 => Key::Backspace,
            5 => Key::Undo,
            6 => Key::Reset,
            _ => return KFAction::default(),
        };
        let (kind, delete, insert, switch_to) = match e.engine.handle(key) {
            Action::Pass => return KFAction::default(),
            Action::Replace {
                delete,
                insert,
                switch_to,
            } => (1, delete, insert, switch_to),
            Action::Revert {
                delete,
                insert,
                switch_to,
            } => (2, delete, insert, switch_to),
        };
        let Ok(insert) = CString::new(insert) else {
            return KFAction::default();
        };
        let Ok(delete_count) = u32::try_from(delete) else {
            return KFAction::default();
        };
        e.insert = Some(insert);
        KFAction {
            kind,
            delete_count,
            insert: e.insert.as_ref().unwrap().as_ptr(),
            switch_to: if switch_to == Lang::En { 1 } else { 0 },
        }
    })
}
/// # Safety
/// `e` must be null or a live, exclusively accessed handle on its owning thread.
#[no_mangle]
pub unsafe extern "C" fn kf_take_events(e: *mut KFEngine) -> *mut c_char {
    guard(ptr::null_mut(), || {
        let Some(e) = e.as_mut() else {
            return ptr::null_mut();
        };
        serde_json::to_string(&e.engine.take_events())
            .map(owned_string)
            .unwrap_or(ptr::null_mut())
    })
}
/// # Safety
/// `e` must be null or a live handle on its owning thread.
#[no_mangle]
pub unsafe extern "C" fn kf_never_get(e: *mut KFEngine) -> *mut c_char {
    guard(ptr::null_mut(), || {
        let Some(e) = e.as_ref() else {
            return ptr::null_mut();
        };
        serde_json::to_string(e.engine.never())
            .map(owned_string)
            .unwrap_or(ptr::null_mut())
    })
}
/// # Safety
/// `e` must be null or live and exclusively accessed on its owning thread;
/// `json_array` must be null or a NUL-terminated buffer. Invalid input is ignored.
#[no_mangle]
pub unsafe extern "C" fn kf_never_set(e: *mut KFEngine, json_array: *const c_char) {
    guard((), || {
        let Some(e) = e.as_mut() else {
            return;
        };
        if let Ok(json) = text(json_array) {
            if let Ok(never) = serde_json::from_str::<BTreeSet<String>>(json) {
                e.engine.set_never(never);
            }
        }
    });
}
/// # Safety
/// `e` must be null or a live, exclusively accessed handle on its owning thread.
#[no_mangle]
pub unsafe extern "C" fn kf_never_clear(e: *mut KFEngine) {
    guard((), || {
        if let Some(e) = e.as_mut() {
            e.engine.clear_never();
        }
    });
}
/// # Safety
/// `s` must be null or an unfreed string returned by this ABI (not action.insert
/// or kf_version). It must be freed exactly once.
#[no_mangle]
pub unsafe extern "C" fn kf_string_free(s: *mut c_char) {
    guard((), || {
        if !s.is_null() {
            drop(CString::from_raw(s));
        }
    });
}
#[no_mangle]
pub extern "C" fn kf_version() -> *const c_char {
    // A static, NUL-terminated string; no allocation or fallible operation.
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr().cast()
}

#[cfg(test)]
mod tests {
    #[test]
    fn catches_panics() {
        assert_eq!(super::guard(42, || panic!("test panic boundary")), 42);
    }
}
