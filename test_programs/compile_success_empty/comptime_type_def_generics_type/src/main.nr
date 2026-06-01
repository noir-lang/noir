#[test_attr]
pub struct Pair<T> {
    x: T,
    y: T,
}

comptime fn test_attr(pair_def: TypeDefinition) {
    let generics = pair_def.generics();
    let generics_type = std::meta::type_of(generics);
    let _ = generics_type.as_vector().unwrap();
}

fn main() {}
