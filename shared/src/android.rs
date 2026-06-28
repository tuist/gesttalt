use std::ffi::c_void;

use crate::core;

#[no_mangle]
pub extern "system" fn Java_com_gesttalt_app_SharedBridge_latticeScore(
    _env: *mut c_void,
    _class: *mut c_void,
    seed: i32,
) -> i32 {
    core::lattice_score(seed)
}
