use std::hash::{Hash, Hasher};

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

fn main() {
    comptime {
        let a = quote { true };

        let expr: Expr = quote { true }.as_expr().unwrap();
        let b = quote { $expr };

        assert(a == b);

        let mut h1 = TestHasher { result: 0 };
        a.hash(&mut h1);
        let hash_a = h1.finish();

        let mut h2 = TestHasher { result: 0 };
        b.hash(&mut h2);
        let hash_b = h2.finish();

        assert_eq(hash_a, hash_b);
    }
}
