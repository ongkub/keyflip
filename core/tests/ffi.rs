mod common;
use common::{Models, CONFIG, PAIRS};
use keyflip_core::ffi::*;
use serde_json::{json, Value};
use std::{
    ffi::{c_char, CStr, CString},
    ptr,
};
fn c(s: &str) -> CString {
    CString::new(s).unwrap()
}
unsafe fn json_string(p: *mut c_char) -> Value {
    assert!(!p.is_null());
    let parsed = serde_json::from_str(CStr::from_ptr(p).to_str().unwrap()).unwrap();
    kf_string_free(p);
    parsed
}
struct Handle(*mut KFEngine);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            kf_engine_free(self.0);
        }
    }
}
unsafe fn engine(dir: &Models) -> Handle {
    let mut error = ptr::null_mut();
    let e = kf_engine_new(
        c(dir.0.to_str().unwrap()).as_ptr(),
        c(CONFIG).as_ptr(),
        c(PAIRS).as_ptr(),
        &mut error,
    );
    assert!(!e.is_null());
    assert!(error.is_null());
    Handle(e)
}
#[test]
fn c_abi_actions_events_strings_and_persistence() {
    unsafe {
        let dir = Models::new();
        let e = engine(&dir);
        assert_eq!(
            CStr::from_ptr(kf_version()).to_str().unwrap(),
            env!("CARGO_PKG_VERSION")
        );
        assert_eq!(kf_key(e.0, 0, c("a").as_ptr()).kind, 0);
        let action = kf_key(e.0, 0, c("b").as_ptr());
        assert_eq!(
            (action.kind, action.delete_count, action.switch_to),
            (1, 1, 0)
        );
        assert_eq!(CStr::from_ptr(action.insert).to_str().unwrap(), "กข");
        let events = json_string(kf_take_events(e.0));
        assert_eq!(events[0]["event"], "converted");
        assert_eq!(events[0]["kind"], "typing");
        assert_eq!(events[0]["word"], "ab");
        assert_eq!(events[0]["result"], "กข");
        assert_eq!(events[0]["key"], 2);
        assert!(events[0]["prev"].is_null());
        assert_eq!(events[0]["from"], "en");
        assert_eq!(events[0]["to"], "th");
        assert!(events[0]["score"].is_number());
        // Event/never calls must preserve the action's borrowed pointer.
        assert_eq!(json_string(kf_take_events(e.0)), json!([]));
        assert_eq!(json_string(kf_never_get(e.0)), json!([]));
        assert_eq!(CStr::from_ptr(action.insert).to_str().unwrap(), "กข");
        assert_eq!(kf_key(e.0, 2, ptr::null()).kind, 0);
        let undo = kf_key(e.0, 5, ptr::null());
        assert_eq!((undo.kind, undo.delete_count, undo.switch_to), (2, 3, 1));
        assert_eq!(CStr::from_ptr(undo.insert).to_str().unwrap(), "ab\t");
        assert_eq!(
            json_string(kf_take_events(e.0)),
            json!([
                {"event":"runEnded","lang":"th"},
                {"event":"reverted","kind":"shift","from":"th","to":"en","word":"กข","result":"ab"}
            ])
        );
        assert_eq!(json_string(kf_never_get(e.0)), json!(["ab"]));
        kf_never_set(e.0, c(r#"["z","a","z","ไทย"]"#).as_ptr());
        assert_eq!(json_string(kf_never_get(e.0)), json!(["a", "z", "ไทย"]));
        for invalid in ["{}", "[1]", "null", "bad"] {
            kf_never_set(e.0, c(invalid).as_ptr());
        }
        assert_eq!(json_string(kf_never_get(e.0)), json!(["a", "z", "ไทย"]));
        kf_never_clear(e.0);
        assert_eq!(json_string(kf_never_get(e.0)), json!([]));
        assert_eq!(kf_key(e.0, 6, ptr::null()).kind, 0);
        assert_eq!(kf_key(e.0, 0, c("a").as_ptr()).kind, 0);
        assert_eq!(kf_key(e.0, 3, ptr::null()).kind, 0);
        assert_eq!(
            json_string(kf_take_events(e.0)),
            json!([{"event":"runEnded","lang":"en"}])
        );
        kf_key(e.0, 0, c("a").as_ptr());
        kf_key(e.0, 0, c("b").as_ptr());
        let undo = kf_key(e.0, 4, ptr::null());
        assert_eq!(undo.kind, 2);
        assert_eq!(CStr::from_ptr(undo.insert).to_str().unwrap(), "ab");
        // C struct offsets, independent of target pointer width/alignment.
        let base = (&action as *const KFAction) as usize;
        assert_eq!((&action.kind as *const _ as usize) - base, 0);
        assert_eq!((&action.delete_count as *const _ as usize) - base, 4);
        assert_eq!((&action.insert as *const _ as usize) - base, 8);
        assert_eq!(
            (&action.switch_to as *const _ as usize) - base,
            8 + std::mem::size_of::<*const c_char>()
        );
    }
}
#[test]
fn c_abi_nulls_invalid_utf8_and_errors() {
    unsafe {
        let bad = [0xffu8, 0];
        let bad = bad.as_ptr().cast();
        for kind in 0..=7 {
            let a = kf_key(ptr::null_mut(), kind, ptr::null());
            assert_eq!((a.kind, a.delete_count, a.switch_to), (0, 0, 0));
            assert!(a.insert.is_null());
        }
        assert!(kf_take_events(ptr::null_mut()).is_null());
        assert!(kf_never_get(ptr::null_mut()).is_null());
        kf_never_set(ptr::null_mut(), bad);
        kf_never_clear(ptr::null_mut());
        kf_engine_free(ptr::null_mut());
        kf_string_free(ptr::null_mut());
        let mut error = ptr::null_mut();
        assert!(kf_engine_new(ptr::null(), ptr::null(), ptr::null(), &mut error).is_null());
        assert!(!error.is_null());
        kf_string_free(error);
        assert!(kf_engine_new(bad, ptr::null(), ptr::null(), ptr::null_mut()).is_null());
        let dir = Models::new();
        let path = c(dir.0.to_str().unwrap());
        for (config, pairs) in [(bad, ptr::null()), (ptr::null(), bad)] {
            assert!(kf_engine_new(path.as_ptr(), config, pairs, &mut error).is_null());
            assert!(!error.is_null());
            kf_string_free(error);
        }
        for malformed in ["bad", "[]", r#"{"never":[false]}"#] {
            assert!(kf_engine_new(
                path.as_ptr(),
                c(malformed).as_ptr(),
                ptr::null(),
                &mut error
            )
            .is_null());
            assert!(!error.is_null());
            kf_string_free(error);
        }
        let e = engine(&dir);
        for text in [bad, ptr::null()] {
            assert_eq!(kf_key(e.0, 0, text).kind, 0);
            kf_never_set(e.0, text);
        }
        assert_eq!(kf_key(e.0, u32::MAX, bad).kind, 0);
        assert_eq!(json_string(kf_take_events(e.0)), json!([]));
        assert_eq!(json_string(kf_never_get(e.0)), json!([]));
        // A successful constructor clears a preexisting error_out value.
        error = std::ptr::dangling_mut();
        let defaults = kf_engine_new(path.as_ptr(), ptr::null(), ptr::null(), &mut error);
        assert!(!defaults.is_null());
        assert!(error.is_null());
        kf_engine_free(defaults);
    }
}
