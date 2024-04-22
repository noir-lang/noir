fn main(x: field, y: pub field) {
    let mut V = dep::crate2::MyStruct { Q: x };
    V.Add10();
    assert(V.Q == y);
}
