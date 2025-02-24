global SIZE: u32 = 30;

fn main(input: [[u8; 2]; SIZE]) -> pub [[u8; 32]; SIZE] {
    let mut results: [[u8; 32]; SIZE] = [[0; 32]; SIZE];
    for i in 0..SIZE {
        results[i] = std::hash::sha256(input[i]);
    }

    results
}
