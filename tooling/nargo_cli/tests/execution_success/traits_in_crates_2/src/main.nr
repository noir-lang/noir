fn main(x : Field, y : pub Field) {
    let mut V = dep::crate2::MyStruct { Q: x };
    V.Add10();
    assert(V.Q == y);
}
