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

mod inner {
    pub fn other_function() {}
}

comptime fn check_function_location(f: FunctionDefinition) {
    let l1 = f.location();
    let l2 = f.location();
    assert(l1 == l2);
}

comptime fn check_trait_location(t: TraitDefinition) {
    let l1 = t.location();
    let l2 = t.location();
    assert(l1 == l2);
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
        assert(func_a.location() == func_a.location());

        // Module::location() works.
        assert(func_b_def.module().location() != func_b_def.location());

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
