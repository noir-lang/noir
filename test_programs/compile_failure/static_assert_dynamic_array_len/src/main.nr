fn main(
    dynamic_one: Field, // == 1
) {
    std::static_assert([1, 2, dynamic_one].len() == 4, "");
}
