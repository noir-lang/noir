fn main(x : Field, y : pub Field) {
    constrain Foo::double(x) == Foo::triple(y);
}

contract Foo {
    fn double(x: Field) -> Field { x * 2 }
    fn triple(x: Field) -> Field { x * 3 }
}
