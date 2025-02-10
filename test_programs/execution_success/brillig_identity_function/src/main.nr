struct myStruct {
    foo: Field,
    foo_arr: [Field; 2],
}
// Tests a very simple program.
//
// The features being tested is the identity function in Brillig
fn main(x: Field) {
    /// Safety: testing context
    unsafe {
        assert(x == identity(x));
        // TODO: add support for array comparison
        let arr = identity_array([x, x]);
        assert(x == arr[0]);
        assert(x == arr[1]);

        let s = myStruct { foo: x, foo_arr: [x, x] };
        let identity_struct = identity_struct(s);
        assert(x == identity_struct.foo);
        assert(x == identity_struct.foo_arr[0]);
        assert(x == identity_struct.foo_arr[1]);
    }
}

unconstrained fn identity(x: Field) -> Field {
    x
}

unconstrained fn identity_array(arr: [Field; 2]) -> [Field; 2] {
    arr
}

unconstrained fn identity_struct(s: myStruct) -> myStruct {
    s
}
