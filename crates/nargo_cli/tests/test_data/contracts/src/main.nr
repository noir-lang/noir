fn main(x : Field, y : pub Field) {
    assert(x * 2 == y * 3);
}

contract Foo {
    fn double(x: Field) -> pub Field { x * 2 }
    fn triple(x: Field) -> pub Field { x * 3 }
}
