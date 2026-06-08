use mylib::{anchor, PublicBox};

// A macro must not be able to splice a dependency-private type into a public API. The
// `hidden::Secret` type is reachable here only by reflecting over `PublicBox`'s field; it
// cannot be named by hand, so it must not leak through a `pub fn` return or public struct field.
comptime fn emit_private_type_api(_f: FunctionDefinition) -> Quoted {
    let typ = quote { PublicBox }.as_type();
    let (def, generics) = typ.as_data_type().unwrap();
    let fields = def.fields(generics);
    let private_type = fields[0].1;

    quote {
        pub fn leaked_private_type_api() -> Option<$private_type> {
            Option::none()
        }

        pub struct LeakedPublicField {
            pub value: Option<$private_type>,
        }
    }
}

#[emit_private_type_api]
fn main() -> pub Field {
    anchor()
}
