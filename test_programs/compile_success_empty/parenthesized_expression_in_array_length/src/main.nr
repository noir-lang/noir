global N = 100;
global BLOCK_SIZE = 10;

fn main() {
    let _: [Field; 110] = [0; ((N + BLOCK_SIZE) * BLOCK_SIZE) / BLOCK_SIZE];
}
