fn main() -> pub (Field, Field) {
    // Safety: testing context
    unsafe {
        let result_tuple = foo_tuple();
        let result_field = foo_field();
        (result_tuple, result_field)
    }
}

unconstrained fn foo_tuple() -> Field {
    bar_tuple(&mut 0xcafebabe, (0xdeadbeef,))
}

unconstrained fn bar_tuple(mut a: &mut Field, mut b: (Field,)) -> Field {
    let mut i = 0;
    loop {
        if i == 2 {
            break
        }
        i = i + 1;

        // Reset b each iteration
        b = (0xdeadbeef,);

        // Store *a into b.0
        b.0 = *a;

        // Make a point to b.0 (creates loop-carried alias)
        a = &mut b.0;
    }
    b.0
}

unconstrained fn foo_field() -> Field {
    bar_field(&mut 0xcafebabe, 0xdeadbeef)
}

unconstrained fn bar_field(mut a: &mut Field, mut b: Field) -> Field {
    let mut i = 0;
    loop {
        if i == 2 {
            break
        }

        i = i + 1;

        // Reset b each iteration
        b = 0xdeadbeef;

        // Store *a into b (equivalent to: b.0 = *a)
        b = *a;

        // Make a point to b (equivalent to: a = &mut b.0)
        a = &mut b;
    }

    b
}
