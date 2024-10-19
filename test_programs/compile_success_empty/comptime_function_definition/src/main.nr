use std::meta::type_of;

pub struct Foo {
    x: Field,
    field: Field,
}

#[function_attr]
pub fn foo(
    w: i32,
    y: Field,
    Foo { x, field: some_field }: Foo,
    mut a: bool,
    (b, c): (i32, i32),
) -> i32 {
    let _ = (w, y, x, some_field, a, b, c);
    1
}

#[test]
#[deprecated]
#[check_named_attribute]
fn some_test() {}

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

    // Check FunctionDefinition::name
    assert_eq(f.name(), quote { foo });

    assert(f.has_named_attribute("function_attr"));
}

comptime fn check_named_attribute(f: FunctionDefinition) {
    assert(f.has_named_attribute("test"));
    assert(f.has_named_attribute("deprecated"));
}

#[mutate_add_one]
fn add_one() {}

comptime fn mutate_add_one(f: FunctionDefinition) {
    // fn add_one(x: Field)
    assert_eq(f.parameters().len(), 0);
    f.set_parameters(&[(quote { x }, type_of(0))]);
    assert_eq(f.parameters().len(), 1);

    // fn add_one(x: Field) -> Field
    assert_eq(f.return_type(), type_of(()));
    f.set_return_type(type_of(0));
    assert_eq(f.return_type(), type_of(0));

    // fn add_one(x: Field) -> Field { x + 1 }
    f.set_body(quote { x + 1 }.as_expr().unwrap());
}

fn main() {
    assert_eq(add_one(41), 42);
}

contract some_contract {
    // No pub on the return type is an error
    #[super::set_pub_return]
    pub fn bar() -> Field {
        1
    }
}

comptime fn set_pub_return(f: FunctionDefinition) {
    f.set_return_public(true);
}

mod foo {
    #[attr]
    pub fn some() {}

    comptime fn attr(f: FunctionDefinition) {
        assert_eq(f.module().name(), quote { foo });

        assert(!f.is_unconstrained());

        f.set_unconstrained(true);
        assert(f.is_unconstrained());

        f.set_unconstrained(false);
        assert(!f.is_unconstrained());
    }
}
