use std::os::raw::c_char;

// NOTE: to generate C headers from this Rust file use "cbindgen --lang C -o c_header_file.h ."

pub type NoirValue = [u8; 32];

extern "C" {
    // Makes a call to a C function which will populate the output vector of ExternFuncCall
    fn call_func(name: *const c_char, inputs: *const NoirValue, outputs: *mut NoirValue);
}
