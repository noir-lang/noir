
    fn main() {
        let foo = Foo { x: 10, y: 20 };
        match foo {
            Foo { x: _, x: _, y: _ } => {}
        }
    }

    struct Foo {
        x: i32,
        y: Field,
    }
    