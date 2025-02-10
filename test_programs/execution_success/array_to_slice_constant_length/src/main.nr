// Regression test for https://github.com/noir-lang/noir/issues/4722
unconstrained fn return_array(val: Field) -> [Field; 1] {
    [val; 1]
}

fn main(val: Field) {
    /// Safety: testing context
    unsafe {
        let array = return_array(val);
        assert_constant(array.as_slice().len());
    }
}
