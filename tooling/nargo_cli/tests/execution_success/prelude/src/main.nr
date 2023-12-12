fn main(x: Field, y: pub Field) {
    let xs = Vec::new();
    let option = Option::none();

    print("42\n");
    println("42");
}

mod a {
    // We don't want to give an error due to re-importing elements that are already in the prelude.
    use dep::std::collections::vec::Vec;
    use dep::std::option::Option;
    use dep::{print, println};

    fn main() {
        let xs = Vec::new();
        let option = Option::none();

        print("42\n");
        println("42");
    }
}

mod b {
    fn main() {
        let xs = Vec::new();
        let option = Option::none();

        print("42\n");
        println("42");
    }
}
