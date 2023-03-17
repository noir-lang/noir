fn main(x : Field, y : pub Field) {
    constrain x * 2 == y * 3;
}

contract Foo {
    fn double(x: Field) -> Field { x * 2 }
    fn triple(x: Field) -> Field { x * 3 }
}
