struct S {
    a: u8,
}

comptime fn make_s() -> Quoted {
    quote { S { a: 1 } }
}

fn main(x: pub Field) {
    // `v` is an `S`, so a later comptime block that reflects on `[v]` sees `[S; 1]` and the
    // assertion is part of the circuit.
    comptime let v = make_s!();
    let is_struct = comptime {
        std::meta::type_of([v]).as_array().unwrap().element_type().as_data_type().is_some()
    };
    if is_struct {
        assert(x == 0);
    }
}
