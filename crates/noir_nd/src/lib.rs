// This is the non determinism (nd) module.

pub mod c_header;
mod wasm_loader;
pub use noir_field::FieldElement;

use libloading::{Library, Symbol};
use std::{os::raw::c_char, path::Path};

use crate::wasm_loader::CompiledModule;

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
    let path = path.as_ref();
    let wasm_bytes = std::fs::read(path).expect("file does not exist");

    // We store the cache in a directory one level deeper than where we found the wasm file
    let mut path_to_cache = path.to_path_buf();
    path_to_cache.pop(); // This pops off the file from the path
    path_to_cache.push("cache");

    // TODO: can we do this once outside of this module
    let mut compiled_mod = CompiledModule::new(&wasm_bytes, &path_to_cache);

    let name = name.as_bytes();
    // Send the inputs as a single flat vector to WASM and divide by 32 to get the number of items
    let inputs: Vec<_> = inputs
        .into_iter()
        .map(|input| input.to_vec())
        .flatten()
        .collect();
    let outputs_bytes_len: usize = outputs.iter().map(|output| output.len()).sum();

    let name_ptr = compiled_mod.allocate(name);
    let inputs_ptr = compiled_mod.allocate(&inputs);
    let outputs_ptr = compiled_mod.allocate(&vec![0u8; outputs_bytes_len]);

    use wasmer::Value;

    compiled_mod.call_multiple(
        "call_func",
        vec![
            &name_ptr.addr,
            &Value::I32(name_ptr.size as i32),
            &inputs_ptr.addr,
            &Value::I32(inputs_ptr.size as i32),
            &outputs_ptr.addr,
            &Value::I32(outputs_ptr.size as i32),
        ],
    );

    // The memory referenced by outputs ptr should now be populated
    let populated_output = compiled_mod.read_memory(&outputs_ptr, outputs_bytes_len);
    for (i, chunk) in populated_output.chunks(32).enumerate() {
        outputs[i] = chunk.try_into().unwrap();
    }

    compiled_mod.free(name_ptr);
    compiled_mod.free(inputs_ptr);
    compiled_mod.free(outputs_ptr);
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
