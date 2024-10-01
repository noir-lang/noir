mod foo {
    pub fn another_function() -> Field {
        3 + 4
    }
}

use foo::another_function;
use foo::another_function as aliased_function;

mod bar;

fn some_function() -> Field {
    1 + 2
}

fn main() {
    let _ = another_function();
    bar::baz();
    bar::inline::qux();
}

struct Foo {

}

trait Trait {

}

use dependency::something;

#[attr]
pub fn foo() {}

comptime fn attr(_: FunctionDefinition) -> Quoted {
    quote { pub fn hello() {} }
}