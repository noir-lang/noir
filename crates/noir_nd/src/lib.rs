// This is the non determinism (nd) module.

pub mod c_header;
pub use c_header::C_ExternFuncCall;

use libloading::{Library, Symbol};
use std::path::Path;

// The outputs vector will always be empty and will be populated by the C code
pub fn to_c_extern_func_call(
    name: &std::ffi::CString,
    inputs: &Vec<[u8; 32]>,
    outputs: &mut Vec<[u8; 32]>,
) -> C_ExternFuncCall {
    C_ExternFuncCall {
        name: name.as_ptr(),
        inputs: inputs.as_ptr(),
        outputs: outputs.as_mut_ptr(),
    }
}

pub fn make_extern_call<P: AsRef<Path>>(
    path_to_dynamic_lib: P,
    name: String,
    inputs: &Vec<[u8; 32]>,
    mut outputs: Vec<[u8; 32]>,
) -> Vec<[u8; 32]> {
    let name_as_c_string = std::ffi::CString::new(name).unwrap();
    let c_struct = to_c_extern_func_call(&name_as_c_string, inputs, &mut outputs);
    let outputs_len = outputs.len();
    let outputs_cap = outputs.capacity();
    std::mem::forget(outputs);

    unsafe {
        // Load up the dynamic library
        let path = path_to_dynamic_lib.as_ref();

        let lib = Library::new(path).unwrap();

        // We assume the dynamic library has a method named `call_func`
        // and load up that functions symbol
        let f: Symbol<unsafe extern "C" fn(C_ExternFuncCall)> = lib.get(b"call_func\0").unwrap();

        // Call method which should modify the outputs struct
        f(c_struct);

        Vec::from_raw_parts(c_struct.outputs, outputs_len, outputs_cap)
    }
}
