use dep::protocol_types::{constants::GENERATOR_INDEX__SYMMETRIC_KEY, scalar::Scalar, point::Point, utils::arr_copy_slice};
use crate::utils::point::point_to_bytes;
use std::{hash::sha256, embedded_curve_ops::multi_scalar_mul};

// TODO(#5726): This function is called deriveAESSecret in TS. I don't like point_to_symmetric_key name much since
// point is not the only input of the function. Unify naming with TS once we have a better name.
pub fn point_to_symmetric_key(secret: Scalar, point: Point) -> [u8; 32] {
    let shared_secret: Point = multi_scalar_mul([point], [secret]);
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/6061): make the func return Point struct directly
    let shared_secret = point_to_bytes(shared_secret);
    let mut shared_secret_bytes_with_separator = [0 as u8; 33];
    shared_secret_bytes_with_separator = arr_copy_slice(shared_secret, shared_secret_bytes_with_separator, 0);
    shared_secret_bytes_with_separator[32] = GENERATOR_INDEX__SYMMETRIC_KEY;
    sha256(shared_secret_bytes_with_separator)
}

#[test]
fn check_point_to_symmetric_key() {
    // Value taken from "derive shared secret" test in encrypt_buffer.test.ts
    let secret = Scalar {
        lo: 0x00000000000000000000000000000000649e7ca01d9de27b21624098b897babd,
        hi: 0x0000000000000000000000000000000023b3127c127b1f29a7adff5cccf8fb06
    };
    let point = Point {
        x: 0x2688431c705a5ff3e6c6f2573c9e3ba1c1026d2251d0dbbf2d810aa53fd1d186,
        y: 0x1e96887b117afca01c00468264f4f80b5bb16d94c1808a448595f115556e5c8e,
        is_infinite: false
    };

    let key = point_to_symmetric_key(secret, point);
    // The following value gets updated when running encrypt_buffer.test.ts with AZTEC_GENERATE_TEST_DATA=1
    let expected_key = [
        217, 245, 196, 116, 55, 39, 202, 184, 117, 231, 19, 56, 102, 254, 94, 14, 172, 169, 123, 96, 61, 247, 209, 140, 4, 132, 119, 222, 79, 1, 154, 136
    ];
    assert_eq(key, expected_key);
}
