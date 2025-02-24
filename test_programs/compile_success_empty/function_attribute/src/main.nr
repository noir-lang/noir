#[function_attr]
pub fn foo() {}

struct Foo {}

comptime fn function_attr(_f: FunctionDefinition) -> Quoted {
    quote {
        impl Default for Foo {
            fn default() -> Foo {
                Foo {}
            }
        }
    }
}

fn main() {
    let _ = Foo::default();
}
