fn main() {
    let four: u8 = 4;
    let not_four: u8 = !four;

    let five: u8 = 5;
    let not_five: u8 = !five;
    assert(not_four != not_five);
}
