#[test_as_constraint]
pub trait Foo<T> {}

comptime fn test_as_constraint(t: TraitDefinition) {
    let constraint = t.as_trait_constraint();
    assert(constraint == constraint);
}

fn main() {}
