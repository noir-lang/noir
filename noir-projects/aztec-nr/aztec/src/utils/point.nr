use dep::protocol_types::point::Point;

// I am storing the modulus divided by 2 plus 1 here because full modulus would throw "String literal too large" error
// Full modulus is 21888242871839275222246405745257275088548364400416034343698204186575808495617
global BN254_FR_MODULUS_DIV_2: Field = 10944121435919637611123202872628637544274182200208017171849102093287904247808;

/// Converts a public key to a byte array.
///
/// We don't serialize the point at infinity flag because this function is used in situations where we do not want
/// to waste the extra byte (encrypted log).
pub fn point_to_bytes(pk: Point) -> [u8; 32] {
    assert(!pk.is_infinite, "Cannot serialize point at infinity as bytes.");

    let mut result = pk.x.to_be_bytes(32);

    // We store only a "sign" of the y coordinate because the rest can be derived from the x coordinate. To get
    // the sign we check if the y coordinate is greater than the curve's order minus 1 divided by 2.
    if !BN254_FR_MODULUS_DIV_2.lt(pk.y) {
        // y is <= (modulus - 1) / 2 so we set the sign bit to 1
        // Here we leverage that field fits into 254 bits (log2(Fr.MODULUS) < 254) and given that we serialize Fr to 32
        // bytes and we use big-endian the 2 most significant bits are never populated. Hence we can use one of
        // the bits as a sign bit.
        result[0] += 128;
    }

    result.as_array()
}

mod test {
    use dep::protocol_types::point::Point;
    use crate::utils::point::point_to_bytes;

    #[test]
    fn test_point_to_bytes_positive_sign() {
        let p = Point {
            x: 0x1af41f5de96446dc3776a1eb2d98bb956b7acd9979a67854bec6fa7c2973bd73,
            y: 0x07fc22c7f2c7057571f137fe46ea9c95114282bc95d37d71ec4bfb88de457d4a,
            is_infinite: false
        };

        let compressed_point = point_to_bytes(p);

        let expected_compressed_point_positive_sign = [
            154, 244, 31, 93, 233, 100, 70, 220, 55, 118, 161, 235, 45, 152, 187, 149, 107, 122, 205, 153, 121, 166, 120, 84, 190, 198, 250, 124, 41, 115, 189, 115
        ];

        assert_eq(expected_compressed_point_positive_sign.len(), compressed_point.len());
        for i in 0..expected_compressed_point_positive_sign.len() {
            assert_eq(compressed_point[i], expected_compressed_point_positive_sign[i]);
        }
    }

    #[test]
    fn test_point_to_bytes_negative_sign() {
        let p = Point {
            x: 0x247371652e55dd74c9af8dbe9fb44931ba29a9229994384bd7077796c14ee2b5,
            y: 0x26441aec112e1ae4cee374f42556932001507ad46e255ffb27369c7e3766e5c0,
            is_infinite: false
        };

        let compressed_point = point_to_bytes(p);

        let expected_compressed_point_negative_sign = [
            36, 115, 113, 101, 46, 85, 221, 116, 201, 175, 141, 190, 159, 180, 73, 49, 186, 41, 169, 34, 153, 148, 56, 75, 215, 7, 119, 150, 193, 78, 226, 181
        ];

        assert_eq(expected_compressed_point_negative_sign.len(), compressed_point.len());
        for i in 0..expected_compressed_point_negative_sign.len() {
            assert_eq(compressed_point[i], expected_compressed_point_negative_sign[i]);
        }
    }
}
