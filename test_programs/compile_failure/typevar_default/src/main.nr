fn main() {
    // Expecting error: type annotations needed (N not known)
    let _ = list_to_array(&[1, 2, 3]);
}

fn list_to_array<N>(list: [Field]) -> [Field; N] {
    let mut array = [0; N];
    for i in 0 .. N {
        array[i] = list[i];
    }
    array
}
