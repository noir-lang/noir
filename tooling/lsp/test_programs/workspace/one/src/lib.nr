pub fn function_one<A, B>() {}

mod subone {
    struct SubOneStruct {
        some_field: i32,
        some_other_field: Field,
    }

    impl SubOneStruct {
        fn foo(self, x: i32, y: i32) -> Field {
            0
        }
    }

    trait SomeTrait {
    }

    global some_global = 2;

    type SomeAlias = i32;

    struct GenericStruct<A, B> {

    }
}
