use std::hash::Hasher;
use std::hash::poseidon::PoseidonHasher;
use std::hash::poseidon2::Poseidon2Hasher;

fn main(x: Field, y: pub Field) {
    let mut hasher = PoseidonHasher::default();
    hasher.write(x);
    hasher.write(y);
    let poseidon_expected_hash = hasher.finish();
    // Check that we get the same result when using the hasher in a
    // method that purely uses trait methods without a supplied implementation.
    assert(hash_simple_array::<PoseidonHasher>([x, y]) == poseidon_expected_hash);

    // Now let's do the same logic but with a different `Hasher` supplied to the turbofish operator
    // We want to make sure that we have correctly monomorphized a function with a trait generic
    // where the generic is not used on any function parameters or the return value.
    let mut hasher = Poseidon2Hasher::default();
    hasher.write(x);
    hasher.write(y);
    let poseidon2_expected_hash = hasher.finish();
    assert(hash_simple_array::<Poseidon2Hasher>([x, y]) == poseidon2_expected_hash);
}

fn hash_simple_array<H>(input: [Field; 2]) -> Field
where
    H: Hasher + Default,
{
    // Check that we can call a trait method instead of a trait implementation
    let mut hasher = H::default();
    // Regression that the object is converted to a mutable reference type `&mut _`.
    // Otherwise will see `Expected type &mut _, found type H`.
    // Then we need to make sure to also auto dereference later in the type checking process
    // when searching for a matching impl or else we will get `No matching impl found for `&mut H: Hasher`
    hasher.write(input[0]);
    hasher.write(input[1]);
    hasher.finish()
}
