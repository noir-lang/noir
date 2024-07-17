fn foo(_x: i32) {
    let _ = 1;
}

struct SomeStruct {
    field: i32,
}

impl SomeStruct {
    fn new() -> SomeStruct {
        SomeStruct { field: 0 }
    }
}

trait SomeTrait<U> {
    fn some_method(x: U);
}

impl SomeTrait<i32> for SomeStruct {
    fn some_method(_x: i32) {
    }
}

mod submodule {
    global SOME_GLOBAL = 1;
}
