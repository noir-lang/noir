
    struct Foo {
        bar: Field,
        array: [Field; 2],
    }

    struct Default {
        x: Field,
        z: Field,
    }

    impl Default for Foo {
        fn default(x: Field, y: Field) -> Self {
            Self { bar: x, array: [x,y] }
        }
    }

    fn main() {
        let _ = Default { x: 1, z: 1 }; // silence Default never constructed warning
    }
    