use noir_field::FieldElement;

use super::Barretenberg;
use super::BARRETENBERG;

impl Barretenberg {
    /// Hashes to a bn254 scalar field element using blake2s
    pub fn hash_to_field(&mut self, input: &[u8]) -> FieldElement {
        let _m = BARRETENBERG.lock().unwrap();
        let result_prt = barretenberg_wrapper::blake2s::hash_to_field(input);

        FieldElement::from_be_bytes_reduce(&result_prt)
    }
}

#[test]
fn basic_interop() {
    // Expected values were taken from barretenberg by running `crypto::pedersen::compress_native`
    // printing the result in hex to `std::cout` and copying
    struct Test<'a> {
        input: Vec<u8>,
        expected_hex: &'a str,
    }

    let tests = vec![
        Test {
            input: vec![0; 64],
            expected_hex: "0x1cdcf02431ba623767fe389337d011df1048dcc24b98ed81cec97627bab454a0",
        },
        Test {
            input: vec![1; 64],
            expected_hex: "0x1aab12b2f330c2fb811d6042f10ce65c0678803354529dc7f9bb5b1d9ff6987b",
        },
        Test {
            input: vec![2; 64],
            expected_hex: "0x06c2335d6f7acb84bbc7d0892cefebb7ca31169a89024f24814d5785e0d05324",
        },
    ];

    let mut barretenberg = Barretenberg::new();
    for test in tests {
        let expected = FieldElement::from_hex(test.expected_hex).unwrap();
        let got = barretenberg.hash_to_field(&test.input);
        assert_eq!(got, expected);
    }
}
