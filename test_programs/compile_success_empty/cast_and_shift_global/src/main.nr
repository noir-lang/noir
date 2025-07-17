global THREE: u32 = 3;
global EIGHT: u32 = 1 << THREE as u8;
global SEVEN: u32 = EIGHT - 1;

fn main() {
    assert([0; EIGHT] == [0; 8]);
    assert([0; SEVEN] == [0; 7]);
}
