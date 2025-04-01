
    fn main() {
        let foo = Foo { x: 10, y: 20 };
        match foo {
            Foo { x: _ } => {}
        }
    }

    struct Foo {
        x: i32,
        y: Field,
    }
    