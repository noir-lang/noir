use noirc_evaluator::ssa::{
    interpreter::tests::{
        executes_with_no_errors, expect_printed_output, expect_value, expect_values,
        expect_values_with_args, from_constant,
    },
    ir::types::NumericType,
};

fn main() {
    let ssa = r#"
g0 = u32 256
g1 = u32 65536
g2 = u32 16777216

acir(inline) fn main f0 {
  b0():
    v6 = call f1(u32 128, u8 3) -> u32                  // test_programs/execution_success/a_1_mul/src/main.nr:36:13
    call f2(v6)                                         // test_programs/execution_success/a_1_mul/src/main.nr:37:5
    return v6
}
acir(inline_always) fn lshift8 f1 {
  b0(v3: u32, v4: u8):
    v10 = eq v4, u8 0                                   // test_programs/execution_success/a_1_mul/src/main.nr:20:12
    jmpif v10 then: b1, else: b2
  b1():
    jmp b3(v3)
  b2():
    v12 = eq v4, u8 1                                   // test_programs/execution_success/a_1_mul/src/main.nr:22:19
    jmpif v12 then: b4, else: b5
  b3(v5: u32):
    return v5
  b4():
    v20 = mul v3, u32 256                               // test_programs/execution_success/a_1_mul/src/main.nr:23:13
    jmp b6(v20)
  b5():
    v14 = eq v4, u8 2                                   // test_programs/execution_success/a_1_mul/src/main.nr:24:19
    jmpif v14 then: b7, else: b8
  b6(v6: u32):
    jmp b3(v6)
  b7():
    v19 = mul v3, u32 65536                             // test_programs/execution_success/a_1_mul/src/main.nr:25:13
    jmp b9(v19)
  b8():
    v16 = eq v4, u8 3                                   // test_programs/execution_success/a_1_mul/src/main.nr:26:19
    jmpif v16 then: b10, else: b11
  b9(v7: u32):
    jmp b6(v7)
  b10():
    v18 = mul v3, u32 16777216                          // test_programs/execution_success/a_1_mul/src/main.nr:27:13
    jmp b12(v18)
  b11():
    jmp b12(u32 0)
  b12(v8: u32):
    jmp b9(v8)
}
acir(inline) fn println f2 {
  b0(v3: u32):
    call f3(u1 1, v3)                                   // std/lib.nr:40:9
    return
}
brillig(inline) fn print_unconstrained f3 {
  b0(v3: u1, v4: u32):
    v24 = make_array b"{\"kind\":\"unsignedinteger\",\"width\":32}"     // std/lib.nr:40:9
    call print(v3, v4, v24, u1 0)                       // std/lib.nr:34:5
    return
}
      "#;
    let values = expect_printed_output(ssa);
    println!("values: {:?}", values);
}
