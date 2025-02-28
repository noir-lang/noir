use std::validation::AssertsIsValidInput;

struct Option2<T> {
    _is_some: bool,
    _value: T,
}

struct SelfReferential {
    prop: Option2<SelfReferential>,
}

impl AssertsIsValidInput for SelfReferential {
    fn assert_is_valid_input(self) {}
}

fn main(x: SelfReferential) {
    assert(x._is_some);
}
