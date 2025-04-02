
        global A: Field = 4294967297;

        fn foo<let A: Field>() { }

        fn main() {
            let _ = foo::<A>();
        }
    