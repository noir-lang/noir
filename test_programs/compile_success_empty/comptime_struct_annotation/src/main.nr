
#[print_type_name]
struct Foo {
    bar: Field,
}

comptime fn print_type_name(typ: TypeDefinition) {
    println("hello from print_type_name at compile-time");
    println(typ);
}

fn main(){}
