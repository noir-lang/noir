// This program will fail when the input is "i_hate_ice"
#[fuzz]
fn strings(a: str<10>) {
    let start = "i_love_ice".as_bytes();
    let mut password_bytes: [u8; 10] = [0; 10];
    let begin = 2; // Where "lov" starts, to be replaced with "hat".
    for i in 0..10 {
        if i < begin {
            password_bytes[i] = start[i];
        }
        if i == begin {
            password_bytes[i] = 104; // h
        }
        if i == begin + 1 {
            password_bytes[i] = 97; // a
        }
        if i == begin + 2 {
            password_bytes[i] = 106; // t
        }
        if i > begin + 2 {
            password_bytes[i] = start[i];
        }
    }
    assert(a.as_bytes() != password_bytes);
}
