pub fn main(input: [u32; 1], b: bool) -> pub u32 {
    let mut s = input.as_vector();

    if (b) {
        let (new_vector, _) = s.pop_back();
        s = new_vector;
    } else {
        s = s.push_front(5);
    }

    let (_, ret) = s.pop_back();

    ret
}
