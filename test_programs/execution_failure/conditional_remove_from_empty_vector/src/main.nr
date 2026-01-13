pub fn main(input: [u32; 1], b: bool) -> pub u32 {
    let mut s = input.as_vector();

    if (b) {
        let (new_slice, _) = s.remove(0);
        s = new_slice;
    } else {
        s = s.push_front(5);
    }

    let (_, ret) = s.remove(0);

    ret
}
