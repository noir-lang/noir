use mylib::PublicBox;

// `Secret` is reached by reflecting over `PublicBox`'s field, but the library re-exports it
// publicly, so splicing it into a public API is allowed: hand-written code could name it too.
comptime fn emit_reexported_type_api(_f: FunctionDefinition) -> Quoted {
    let typ = quote { PublicBox }.as_type();
    let data_type = typ.as_data_type().unwrap();
    let fields = data_type.definition().fields(data_type.generics());
    let reexported_type = fields[0].typ();

    quote {
        pub fn reexported_type_api() -> Option<$reexported_type> {
            Option::none()
        }

        pub struct ReexportedPublicField {
            pub value: Option<$reexported_type>,
        }
    }
}

#[emit_reexported_type_api]
fn main() {}
