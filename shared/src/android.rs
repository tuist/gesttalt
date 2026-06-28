use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};
use jni::JNIEnv;

use crate::core;

#[no_mangle]
pub extern "system" fn Java_com_gesttalt_app_SharedBridge_greeting(
    mut env: JNIEnv,
    _class: JClass,
    name: JString,
) -> jstring {
    let name = env
        .get_string(&name)
        .map(|value| value.into())
        .unwrap_or_else(|_| "Android".to_string());
    let output = core::greeting(&name);
    env.new_string(output)
        .map(|value| value.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_gesttalt_app_SharedBridge_latticeScore(
    _env: JNIEnv,
    _class: JClass,
    seed: jint,
) -> jint {
    core::lattice_score(seed)
}
