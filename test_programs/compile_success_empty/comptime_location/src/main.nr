use std::hash::{Hash, Hasher};

#[check_function_location]
fn target_function() {}

#[check_trait_location]
trait TargetTrait {
    fn touch();
}

impl TargetTrait for Field {
    fn touch() {}
}

#[check_type_location]
struct TargetType {
    x: Field,
}

mod inner {
    pub fn other_function() {}

    pub struct OtherType {
        y: Field,
    }
}

comptime fn check_function_location(f: FunctionDefinition) {
    let l1 = f.location();
    let l2 = f.location();
    assert_eq(l1, l2);
}

comptime fn check_trait_location(t: TraitDefinition) {
    let l1 = t.location();
    let l2 = t.location();
    assert_eq(l1, l2);
}

comptime fn check_type_location(t: TypeDefinition) {
    let l1 = t.location();
    let l2 = t.location();
    assert_eq(l1, l2);
}

fn main() {
    Field::touch();
    comptime {
        let func_a = quote { target_function }.as_expr().unwrap().resolve(Option::none());
        let func_b = quote { inner::other_function }.as_expr().unwrap().resolve(Option::none());

        let func_a_def = func_a.as_function_definition().unwrap();
        let func_b_def = func_b.as_function_definition().unwrap();

        // Calling location() repeatedly returns equal values for the same definition.
        assert_eq(func_a_def.location(), func_a_def.location());

        // Distinct definitions have distinct locations.
        let loc_a = func_a_def.location();
        let loc_b = func_b_def.location();
        assert(loc_a != loc_b);

        // TypedExpr::location() works.
        assert_eq(func_a.location(), func_a.location());

        // Module::location() works.
        assert(func_b_def.module().location() != func_b_def.location());

        // TypeDefinition::location() works.
        let type_def_a = quote { TargetType }.as_type().as_data_type().unwrap().0;
        let type_def_b = quote { inner::OtherType }.as_type().as_data_type().unwrap().0;
        assert_eq(type_def_a.location(), type_def_a.location());
        assert(type_def_a.location() != type_def_b.location());

        // Hash trait is implemented; equal locations hash to the same value.
        let mut h1 = TestHasher { result: 0 };
        let mut h2 = TestHasher { result: 0 };
        loc_a.hash(&mut h1);
        func_a_def.location().hash(&mut h2);
        assert_eq(h1.finish(), h2.finish());

        // Different locations hash to different values.
        let mut h3 = TestHasher { result: 0 };
        loc_b.hash(&mut h3);
        assert(h1.finish() != h3.finish());
    }
}

struct TestHasher {
    result: Field,
}

impl Hasher for TestHasher {
    fn finish(self) -> Field {
        self.result
    }

    fn write(&mut self, input: Field) {
        self.result += input;
    }
}
