impl dep::crate1::MyTrait for dep::crate2::MyStruct {
}

fn main(x : Field, y : pub Field) {
    assert(x != y);
}
