// We compile this Rust project as a dynamic library to be used for stdlib non deterministic methods

mod test_lib;
mod wasm_lib;
use std::{ffi::CStr, os::raw::c_char};

#[no_mangle]
pub extern "C" fn call_func(name: *const c_char, inputs: *const [u8; 32], outputs: *mut [u8; 32]) {
    let foo = [3u8; 32];
    let foo2 = [5u8; 32];

    unsafe {
        *outputs.offset(0) = foo;
        *outputs.offset(1) = foo2;
    }

    let c_str: &CStr = unsafe { CStr::from_ptr(name) };
    let str_slice: &str = c_str.to_str().unwrap();
    println!("function called is named : {}", str_slice);
}
