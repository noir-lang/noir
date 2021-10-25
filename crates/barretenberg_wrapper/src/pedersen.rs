use crate::bindings::pedersen;

//////PEDERSEN///////////////////////
pub fn compress_native(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut result = [0_u8; 32];

    unsafe {
        pedersen::pedersen_compress_fields(
            left.as_ptr() as *const u8,
            right.as_ptr() as *const u8,
            result.as_mut_ptr(),
        );
    }
    result
}

//pub fn compress_many(inputs: Vec<[u8; 32]>) -> [u8; 32] {
pub fn compress_many(inputs: &Vec<[u8; 32]>) -> [u8; 32] {
    //convert inputs into one buffer: length + data
    let mut buffer = Vec::new();
    let witness_len = inputs.len() as u32;
    buffer.extend_from_slice(&witness_len.to_be_bytes());
    for assignment in &*inputs {
        buffer.extend_from_slice(assignment);
    }

    let mut result = [0_u8; 32];
    unsafe {
        pedersen::pedersen_compress(buffer.as_ptr() as *const u8, result.as_mut_ptr());
    }
    result
}

pub fn encrypt(inputs_buffer: &Vec<[u8; 32]>) -> ([u8; 32], [u8; 32]) {
    let mut buffer = Vec::new();
    let buffer_len = inputs_buffer.len() as u32;
    let mut result = [0_u8; 64];
    buffer.extend_from_slice(&buffer_len.to_be_bytes());
    for e in &*inputs_buffer {
        buffer.extend_from_slice(e);
    }

    unsafe {
        pedersen::pedersen_encrypt(buffer.as_ptr() as *const u8, result.as_mut_ptr());
    }
    (
        *slice_as_array!(&result[0..32], [u8; 32]).unwrap(),
        *slice_as_array!(&result[32..64], [u8; 32]).unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_interop() {
        // Expected values were taken from Barretenberg by running `crypto::pedersen::compress_native`
        // printing the result in hex to `std::cout` and copying
        struct Test<'a> {
            input_left: [u8; 32],
            input_right: [u8; 32],
            expected_hex: &'a str,
        }
        let f_zero = [0_u8; 32];
        let mut f_one = [0_u8; 32];
        f_one[31] = 1;

        let tests = vec![
            Test {
                input_left: f_zero,
                input_right: f_one,
                expected_hex: "108800e84e0f1dafb9fdf2e4b5b311fd59b8b08eaf899634c59cc985b490234b",
            },
            Test {
                input_left: f_one,
                input_right: f_one,
                expected_hex: "00f1c7ea35a4cf7ea5e678fcc2a5fac5351a563a3ff021f0c4a4126462aa081f",
            },
            Test {
                input_left: f_one,
                input_right: f_zero,
                expected_hex: "2619a3512420b4d3c72e43fdadff5f5a3ec1b0e7d75cd1482159a7e21f6c6d6a",
            },
        ];

        for test in tests {
            let got = compress_native(&test.input_left, &test.input_right);
            let mut many_intputs: Vec<[u8; 32]> = Vec::new();
            many_intputs.push(test.input_left);
            many_intputs.push(test.input_right);
            //test.input_left.to_vec().extend(test.input_right);
            let got_many = compress_many(many_intputs);
            //let got_many = compress_many(vec![test.input_left, test.input_right]);
            assert_eq!(hex::encode(got), test.expected_hex);
            assert_eq!(got, got_many);
        }
    }

    #[test]
    fn pedersen_hash_to_point() {
        let f_zero = [0_u8; 32];
        let mut f_one = [0_u8; 32];
        f_one[31] = 1;
        let mut inputs: Vec<[u8; 32]> = Vec::new();
        inputs.push(f_zero);
        inputs.push(f_one);

        let (x, y) = encrypt(inputs);
        let expected_x = "108800e84e0f1dafb9fdf2e4b5b311fd59b8b08eaf899634c59cc985b490234b";
        let expected_y = "2d43ef68df82e0adf74fed92b1bc950670b9806afcfbcda08bb5baa6497bdf14";
        assert_eq!(expected_x, hex::encode(x));
        assert_eq!(expected_y, hex::encode(y));
    }
}
