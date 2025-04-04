fn main() {
    let mut x = 100;
    let mut xref = &mut x;
    increment(xref);
    assert(*xref == 101);

    let mut x_three_refs = &mut &mut &mut x;
    increment_deeper_ref(x_three_refs);
    assert(***x_three_refs == 102);

    regression_2445();
    regression_2445_deeper_ref();
    struct_alias_in_array();
    single_alias_inside_loop();
    assert(5 == struct_field_refs_across_blocks(MyStruct { a: 5, b: 10 })[0]);
}

fn increment(mut r: &mut Field) {
    *r = *r + 1;
}
fn increment_deeper_ref(mut r: &mut &mut &mut Field) {
    ***r = ***r + 1;
}
// If aliasing within arrays and constant folding within the mem2reg pass aren't
// handled, we'll fail to optimize out all the references in this function.
fn regression_2445() {
    let mut var = 0;
    let ref = &mut &mut var;

    let mut array = [ref, ref];

    assert(**array[0] == 0);

    **array[0] = 1;
    **array[1] = 2;

    assert(var == 2);
    assert(**ref == 2);
    assert(**array[0] == 2);
    assert(**array[1] == 2);
}

fn regression_2445_deeper_ref() {
    let mut var = 0;
    let ref = &mut &mut &mut var;

    let mut array = [ref, ref];

    ***array[0] = 1;
    ***array[1] = 2;

    assert(var == 2);
    assert(***ref == 2);
    assert(***array[0] == 2);
    assert(***array[1] == 2);
}

struct Foo {
    inner: u32,
}

fn struct_alias_in_array() {
    let mut var = Foo { inner: 0 };
    let ref = &mut &mut var;

    let mut array = [ref, ref];
    assert((**array[0]).inner == 0);

    (**array[0]).inner = 1;
    (**array[1]).inner = 2;

    assert(var.inner == 2);
    assert((**ref).inner == 2);
    assert((**array[0]).inner == 2);
    assert((**array[0]).inner == 2);
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
