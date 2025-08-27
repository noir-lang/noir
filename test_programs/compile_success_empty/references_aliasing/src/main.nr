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
    deeper_ref_in_loop();
    struct_alias_in_loop();
    struct_multi_field_alias();
    struct_multi_field_alias_in_loop();
    struct_multi_field_different_aliases();
    struct_multi_field_different_aliases_in_loop();
    nested_struct_aliases();
    nested_struct_multi_field_different_aliases_in_loop();
    nested_struct_aliases_in_array();
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
    assert((**array[1]).inner == 2);
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

fn deeper_ref_in_loop() {
    let mut var = 0;
    let ref = &mut &mut &mut var;

    for _ in 0..1 {
        ***ref = 2;
    }

    assert(var == 2);
    assert(***ref == 2);
}

fn struct_alias_in_loop() {
    let mut var = Foo { inner: 0 };
    let ref = &mut &mut var;

    for _ in 0..1 {
        (**ref).inner = 1;
        (**ref).inner = 2;
    }

    assert(var.inner == 2);
    assert((**ref).inner == 2);
}

struct MyStruct {
    a: Field,
    b: u32,
}

fn struct_multi_field_alias() {
    let mut var = MyStruct { a: 0, b: 10 };
    let ref = &mut &mut var;

    (**ref).a = 1;
    (**ref).b = 20;

    assert(var.a == 1);
    assert(var.b == 20);
    assert((**ref).a == 1);
    assert((**ref).b == 20);
}

fn struct_multi_field_alias_in_loop() {
    let mut var = MyStruct { a: 0, b: 0 };
    let ref = &mut &mut var;

    for _ in 0..1 {
        (**ref).a = (**ref).a + 1;
        (**ref).b = (**ref).b + 2;
    }

    assert(var.a == 1);
    assert(var.b == 2);
    assert((**ref).a == 1);
    assert((**ref).b == 2);
}

fn struct_multi_field_different_aliases_in_loop() {
    let mut var = MyStruct { a: 0, b: 0 };
    let ref_a = &mut var;
    let ref_b = &mut var;

    for _ in 0..3 {
        (*ref_a).a += 2;
        (*ref_b).a += 3;
        (*ref_a).b += 1;
        (*ref_b).b += 2;
    }

    // Updates from the loop:
    // a = 0+2+3 = 5, b = 0+1+2 = 3
    // a = 5+2+3 = 10, b = 3+1+2 = 6
    // a = 10+2+3 = 15, b = 6+1+2 = 9
    assert(var.a == 15);
    assert(var.b == 9);

    assert((*ref_a).a == 15);
    assert((*ref_a).b == 9);
    assert((*ref_b).a == 15);
    assert((*ref_b).b == 9);
}

fn struct_multi_field_different_aliases() {
    let mut var = MyStruct { a: 0, b: 0 };
    let ref_a = &mut var;
    let ref_b = &mut var;

    (*ref_a).a = 10;
    (*ref_a).b = 20;

    // Should read the 10 from ref_a and write 15
    (*ref_b).a = (*ref_b).a + 5;
    // Should read the 20 from ref_a and write 25
    (*ref_b).b = (*ref_b).b + 5;

    assert(var.a == 15);
    assert(var.b == 25);

    assert((*ref_a).a == 15);
    assert((*ref_a).b == 25);
    assert((*ref_b).a == 15);
    assert((*ref_b).b == 25);
}

struct Outer {
    inner: MyStruct,
}

fn nested_struct_aliases() {
    let mut var = Outer { inner: MyStruct { a: 0, b: 0 } };

    let ref_outer = &mut var;
    let ref_inner = &mut ref_outer.inner;

    (*ref_outer).inner.a = 5;
    (*ref_inner).b = 10;

    assert(var.inner.a == 5);
    assert(var.inner.b == 10);
    assert((*ref_outer).inner.a == 5);
    assert((*ref_outer).inner.b == 10);
    assert((*ref_inner).a == 5);
    assert((*ref_inner).b == 10);
}

fn nested_struct_multi_field_different_aliases_in_loop() {
    let mut var = Outer { inner: MyStruct { a: 0, b: 0 } };
    let ref_outer = &mut var;
    let ref_inner = &mut var.inner;

    for _ in 0..3 {
        (*ref_outer).inner.a += 2;
        (*ref_inner).a += 3;
        (*ref_outer).inner.b += 1;
        (*ref_inner).b += 2;
    }

    assert(var.inner.a == 15);
    assert(var.inner.b == 9);
    assert((*ref_outer).inner.a == 15);
    assert((*ref_outer).inner.b == 9);
    assert((*ref_inner).a == 15);
    assert((*ref_inner).b == 9);
}

fn nested_struct_aliases_in_array() {
    let mut var = Outer { inner: MyStruct { a: 0, b: 0 } };

    let ref = &mut &mut &mut var;

    let mut array = [ref, ref];

    // Update the same inner field `a` through both deep aliases
    (**array[0]).inner.a += 5;
    (**array[1]).inner.a += 2;

    assert(var.inner.a == 7);
    assert((***ref).inner.a == 7);
    assert((**array[0]).inner.a == 7);
    assert((**array[1]).inner.a == 7);

    assert(var.inner.b == 0);
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
