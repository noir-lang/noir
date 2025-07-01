fn constrained_le_decompose(x: Field) -> [u8; 16] {
    let x_bytes: [u8; 16] = x.to_le_bytes();
    println(x);
    println(x_bytes);
    x_bytes
}

unconstrained fn brillig_le_decompose(x: Field) -> [u8; 16] {
    constrained_le_decompose(x)
}

#[test(should_fail_with = "Field failed to decompose into specified 16 limbs")]
fn test_acir() {
    let _ = constrained_le_decompose(
        0x0f84cda3598b8e2712c91f5c6a2b347f2289c66663137b9c2f02f05253c31ce1,
    );
}

// We do not constrain a decomposition in Brillig. We instead just write up to the number of specified limbs,
// thus this does not fail in Brillig where the number of limbs determines the correctness of the byte decomposition.
#[test]
fn test_brillig() {
    // Safety: testing context
    let _ = unsafe {
        brillig_le_decompose(
            0x0f84cda3598b8e2712c91f5c6a2b347f2289c66663137b9c2f02f05253c31ce1,
        )
    };
}
