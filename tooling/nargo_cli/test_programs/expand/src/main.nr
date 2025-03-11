use std::as_witness as aliased_as_witness;

trait SomeTrait {
    fn some_method();
}

/// Docs on top of Foo
pub struct Foo<T> {
    /// Docs on top of value
    value: T,
    int: i32,
}

impl<T> Foo<T> {
    fn method(self, x: i32) -> i32 {
        self.int() + x
    }

    /// int docs
    pub fn int(self) -> i32 {
        self.int
    }
}

impl<T> SomeTrait for Foo<T> {
    fn some_method() {
        let name = "some_method";
        panic(f"Implement \\\n\t {name} {{ }}")
    }
}

mod module {
    pub struct Bar {}
}

fn main() {
    let _x = aliased_as_witness(1);
    let _y = comptime { 1 + 2 };
    let foo = Foo { value: 1, int: 2 };
    let _ = foo.method(10);
}

#[abi(functions)]
fn bar() -> module::Bar {
    let bar = module::Bar {};
    bar
}

#![generate_baz]

comptime fn generate_baz(_: Module) -> Quoted {
    quote {
        pub fn baz() {}
    }
}
