// A dynamic-index `array_set` on an array whose element type has zero flattened width
// (`[Field; 0]`, likewise `str<0>`) builds a zero-length `AcirValue::DynamicArray`. That block
// is recorded as initialized but emits no `MemoryInit` (see the "Zero-Length Arrays" rule in
// `acir/arrays.rs`). Passing the array into an unconstrained call must therefore not lower it to
// a `BrilligInputs::MemoryArray` referencing the orphan block: doing so panicked
// `MergeExpressionsOptimizer` with "Unknown block id" (and failed raw ACVM with
// `MissingMemoryBlock`). The array must stay live through the Brillig call as a `DynamicArray`,
// so it is returned back out.
unconstrained fn id(x: [[Field; 0]; 1]) -> [[Field; 0]; 1] {
    x
}

fn main(idx: u32) -> pub [[Field; 0]; 1] {
    let mut arr: [[Field; 0]; 1] = [[]];
    arr[idx] = [];
    // Safety: `id` returns its argument unchanged; there are no values to constrain.
    unsafe { id(arr) }
}
