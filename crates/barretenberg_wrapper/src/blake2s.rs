extern crate hex;

//use crate::bindings;
use crate::bindings::blake2s;

pub fn hash_to_field(/*&mut self,*/ input: &[u8]) -> [u8; 32] {
    let mut r = [0_u8; 32];
    let data = input.as_ptr() as *const u8;
    unsafe {
        blake2s::blake2s_to_field(data, input.len() as u64, r.as_mut_ptr());
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black2s() {
        struct Test<'a> {
            input: Vec<u8>,
            expected_hex: &'a str,
        }

        let tests = vec![
            Test {
                input: vec![0; 64],
                expected_hex: "1cdcf02431ba623767fe389337d011df1048dcc24b98ed81cec97627bab454a0",
            },
            Test {
                input: vec![1; 64],
                expected_hex: "1aab12b2f330c2fb811d6042f10ce65c0678803354529dc7f9bb5b1d9ff6987b",
            },
            Test {
                input: vec![2; 64],
                expected_hex: "06c2335d6f7acb84bbc7d0892cefebb7ca31169a89024f24814d5785e0d05324",
            },
        ];
        for test in tests {
            let r = hash_to_field(&test.input);
            assert_eq!(hex::encode(r), test.expected_hex);
        }
    }
}
