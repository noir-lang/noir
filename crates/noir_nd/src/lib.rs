// This is the non determinism (nd) module.

pub mod c_header;
mod wasm_loader;
pub use noir_field::FieldElement;

use libloading::{Library, Symbol};
use std::{os::raw::c_char, path::Path};

pub type CExternInput = (*const c_char, *const [u8; 32], *mut [u8; 32]);

// The outputs vector will always be empty and will be populated by the C code
pub fn to_c_extern_func_call(
    name: &std::ffi::CString,
    inputs: &Vec<[u8; 32]>,
    outputs: &mut [[u8; 32]],
) -> (*const c_char, *const [u8; 32], *mut [u8; 32]) {
    (name.as_ptr(), inputs.as_ptr(), outputs.as_mut_ptr())
}
// Note: We have two choices on how we load libraries
// We could specify that DLLs are for native and wasm is for web
// This would allow us to use conditional compilation to only compile the wasm and dll functions when we needed
// This code does an alternative instead; we compile both and check the path extension
pub fn make_extern_call<P: AsRef<Path>>(
    path: P,
    name: String,
    inputs: &Vec<[u8; 32]>,
    outputs: &mut [[u8; 32]],
) {
    // To see if we need to load a WASM file or a DLL, we can check the extension of the path
    let ext = path.as_ref().extension().expect("file has no extension");
    if ext == "wasm" {
        make_extern_call_wasm(path, name, inputs, outputs)
    } else {
        make_extern_call_dll(path, name, inputs, outputs)
    }
}
pub fn make_extern_call_wasm<P: AsRef<Path>>(
    path: P,
    name: String,
    inputs: &Vec<[u8; 32]>,
    outputs: &mut [[u8; 32]],
) {
    todo!("WASM compilation is not yet implemented")
}

fn make_extern_call_dll<P: AsRef<Path>>(
    path_to_dynamic_lib: P,
    name: String,
    inputs: &Vec<[u8; 32]>,
    outputs: &mut [[u8; 32]],
) {
    let name_as_c_string = std::ffi::CString::new(name).unwrap();
    let (c_name, c_inputs, c_outputs) = to_c_extern_func_call(&name_as_c_string, inputs, outputs);

    unsafe {
        // Load up the dynamic library
        let path = path_to_dynamic_lib.as_ref();

        let lib = Library::new(path).unwrap();

        // We assume the dynamic library has a method named `call_func`
        // and load up that functions symbol
        let f: Symbol<unsafe extern "C" fn(*const c_char, *const [u8; 32], *mut [u8; 32])> =
            lib.get(b"call_func\0").unwrap();

        // Call method which should modify the outputs struct
        f(c_name, c_inputs, c_outputs);
    }
}
