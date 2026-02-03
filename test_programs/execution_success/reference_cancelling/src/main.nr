fn main() {
    assert(comptime { store_through_outer_pointer() });
    assert(store_through_outer_pointer());

    assert(comptime { store_through_middle_pointer() });
    assert(store_through_middle_pointer());
}

fn store_through_outer_pointer() -> bool {
    let a = &mut ((false,),);
    let b = &mut a.0.0;
    *a = ((true,),);
    println(*b);
    *b
}

fn store_through_middle_pointer() -> bool {
    let a = &mut ((false,),);
    let b = &mut a.0.0;
    a.0 = (true,);
    println(*b);
    *b
}
