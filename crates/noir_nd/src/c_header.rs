use std::os::raw::c_char;

// NOTE: to generate C headers from this Rust file use "cbindgen --lang C -o c_header_file.h ."
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct C_ExternFuncCall {
    pub name: *const c_char,
    // Vec<[u8;32]>
    pub inputs: *const [u8; 32],
    // Vec<[u8;32]>
    pub outputs: *mut [u8; 32],
}

extern "C" {
    // Makes a call to a C function which will populate the output vector of ExternFuncCall
    fn call_func(x: C_ExternFuncCall);
}
