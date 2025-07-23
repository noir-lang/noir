use noirc_evaluator::ssa::{
    interpreter::tests::{
        executes_with_no_errors, expect_value, expect_values, expect_values_with_args,
        from_constant,
    },
    ir::types::NumericType,
};

fn main() {
    let ssa = "
    acir(inline) fn main f0 {
          b0():
            v0 = cast u32 2 as Field
            v1 = cast u32 3 as u8
            v2 = cast i8 255 as i32   // -1, remains as 255
            v3 = cast i8 255 as u128  // also zero-extended, remains 255
                                      // casts like this should be sign-extended in Noir
                                      // but we rely on other SSA instructions to manually do this.
            return v0, v1, v2, v3
        }
      ";
    let values = expect_values(ssa);
    println!("values: {:?}", from_constant(255_u32.into(), NumericType::signed(32)));
    assert_eq!(values[0], from_constant(2_u32.into(), NumericType::NativeField));
    assert_eq!(values[1], from_constant(3_u32.into(), NumericType::unsigned(8)));
    assert_eq!(values[2], from_constant(i32::from(-1).into(), NumericType::signed(32)));
    assert_eq!(values[3], from_constant(255_u32.into(), NumericType::unsigned(128)));
}
