use std::meta::type_of;

struct Foo { x: Field }

#[function_attr]
fn foo(w: i32, y: Field, Foo { x }: Foo, mut a: bool) -> i32 {
    1
}

comptime fn function_attr(f: FunctionDefinition) {
    // Check FunctionDefinition::parameters
    let parameters = f.parameters();
    assert_eq(parameters.len(), 4);

    assert_eq(parameters[0].0, quote { w });
    assert_eq(parameters[1].0, quote { y });
    assert_eq(parameters[2].0, quote { Foo { x } });
    assert_eq(parameters[3].0, quote { mut a });

    let an_i32: i32 = 0;

    assert_eq(parameters[0].1, type_of(an_i32));
    assert_eq(parameters[1].1, type_of(0));
    assert_eq(parameters[2].1, type_of(Foo { x: 0 }));
    assert_eq(parameters[3].1, type_of(true));

    // Check FunctionDefinition::return_type
    assert_eq(f.return_type(), type_of(an_i32));
}

fn main() {}
