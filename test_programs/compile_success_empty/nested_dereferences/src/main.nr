struct Foo {
    inner: Field,
}

fn main() {
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
