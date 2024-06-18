impl crate1::MyTrait for crate2::MyStruct {
}

fn main(x: Field, y: pub Field) {
    assert(x != y);
}
