use std::hash::Hasher;
use std::hash::poseidon2::Poseidon2Hasher;

fn main(x: Field, y: pub Field) {
    let mut a_mut_ref = AType { x };

    pass_trait_by_value(a_mut_ref, y);
    assert(a_mut_ref.x == x);

    pass_trait_by_value_impl_param(a_mut_ref, y);
    assert(a_mut_ref.x == x);

    pass_trait_by_mut_ref(&mut a_mut_ref, y);
    assert(a_mut_ref.x == y);

    let mut hasher = Poseidon2Hasher::default();
    hasher.write(x);
    hasher.write(y);
    let expected_hash = hasher.finish();
    // Check that we get the same result when using the hasher in a
    // method that purely uses trait methods without a supplied implementation.
    assert(hash_simple_array::<Poseidon2Hasher>([x, y]) == expected_hash);
}

trait SomeTrait {
    fn set_value(&mut self, new_value: Field) -> ();

    fn get_value(self) -> Field;
}

struct AType {
    x: Field,
}

impl SomeTrait for AType {
    fn set_value(&mut self, new_value: Field) -> () {
        self.x = new_value;
    }

    fn get_value(self) -> Field {
        self.x
    }
}

fn pass_trait_by_value_impl_param(mut a_mut_ref: impl SomeTrait, value: Field) {
    // We auto add a mutable reference to the object type if the method call expects a mutable self
    a_mut_ref.set_value(value);
    assert(a_mut_ref.get_value() == value);
}

fn pass_trait_by_value<T>(mut a_mut_ref: T, value: Field)
where
    T: SomeTrait,
{
    // We auto add a mutable reference to the object type if the method call expects a mutable self
    a_mut_ref.set_value(value);
    assert(a_mut_ref.get_value() == value);
}

fn pass_trait_by_mut_ref<T>(a_mut_ref: &mut T, value: Field)
where
    T: SomeTrait,
{
    // We auto add a mutable reference to the object type if the method call expects a mutable self
    a_mut_ref.set_value(value);
}

fn hash_simple_array<H>(input: [Field; 2]) -> Field
where
    H: Hasher + Default,
{
    // Check that we can call a trait method instead of a trait implementation
    // TODO: Need to remove the need for this type annotation
    // TODO: Curently, without the annotation we will get `Expression type is ambiguous` when trying to use the `hasher`
    let mut hasher: H = H::default();
    // Regression that the object is converted to a mutable reference type `&mut _`.
    // Otherwise will see `Expected type &mut _, found type H`.
    // Then we need to make sure to also auto dereference later in the type checking process
    // when searching for a matching impl or else we will get `No matching impl found for `&mut H: Hasher`
    hasher.write(input[0]);
    hasher.write(input[1]);
    hasher.finish()
}
