pub fn main(input: [u32; 1], b: bool) -> pub u32 {
    let mut s = input.as_vector();

    if (b) {
        let (_, new_vector) = s.pop_front();
        s = new_vector;
    } else {
        s = s.push_front(5);
    }

    let (ret, _) = s.pop_front();

    ret
}
