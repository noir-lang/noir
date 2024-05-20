use dep::std::hash::Hasher;
use dep::std::hash::poseidon2::Poseidon2Hasher;

fn main(x: Field, y: pub Field) {
    let mut a_mut_ref = AType { x };

    pass_trait_by_value(a_mut_ref, y);
    assert(a_mut_ref.x == x);

    pass_trait_by_value_impl_param(a_mut_ref, y);
    assert(a_mut_ref.x == x);

    pass_trait_by_mut_ref(&mut a_mut_ref, y);
    assert(a_mut_ref.x == y);
}

trait SomeTrait {
    fn set_value(&mut self, new_value: Field) -> ();

    fn get_value(self) -> Field;
}

struct AType {
    x: Field
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

fn pass_trait_by_value<T>(mut a_mut_ref: T, value: Field) where T: SomeTrait {
    // We auto add a mutable reference to the object type if the method call expects a mutable self
    a_mut_ref.set_value(value);
    assert(a_mut_ref.get_value() == value);
}

fn pass_trait_by_mut_ref<T>(a_mut_ref: &mut T, value: Field) where T: SomeTrait {
    // We auto add a mutable reference to the object type if the method call expects a mutable self
    a_mut_ref.set_value(value);
}
