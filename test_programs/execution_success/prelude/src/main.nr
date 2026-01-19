fn main() {
    let _xs: Vec<Field> = Vec::new();
    let _option: Option<Field> = Option::none();

    print("42\n");
    println("42");
}

mod a {
    // We don't want to give an error due to re-importing elements that are already in the prelude.
    use std::collections::vec::Vec;
    use std::option::Option;

    fn main() {
        let _xs: Vec<Field> = Vec::new();
        let _option: Option<Field> = Option::none();

        print("42\n");
        println("42");
    }
}

mod b {
    fn main() {
        let _xs: Vec<Field> = Vec::new();
        let _option: Option<Field> = Option::none();

        print("42\n");
        println("42");
    }
}
