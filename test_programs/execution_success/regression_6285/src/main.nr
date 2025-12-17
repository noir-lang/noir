fn main(x: bool) {
    println(f"x's value: {x}");
    if x {
        let empty_list: [u8] = &[];
        let _ = empty_list.pop_front();
    }
}
