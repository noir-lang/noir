use wasm_bindgen::prelude::*;

// simple_add
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

use core::alloc::Layout;
// malloc_alloc
#[wasm_bindgen]
pub fn malloc(size: u32, alignment: u32) -> *mut u8 {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size as usize, alignment as usize);
        std::alloc::alloc(layout)
    }
}

#[wasm_bindgen]
pub fn free(ptr: *mut u8, size: u32, alignment: u32) {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size as usize, alignment as usize);
        std::alloc::dealloc(ptr, layout);
    }
}

#[wasm_bindgen]
pub fn call_func(
    name_ptr: *const u8,
    name_len: usize,
    inputs_ptr: *const u8,
    inputs_len: usize,
    outputs_ptr: *mut u8,
    outputs_len: usize,
) {
    let name = unsafe { std::slice::from_raw_parts(name_ptr, name_len) };
    let inputs = unsafe { std::slice::from_raw_parts(inputs_ptr, inputs_len) };
    let outputs = unsafe { std::slice::from_raw_parts_mut(outputs_ptr, outputs_len) };

    if name == String::from("modify_output").as_bytes() {
        outputs[0..64].copy_from_slice(&vec![3u8; 32]);
        outputs[64..128].copy_from_slice(&vec![9u8; 32]);
    }
}
