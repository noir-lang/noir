// Converts an array into a slice.
fn as_slice_push<T, N>(xs: [T; N]) -> [T] {
    let mut slice = &[];
    for elem in xs {
        slice = slice.push_back(elem);
    }
    slice
}

fn main(x: Field, y: Field) {
    let z = 3;
    let w = 4;

    let mut dynamic_array = [0, 0];
    dynamic_array[x] = z;
    dynamic_array[y] = w;

    let literal_slice = &[z, w];
    let convert_dynamic = dynamic_array.as_slice();
    let convert_literal = [z, w].as_slice();

    assert(literal_slice.len() == 2);
    assert(convert_dynamic.len() == 2);
    assert(convert_literal.len() == 2);

    assert(literal_slice[x] == z);
    assert(convert_dynamic[x] == z);
    assert(convert_literal[x] == z);

    assert(literal_slice[y] == w);
    assert(convert_dynamic[y] == w);
    assert(convert_literal[y] == w);

    let hash_literal_slice = dep::std::hash::pedersen_hash(literal_slice);
    let hash_convert_dynamic = dep::std::hash::pedersen_hash(convert_dynamic);
    let hash_convert_literal = dep::std::hash::pedersen_hash(convert_literal);

    assert(hash_literal_slice == hash_convert_dynamic);
    assert(hash_literal_slice == hash_convert_literal);
}
