use std::meta::type_of;

struct Foo { x: Field, field: Field }

#[function_attr]
fn foo(w: i32, y: Field, Foo { x, field: some_field }: Foo, mut a: bool, (b, c): (i32, i32)) -> i32 {
    1
}

comptime fn function_attr(f: FunctionDefinition) {
    // Check FunctionDefinition::parameters
    let parameters = f.parameters();
    assert_eq(parameters.len(), 5);

    assert_eq(parameters[0].0, quote { w });
    assert_eq(parameters[1].0, quote { y });
    assert_eq(parameters[2].0, quote { Foo { x, field: some_field } });
    assert_eq(parameters[3].0, quote { mut a });
    assert_eq(parameters[4].0, quote { (b, c) });

    let an_i32: i32 = 0;

    assert_eq(parameters[0].1, type_of(an_i32));
    assert_eq(parameters[1].1, type_of(0));
    assert_eq(parameters[2].1, type_of(Foo { x: 0, field: 1 }));
    assert_eq(parameters[3].1, type_of(true));
    assert_eq(parameters[4].1, type_of((an_i32, an_i32)));

    // Check FunctionDefinition::return_type
    assert_eq(f.return_type(), type_of(an_i32));
}

fn main() {}
