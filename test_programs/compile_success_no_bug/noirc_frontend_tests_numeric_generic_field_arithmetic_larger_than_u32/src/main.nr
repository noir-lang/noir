
        struct Foo<let F: Field> {}

        fn size<let F: Field>(_x: Foo<F>) -> Field {
            F
        }

        // 2^32 - 1
        global A: Field = 4294967295;

        fn foo<let A: Field>() -> Foo<A + A> {
            Foo {}
        }

        fn main() {
            let _ = size(foo::<A>());
        }
    