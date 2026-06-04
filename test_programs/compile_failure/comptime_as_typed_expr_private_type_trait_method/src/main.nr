// A private foreign type's trait-impl method must not become callable through
// `FunctionDefinition::as_typed_expr`, even when the private type is recovered via
// metaprogramming from a public wrapper's field type.
#[synthesize_private_call]
struct Trigger {}

comptime fn synthesize_private_call(_s: TypeDefinition) -> Quoted {
    let wrapper_type = quote { foo::PublicWrapper }.as_type();
    let (wrapper_def, wrapper_generics) = wrapper_type.as_data_type().unwrap();
    let fields = wrapper_def.fields(wrapper_generics);
    let hidden_type = fields[0].1;

    let leak_trait = quote { foo::Leak }.as_trait_constraint();
    let leak_impl = hidden_type.get_trait_impl(leak_trait).unwrap();
    let raw_compute = leak_impl.methods()[0].as_typed_expr();

    quote {
        pub fn call_private_hidden_trait_impl(x: Field) -> Field {
            $raw_compute(x)
        }
    }
}

fn main() {
    assert(call_private_hidden_trait_impl(999) == 999000);
}
