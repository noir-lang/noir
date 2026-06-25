use midlib::PublicBox;

// This macro runs while elaborating the downstream binary, which does NOT depend on `leaflib`
// directly (only transitively, through `midlib`). `LeafPublic` is reached by reflecting over
// `PublicBox`'s field. It is maximally public, so splicing it into a public API is allowed:
// it is part of the dependency graph's public surface, not "more private" than the item.
comptime fn emit_transitive_type_api(_f: FunctionDefinition) -> Quoted {
    let typ = quote { PublicBox }.as_type();
    let (def, generics) = typ.as_data_type().unwrap();
    let fields = def.fields(generics);
    let transitive_type = fields[0].1;

    quote {
        pub fn transitive_type_api() -> Option<$transitive_type> {
            Option::none()
        }

        pub struct TransitivePublicField {
            pub value: Option<$transitive_type>,
        }
    }
}

#[emit_transitive_type_api]
fn main() {}
