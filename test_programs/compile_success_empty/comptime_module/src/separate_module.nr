#![inner_attribute_separate_module]

comptime fn inner_attribute_separate_module(m: Module) {
    assert_eq(m.name(), quote { separate_module });
    super::increment_counter();
}
