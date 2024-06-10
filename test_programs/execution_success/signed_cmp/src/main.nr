fn main(minus_one: i8) {
    assert(minus_one < 0);
    assert(0 < minus_one as u8);
    assert(0 > minus_one);
    let most_negative_number = minus_one * 127 - 1;
    assert(most_negative_number < 0);
    assert(127 > most_negative_number);
}
