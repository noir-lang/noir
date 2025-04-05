
    struct Foo {
        x: Field
    }

    fn main() {
        let foo = Foo { x: 0 };
        mutate(&mut foo.x);
    }

    fn mutate(foo: &mut Field) {
        *foo = 1;
    }
    