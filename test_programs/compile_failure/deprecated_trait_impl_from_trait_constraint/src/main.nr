#[deprecated(deny, "use something else")]
trait Foo {}

// The header of this impl is a `TraitConstraint` value rather than a path to the trait.
#[generate_impl]
struct Bar {}

comptime fn generate_impl(bar: TypeDefinition) -> Quoted {
    let foo = quote { Foo }.as_trait_constraint();
    let bar = bar.as_type();
    quote { impl $foo for $bar {} }
}

fn main() {
    let _ = Bar {};
}
