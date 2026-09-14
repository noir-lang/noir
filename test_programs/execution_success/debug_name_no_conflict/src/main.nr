// Regression test: user-defined functions with names matching internal debug
// function names (e.g. __debug_var_assign) should not be patched by the
// debug instrumentation pass during monomorphization.

fn __debug_var_assign(var_id: u32, value: Field) -> Field {
    value + var_id as Field
}

fn __debug_var_drop(var_id: u32) -> u32 {
    var_id + 1
}

fn main() {
    let result = __debug_var_assign(5, 10);
    assert(result == 15);

    let dropped = __debug_var_drop(42);
    assert(dropped == 43);
}
