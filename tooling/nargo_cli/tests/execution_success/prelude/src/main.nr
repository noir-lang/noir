fn main(x: Field, y: pub Field) {
    let xs = Vec::new();
    let option = Option::none();

    print("42\n");
    println("42");
}

mod a {
    use dep::std::collections::vec::Vec;
    use dep::std::option::Option;

    fn main(x: Field, y: pub Field) {
        let xs = Vec::new();
        let option = Option::none();
    }
}
