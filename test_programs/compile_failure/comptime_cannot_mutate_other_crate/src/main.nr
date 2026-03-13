// We should not be able to mutate this external type in any way!
use dependency::{ Foo, foo };

comptime fn get_lib_foos() -> (TypeDefinition, FunctionDefinition) {
    let foo_t = quote { Foo }.as_type().as_data_type().unwrap().0;
    let foo_f = quote { foo }.as_expr().unwrap().resolve(Option::none()).as_function_definition().unwrap();
    (foo_t, foo_f)
}

// We should not be able to mutate the attributes of an external type
#[add_attr_to_external_type]
comptime fn add_attr_to_external_type(_: FunctionDefinition) {
    let (foo_t, _) = get_lib_foos();

    foo_t.add_attribute("bad_type_attr");
}

// We should not be able to mutate the attributes of an external type
#[add_attr_to_external_function]
comptime fn add_attr_to_external_function(_: FunctionDefinition) {
    let (_, foo_f) = get_lib_foos();
    foo_f.add_attribute("bad_function_attr");
}

fn main() {}
