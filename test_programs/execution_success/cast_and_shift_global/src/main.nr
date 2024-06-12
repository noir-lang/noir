global THREE: u64 = 3;
global EIGHT: u64 = 1 << THREE as u8;
global SEVEN: u64 = EIGHT - 1;

fn main() {
    assert([0; EIGHT] == [0; 8]);
    assert([0; SEVEN] == [0; 7]);
}
