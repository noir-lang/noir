use noir_field::FieldElement;
use std::convert::TryInto;

use super::Barretenberg;
use super::field_to_array;

impl Barretenberg {
    pub fn compress_native(&mut self, left: &FieldElement, right: &FieldElement) -> FieldElement {
        let result_bytes = barretenberg_wrapper::pedersen::compress_native(
            left.to_bytes().as_slice().try_into().unwrap(),
            right.to_bytes().as_slice().try_into().unwrap(),
        );
        FieldElement::from_be_bytes_reduce(&result_bytes)
    }

    pub fn compress_many(&mut self, inputs: Vec<FieldElement>) -> FieldElement {
        let mut inputs_buf = Vec::new();
        for f in inputs {
            inputs_buf.push(field_to_array(&f));
        }
        let result = barretenberg_wrapper::pedersen::compress_many(&inputs_buf);
        FieldElement::from_be_bytes_reduce(&result)
    }

    pub fn encrypt(&mut self, inputs: Vec<FieldElement>) -> (FieldElement, FieldElement) {
        let mut inputs_buf = Vec::new();
        for f in inputs {
            inputs_buf.push(field_to_array(&f));
        }
        let (point_x_bytes, point_y_bytes) = barretenberg_wrapper::pedersen::encrypt(&inputs_buf);
        let point_x = FieldElement::from_be_bytes_reduce(&point_x_bytes);
        let point_y = FieldElement::from_be_bytes_reduce(&point_y_bytes);

        (point_x, point_y)
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
#[test]
fn pedersen_hash_to_point() {
    let mut barretenberg = Barretenberg::new();
    let (x, y) = barretenberg.encrypt(vec![FieldElement::zero(), FieldElement::one()]);
    let expected_x = FieldElement::from_hex(
        "0x108800e84e0f1dafb9fdf2e4b5b311fd59b8b08eaf899634c59cc985b490234b",
    )
    .unwrap();
    let expected_y = FieldElement::from_hex(
        "0x2d43ef68df82e0adf74fed92b1bc950670b9806afcfbcda08bb5baa6497bdf14",
    )
    .unwrap();
    assert_eq!(expected_x, x);
    assert_eq!(expected_y, y);
}
