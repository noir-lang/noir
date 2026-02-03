use crate1::MyTrait;

fn main(x: Field, y: pub Field) {
    let mut V = crate2::MyStruct { Q: x };
    V.Add10();
    assert(V.Q == y);
}
