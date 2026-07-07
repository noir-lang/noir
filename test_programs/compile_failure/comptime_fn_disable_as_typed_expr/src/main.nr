// A `disable`d function's body is intentionally never type-checked, so the type-incoherent body of
// `tag` below (returning `bool` where `u32` is declared) does not raise a type error. Capturing the
// method via `as_typed_expr` before disabling it then splicing a call bypasses the `deprecated(deny)`
// lint, so the call reaches monomorphization. That must produce a clean "called disabled function"
// error rather than panicking on the method's missing body.
trait Tag {
    fn tag() -> u32;
}

pub struct Victim {}

impl Tag for Victim {
    fn tag() -> u32 {
        true
    }
}

comptime fn evil(_self: FunctionDefinition) -> Quoted {
    let victim_typ = quote { Victim }.as_type();
    let tag_constraint = quote { Tag }.as_trait_constraint();
    let foreign_impl = victim_typ.get_trait_impl(tag_constraint).unwrap();
    let m = foreign_impl.methods()[0];
    let typed = m.as_typed_expr();
    m.disable(f"tag is disabled");

    quote {
        pub fn use_disabled() -> u32 {
            $typed()
        }
    }
}

#[evil]
fn main() {
    let _ = use_disabled();
}
