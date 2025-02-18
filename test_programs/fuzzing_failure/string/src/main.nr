// This program will fail when the input is "kev_hates_rust"
#[fuzz]
fn strings(a: str<14>) {
    let start = "kev_loves_rust".as_bytes();
    let mut password_bytes: [u8; 14] = [0; 14];
    for i in 0..14 {
        if i < 4 {
            password_bytes[i] = start[i];
        }
        if i == 4 {
            password_bytes[i] = 104;
        }
        if i == 5 {
            password_bytes[i] = 97;
        }
        if i == 6 {
            password_bytes[i] = 116;
        }
        if i > 6 {
            password_bytes[i] = start[i];
        }
    }
    assert(a.as_bytes() != password_bytes);
}