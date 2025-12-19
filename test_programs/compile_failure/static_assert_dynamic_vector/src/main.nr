fn main(
    dynamic_one: Field, // == 1
    dynamic_two: Field, // == 2
) {
    // length unknown at compile time
    let mut dynamic_built_vector_pair = &[];
    if dynamic_one == 1 {
        dynamic_built_vector_pair = dynamic_built_vector_pair.push_back(dynamic_one);
    }
    if dynamic_two == 2 {
        dynamic_built_vector_pair = dynamic_built_vector_pair.push_back(dynamic_two);
    }

    std::static_assert(dynamic_built_vector_pair.len() == 3, "");
}
