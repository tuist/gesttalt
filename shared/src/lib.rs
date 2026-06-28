use std::ffi::{CStr, CString};
use std::os::raw::c_char;

mod core;

#[cfg(target_os = "android")]
mod android;

/// # Safety
///
/// `name` must be null or point to a valid, nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn shared_greeting(name: *const c_char) -> *mut c_char {
    let name = if name.is_null() {
        "iOS".to_string()
    } else {
        unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned()
    };

    string_to_c(core::greeting(&name))
}

#[no_mangle]
pub extern "C" fn shared_lattice_score(seed: i32) -> i32 {
    core::lattice_score(seed)
}

/// # Safety
///
/// `value` must be null or a pointer returned by `shared_greeting` that has not
/// already been freed.
#[no_mangle]
pub unsafe extern "C" fn shared_string_free(value: *mut c_char) {
    if value.is_null() {
        return;
    }

    unsafe {
        drop(CString::from_raw(value));
    }
}

fn string_to_c(value: String) -> *mut c_char {
    let sanitized = value.replace('\0', "");
    CString::new(sanitized)
        .expect("sanitized strings do not contain nul bytes")
        .into_raw()
}
