impl dep::crate1::MyTrait for dep::crate2::MyStruct {
}

fn main(x: field, y: pub field) {
    assert(x != y);
}
