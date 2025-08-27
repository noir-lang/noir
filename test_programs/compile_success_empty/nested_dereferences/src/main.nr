struct Foo {
    inner: Field,
}

struct Bar {
    inner: Foo,
}

fn main() {
    nested_dereferences();
    nested_struct_aliases_in_array();
}

fn nested_dereferences() {
    let mut var = Foo { inner: 0 };
    let ref = &mut &mut var;

    let mut array = [ref, ref];
    assert((**array[0]).inner == 0);

    (*(*array[0])).inner = 1;
    (**array[1]).inner = 2;

    assert(var.inner == 2);
    assert((**ref).inner == 2);
    assert((**array[0]).inner == 2);
    assert((**array[1]).inner == 2);
}

fn nested_struct_aliases_in_array() {
    let mut var = Bar { inner: Foo { inner: 0 } };

    let ref = &mut &mut &mut var;

    let mut array = [ref, ref];

    (**array[0]).inner.inner += 5;
    (**array[1]).inner.inner += 2;

    assert(var.inner.inner == 7);
    assert((***ref).inner.inner == 7);
    assert((**array[0]).inner.inner == 7);
    assert((**array[1]).inner.inner == 7);

    assert(var.inner.inner == 7);
}
