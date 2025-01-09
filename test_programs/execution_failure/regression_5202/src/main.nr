trait ToField {
    fn to_field(self) -> Field;
}

impl ToField for bool {
    fn to_field(self) -> Field {
        self as Field
    }
}

unconstrained fn get_unconstrained_option() -> Option<u32> {
    Option::some(13)
}

unconstrained fn should_i_assert() -> bool {
    false
}

fn get_magical_boolean() -> bool {
    /// Safety: testing context
    let option = unsafe { get_unconstrained_option() };

    let pre_assert = option.is_some().to_field();

    /// Safety: testing context
    if unsafe { should_i_assert() } {
        // Note that `should_i_assert` is unconstrained, so Noir should not be able to infer
        // any behavior from the contents of this block. In this case it is actually false, so the
        // assertion below is not even executed (if it did it'd fail since the values are not equal).
        assert_eq(option, Option::some(42)); // <- this seems to be the trigger for the bug
    }

    // In my testing, the `option` value exhibits weird behavior from this point on, as if it had been mutated
    let post_assert = option.is_some().to_field();

    // The following expression should be true, but I can get it to evaluate to false depending on how I call it
    pre_assert == post_assert
}

fn main() {
    let magic = get_magical_boolean();

    // One of these asserts should fail. Before #5202, they would both pass
    assert_eq(magic, true);
    assert_eq(magic, false);
}
