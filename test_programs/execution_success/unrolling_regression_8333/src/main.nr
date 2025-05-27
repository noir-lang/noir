// Regression for issue #8333 (https://github.com/noir-lang/noir/issues/8333)
fn main() -> pub [[u8; 32]; 2] {
    let mut result = [[0; 32]; 2];
    let actions_data: [Field; 30] = [0; 30];
    for i in 0..1 {
        result = [actions_data[i].to_be_bytes::<32>(), actions_data[i + 1].to_be_bytes::<32>()];
    }
    result
}
