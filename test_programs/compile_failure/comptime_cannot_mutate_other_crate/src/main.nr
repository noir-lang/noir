// We should not be able to mutate this external type in any way!
use dependency::Foo;

comptime fn get_lib_foo() -> TypeDefinition {
    quote { Foo }.as_type().as_data_type().unwrap().0
}

// We should not be able to mutate the fields of an external type
#[mutate_fields]
comptime fn mutate_fields(_: FunctionDefinition) {
    let foo = get_lib_foo();
    let int = quote[i32].as_type();
    foo.set_fields([(quote[bad], int, quote[])]);
}

// We should not be able to mutate the attributes of an external type
#[mutate_attrs]
comptime fn mutate_attrs(_: FunctionDefinition) {
    let foo = get_lib_foo();
    foo.add_attribute("bad_type_attr");
}

// We should not be able to mutate the generics of an external type
#[mutate_generics]
comptime fn mutate_generics(_: FunctionDefinition) {
    let foo = get_lib_foo();
    let _ = foo.add_generic("T");
}

fn main() {}
