use std::hash::{Hash, Hasher};

trait TwoOrdered<A, B> {}

trait TwoNamed {
    type A;
    type B;
}

impl TwoOrdered<Field, u8> for Field {}
impl TwoOrdered<u8, Field> for Field {}

impl TwoNamed for Field {
    type A = Field;
    type B = u8;
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

comptime fn hash_constraint(t: TraitConstraint) -> Field {
    let mut h = TestHasher { result: 0 };
    t.hash(&mut h);
    h.finish()
}

fn main() {
    comptime {
        let t = quote { Field }.as_type();

        // The constraints below are semantically equivalent,
        // so they should be considered equal and have the same hash.
        {
            let c1 = quote { TwoNamed<A = Field, B = u8> }.as_trait_constraint();
            let c2 = quote { TwoNamed<B = u8, A = Field> }.as_trait_constraint();

            // They should be printed in the original order.
            println(c1);
            println(c2);

            assert(t.implements(c1));
            assert(t.implements(c2));

            assert(c1 == c2);

            let hash_c1 = hash_constraint(c1);
            let hash_c2 = hash_constraint(c2);
            assert(hash_c1 == hash_c2);
        }

        // The constraints below are not equivalents.
        {
            let c1 = quote { TwoOrdered<Field, u8> }.as_trait_constraint();
            let c2 = quote { TwoOrdered<u8, Field> }.as_trait_constraint();

            // They should be printed in the original order.
            println(c1);
            println(c2);

            assert(t.implements(c1));
            assert(t.implements(c2));

            assert(c1 != c2);

            let hash_c1 = hash_constraint(c1);
            let hash_c2 = hash_constraint(c2);
            assert(hash_c1 != hash_c2);
        }
    }
}
