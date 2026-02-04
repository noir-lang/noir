fn main() {}

unconstrained fn derive_from_seed(seed: [u8; 2]) -> u128 {
    let lo = seed[0] as u128;
    let hi = seed[1] as u128;
    lo + hi * 256
}

#[test]
unconstrained fn test_nested_array_map_to_u128(seeds: [[u8; 2]; 3]) {
    let fields: [u128; 3] = seeds.map(|seed: [u8; 2]| derive_from_seed(seed));
    for i in 0..3 {
        let expected = seeds[i][0] as u128 + seeds[i][1] as u128 * 256;
        assert_eq(fields[i], expected);
    }
}

#[test]
unconstrained fn test_push_back_nested(seeds: [[u8; 2]; 3]) {
    let mut result: [[u8; 2]] = &[];
    for seed in seeds {
        result = result.push_back(seed);
    }
    assert_eq(result.len(), 3);
    for i in 0..3 {
        assert_eq(result[i][0], seeds[i][0]);
        assert_eq(result[i][1], seeds[i][1]);
    }
}

unconstrained fn batch_process(values: [u128]) {
    // Process each value - just verify they're accessible
    for i in 0..values.len() {
        let _ = values[i];
    }
}

#[test]
unconstrained fn test_nested_array_to_vector(seeds: [[u8; 2]; 3]) {
    let fields: [u128] = seeds.map(|seed: [u8; 2]| derive_from_seed(seed)).as_vector();
    assert_eq(fields.len(), 3);
    batch_process(fields);
}
