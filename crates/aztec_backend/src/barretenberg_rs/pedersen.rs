use noir_field::FieldElement;
use wasmer::Value;

use super::{composer::Assignments, Barretenberg};

impl Barretenberg {
    pub fn compress_native(&mut self, left: &FieldElement, right: &FieldElement) -> FieldElement {
        let lhs_ptr = self.allocate(&left.to_bytes()); // 0..32
        let rhs_ptr = self.allocate(&right.to_bytes()); // 32..64
        let result_ptr = Value::I32(64); // 64..96
        self.call_multiple(
            "pedersen_compress_fields",
            vec![&lhs_ptr, &rhs_ptr, &result_ptr],
        );

        let result_bytes = self.slice_memory(64, 96);
        FieldElement::from_bytes(&result_bytes)
    }
    pub fn compress_many(&mut self, inputs: Vec<FieldElement>) -> FieldElement {
        let input_buf = Assignments(inputs).to_bytes();
        let input_ptr = self.allocate(&input_buf);

        self.call_multiple("pedersen_compress", vec![&input_ptr, &Value::I32(0)]);

        let result_bytes = self.slice_memory(0, 32);
        FieldElement::from_bytes(&result_bytes)
    }
}

#[test]
fn basic_interop() {
    // Expected values were taken from Barretenberg by running `crypto::pedersen::compress_native`
    // printing the result in hex to `std::cout` and copying
    struct Test<'a> {
        input_left: FieldElement,
        input_right: FieldElement,
        expected_hex: &'a str,
    }

    let tests = vec![
        Test {
            input_left: FieldElement::zero(),
            input_right: FieldElement::one(),
            expected_hex: "0x108800e84e0f1dafb9fdf2e4b5b311fd59b8b08eaf899634c59cc985b490234b",
        },
        Test {
            input_left: FieldElement::one(),
            input_right: FieldElement::one(),
            expected_hex: "0x00f1c7ea35a4cf7ea5e678fcc2a5fac5351a563a3ff021f0c4a4126462aa081f",
        },
        Test {
            input_left: FieldElement::one(),
            input_right: FieldElement::zero(),
            expected_hex: "0x2619a3512420b4d3c72e43fdadff5f5a3ec1b0e7d75cd1482159a7e21f6c6d6a",
        },
    ];

    let mut barretenberg = Barretenberg::new();
    for test in tests {
        let expected = FieldElement::from_hex(test.expected_hex).unwrap();

        let got = barretenberg.compress_native(&test.input_left, &test.input_right);
        let got_many = barretenberg.compress_many(vec![test.input_left, test.input_right]);
        assert_eq!(got, expected);
        assert_eq!(got, got_many);
    }
}
