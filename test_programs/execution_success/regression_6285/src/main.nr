fn main(x: bool) {
    println(f"x's value: {x}");
    if x {
        let empty_slice: [u8] = &[];
        let _ = empty_slice.pop_front();
    }
}
