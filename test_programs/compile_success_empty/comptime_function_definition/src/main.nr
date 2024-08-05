struct Foo { x: Field }

#[function_attr]
fn foo(w: i32, y: Field, Foo { x }: Foo, mut a: bool) {}

comptime fn function_attr(f: FunctionDefinition) {
    let parameters = f.parameters();
    assert_eq(parameters.len(), 4);
}

fn main() {}
