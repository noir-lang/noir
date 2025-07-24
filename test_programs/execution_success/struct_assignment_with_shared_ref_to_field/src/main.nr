fn main() {
    let (a, b) = comptime { (plain_store(), store_through_pointer()) };
    let (c, d) = (plain_store(), store_through_pointer());
    assert(a);
    assert(b);
    assert(c);
    assert(d);
}

fn plain_store() -> bool {
    let mut a = (false,);
    let b = &mut a.0;
    a = (true,);
    *b
}

fn store_through_pointer() -> bool {
    let a = &mut (false,);
    let b = &mut a.0;
    *a = (true,);
    *b
}
