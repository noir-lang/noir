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
