fn main(x: Field) {
    x
}

contract foo {
    fn bar(x: Field) -> bool {
        main(x + y)
    }
}
