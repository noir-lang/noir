fn main() {
    // Expecting error: type annotations needed (N not known)
    let _ = vector_to_array(&[1, 2, 3]);
}

fn vector_to_array<N>(vector: [Field]) -> [Field; N] {
    let mut array = [0; N];
    for i in 0 .. N {
        array[i] = vector[i];
    }
    array
}
