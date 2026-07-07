pub fn main(input: [u32; 1]) -> pub u32 {
    let mut vec = input.as_vector();

    let (_vec, _) = vec.pop_back();
    vec = _vec;

    vec[1] = 10;

    vec[0]
}
