fn constrained_le_decompose(x: Field) -> [u8; 16] {
    let x_bytes: [u8; 16] = x.to_le_bytes();
    println(x);
    println(x_bytes);
    x_bytes
}

unconstrained fn brillig_le_decompose(x: Field) -> [u8; 16] {
    constrained_le_decompose(x)
}

#[test]
fn test_acir_field_fits() {
    let _ = constrained_le_decompose(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
}

#[test(should_fail_with = "Field failed to decompose into specified 16 limbs")]
fn test_acir_field_too_big() {
    // One greater than `0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF`
    let _ = constrained_le_decompose(0x100000000000000000000000000000000);
}

// We also check in Brillig that the input field fits in the limbs decomposition
// so that acir and brillig behavior is consistent.
#[test(should_fail_with = "Field failed to decompose into specified 16 limbs")]
fn test_brillig() {
    // Safety: testing context
    let _ = unsafe { brillig_le_decompose(0x100000000000000000000000000000000) };
}
