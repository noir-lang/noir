// We compile this Rust project as a dynamic library to be used for stdlib non deterministic methods

mod test_lib;

use noir_nd::c_header::C_ExternFuncCall;
use std::ffi::CStr;

#[no_mangle]
pub extern "C" fn call_func(x: C_ExternFuncCall) {
    let foo = [3u8; 32];
    let foo2 = [5u8; 32];

    unsafe {
        *x.outputs.offset(0) = foo;
        *x.outputs.offset(1) = foo2;
    }

    let c_str: &CStr = unsafe { CStr::from_ptr(x.name) };
    let str_slice: &str = c_str.to_str().unwrap();
    println!("function called is named : {}", str_slice);
}
