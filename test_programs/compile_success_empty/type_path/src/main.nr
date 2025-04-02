fn main() {
    comptime {
        let foo = quote { Foo }.as_type();
        quote {
            $foo::static()
        }
    }

    // Make sure this call works fine: in the past we used to not distinguish
    // whether a TypePath had generics or not, always resolved them, filling them
    // up with Type::Error, and eventually leading to an ICE.
    let _ = Field::from_be_bytes([1]);

    // Make sure `<...>::name` compiles
    let _: () = <()>::method();
    let _: [i32; 3] = <[i32; 3]>::method();
}

pub struct Foo {}

impl Foo {
    fn static() {}
}

trait Trait {
    fn method() -> Self;
}

impl Trait for () {
    fn method() -> () {
        ()
    }
}

impl Trait for [i32; 3] {
    fn method() -> [i32; 3] {
        [1, 2, 3]
    }
}
