fn main(x : Field, y : pub Field) {
    assert(x * 2 == y * 3);
}

contract Foo {
    fn double(x: Field) -> pub Field { x * 2 }
    fn triple(x: Field) -> pub Field { x * 3 }
    internal fn quadruple(x: Field) -> pub Field { x * 4 }
    open internal fn skibbidy(x: Field) -> pub Field { x * 5 }
}
