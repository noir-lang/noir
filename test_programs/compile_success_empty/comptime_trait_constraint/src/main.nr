use std::hash::{Hash, Hasher};

trait TraitWithGenerics<A, B> {
    fn foo(self) -> (A, B);
}

fn main() {
    comptime
    {
        let constraint1 = quote { Default }.as_trait_constraint();
        let constraint2 = quote { TraitWithGenerics<Field, u32> }.as_trait_constraint();

        assert(constraint1 != constraint2);

        let mut hasher = TestHasher { result: 0 };
        constraint1.hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher = TestHasher { result: 0 };
        constraint2.hash(&mut hasher);
        let hash2 = hasher.finish();

        assert(hash1 != hash2);
    }
}

comptime struct TestHasher {
    result: Field,
}

comptime impl Hasher for TestHasher {
    comptime fn finish(self) -> Field {
        self.result
    }
    
    comptime fn write(&mut self, input: Field) {
        self.result += input;
    }
}
