
    fn main() {
        let foo = Foo { x: 10, y: 20 };
        match foo {
            Foo { x: _, y: _, z: _ } => {}
        }
    }

    struct Foo {
        x: i32,
        y: Field,
    }
    