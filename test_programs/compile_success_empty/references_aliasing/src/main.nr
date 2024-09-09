fn main() {
    let mut x = 100;
    let mut xref = &mut x;
    increment(xref);
    assert(*xref == 101);

    regression_2445();
    single_alias_inside_loop();
    assert(5 == struct_field_refs_across_blocks(MyStruct { a: 5, b: 10 })[0]);
}

fn increment(mut r: &mut Field) {
    *r = *r + 1;
}
// If aliasing within arrays and constant folding within the mem2reg pass aren't
// handled, we'll fail to optimize out all the references in this function.
fn regression_2445() {
    let mut var = 0;
    let ref = &mut &mut var;

    let mut array = [ref, ref];

    **array[0] = 1;
    **array[1] = 2;

    assert(var == 2);
    assert(**ref == 2);
    assert(**array[0] == 2);
    assert(**array[1] == 2);
}

fn single_alias_inside_loop() {
    let mut var = 0;
    let ref = &mut &mut var;

    for _ in 0..1 {
        **ref = 2;
    }

    assert(var == 2);
    assert(**ref == 2);
}

struct MyStruct {
    a: Field,
    b: u32,
}

fn struct_field_refs_across_blocks(mut my_struct: MyStruct) -> [Field; 1] {
    [compute_dummy_hash(my_struct.a, my_struct.b, 20)]
}

fn compute_dummy_hash(input: Field, rhs: u32, in_len: u32) -> Field {
    let mut res = 0;
    if rhs < in_len {
        res += input;
    }
    res
}
