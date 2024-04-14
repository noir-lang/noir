fn main() {
    let inner_value = 3;
    let none = Option::none();
    let some = Option::some(inner_value);

    assert(some.expect(f"Should have the value {inner_value}") == 3);
    assert(none.expect(f"Should have the value {inner_value}") == 3);
}
