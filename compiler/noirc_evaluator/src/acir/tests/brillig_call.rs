use acvm::{acir::circuit::brillig::BrilligFunctionId, assert_circuit_snapshot};

use crate::{
    acir::tests::ssa_to_acir_program_with_debug_info,
    brillig::BrilligOptions,
    ssa::{
        function_builder::FunctionBuilder,
        ir::{
            instruction::BinaryOp,
            map::Id,
            types::{NumericType, Type},
        },
    },
};

// Test that given multiple calls to the same brillig function we generate only one bytecode
// and the appropriate Brillig call opcodes are generated
#[test]
fn multiple_brillig_calls_one_bytecode() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: Field, v1: Field):
        v4 = call f1(v0, v1) -> Field
        v5 = call f1(v0, v1) -> Field
        v6 = call f1(v0, v1) -> Field
        v7 = call f2(v0, v1) -> Field
        v8 = call f1(v0, v1) -> Field
        v9 = call f2(v0, v1) -> Field
        return
    }
    brillig(inline) fn foo f1 {
      b0(v0: Field, v1: Field):
        v2 = eq v0, v1
        constrain v2 == u1 0
        return v0
    }
    brillig(inline) fn foo f2 {
      b0(v0: Field, v1: Field):
        v2 = eq v0, v1
        constrain v2 == u1 0
        return v0
    }
    ";
    let (program, debug) = ssa_to_acir_program_with_debug_info(src);

    let main_debug = &debug[0];
    // We have two normal Brillig functions that were called multiple times.
    // We should have a single locations map for each function's debug metadata.
    assert_eq!(main_debug.brillig_locations.len(), 2);
    assert!(main_debug.brillig_locations.contains_key(&BrilligFunctionId(0)));
    assert!(main_debug.brillig_locations.contains_key(&BrilligFunctionId(1)));

    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w7
    private parameters: [w0, w1]
    public parameters: []
    return values: []
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w2]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w3]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w4]
    BRILLIG CALL func 1: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w5]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w6]
    BRILLIG CALL func 1: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w7]
    
    unconstrained func 0
    [Const { destination: Direct(2), bit_size: Integer(U32), value: 1 }, Const { destination: Direct(1), bit_size: Integer(U32), value: 32839 }, Const { destination: Direct(0), bit_size: Integer(U32), value: 3 }, Const { destination: Relative(3), bit_size: Integer(U32), value: 2 }, Const { destination: Relative(4), bit_size: Integer(U32), value: 0 }, CalldataCopy { destination_address: Direct(32836), size_address: Relative(3), offset_address: Relative(4) }, Mov { destination: Relative(1), source: Direct(32836) }, Mov { destination: Relative(2), source: Direct(32837) }, Call { location: 14 }, Call { location: 15 }, Mov { destination: Direct(32838), source: Relative(1) }, Const { destination: Relative(2), bit_size: Integer(U32), value: 32838 }, Const { destination: Relative(3), bit_size: Integer(U32), value: 1 }, Stop { return_data: HeapVector { pointer: Relative(2), size: Relative(3) } }, Return, Call { location: 23 }, BinaryFieldOp { destination: Relative(3), op: Equals, lhs: Relative(1), rhs: Relative(2) }, Const { destination: Relative(2), bit_size: Integer(U1), value: 0 }, BinaryIntOp { destination: Relative(4), op: Equals, bit_size: U1, lhs: Relative(3), rhs: Relative(2) }, JumpIf { condition: Relative(4), location: 22 }, Const { destination: Relative(5), bit_size: Integer(U32), value: 0 }, Trap { revert_data: HeapVector { pointer: Direct(1), size: Relative(5) } }, Return, Const { destination: Direct(32772), bit_size: Integer(U32), value: 30720 }, BinaryIntOp { destination: Direct(32771), op: LessThan, bit_size: U32, lhs: Direct(0), rhs: Direct(32772) }, JumpIf { condition: Direct(32771), location: 28 }, IndirectConst { destination_pointer: Direct(1), bit_size: Integer(U64), value: 15764276373176857197 }, Trap { revert_data: HeapVector { pointer: Direct(1), size: Direct(2) } }, Return]
    unconstrained func 1
    [Const { destination: Direct(2), bit_size: Integer(U32), value: 1 }, Const { destination: Direct(1), bit_size: Integer(U32), value: 32839 }, Const { destination: Direct(0), bit_size: Integer(U32), value: 3 }, Const { destination: Relative(3), bit_size: Integer(U32), value: 2 }, Const { destination: Relative(4), bit_size: Integer(U32), value: 0 }, CalldataCopy { destination_address: Direct(32836), size_address: Relative(3), offset_address: Relative(4) }, Mov { destination: Relative(1), source: Direct(32836) }, Mov { destination: Relative(2), source: Direct(32837) }, Call { location: 14 }, Call { location: 15 }, Mov { destination: Direct(32838), source: Relative(1) }, Const { destination: Relative(2), bit_size: Integer(U32), value: 32838 }, Const { destination: Relative(3), bit_size: Integer(U32), value: 1 }, Stop { return_data: HeapVector { pointer: Relative(2), size: Relative(3) } }, Return, Call { location: 23 }, BinaryFieldOp { destination: Relative(3), op: Equals, lhs: Relative(1), rhs: Relative(2) }, Const { destination: Relative(2), bit_size: Integer(U1), value: 0 }, BinaryIntOp { destination: Relative(4), op: Equals, bit_size: U1, lhs: Relative(3), rhs: Relative(2) }, JumpIf { condition: Relative(4), location: 22 }, Const { destination: Relative(5), bit_size: Integer(U32), value: 0 }, Trap { revert_data: HeapVector { pointer: Direct(1), size: Relative(5) } }, Return, Const { destination: Direct(32772), bit_size: Integer(U32), value: 30720 }, BinaryIntOp { destination: Direct(32771), op: LessThan, bit_size: U32, lhs: Direct(0), rhs: Direct(32772) }, JumpIf { condition: Direct(32771), location: 28 }, IndirectConst { destination_pointer: Direct(1), bit_size: Integer(U64), value: 15764276373176857197 }, Trap { revert_data: HeapVector { pointer: Direct(1), size: Direct(2) } }, Return]
    ");
}

// Test that given multiple primitive operations that are represented by Brillig directives (e.g. invert/quotient),
// we will only generate one bytecode and the appropriate Brillig call opcodes are generated.
#[test]
fn multiple_brillig_stdlib_calls() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v3 = div v0, v1
        constrain v3 == v2
        v4 = div v1, v2
        constrain v4 == u32 1
        return
    }";
    let (program, debug) = ssa_to_acir_program_with_debug_info(src);
    // We expect two brillig functions:
    //   - Quotient (shared between both divisions)
    //   - Inversion, caused by division-by-zero check (shared between both divisions)
    assert_eq!(
        program.unconstrained_functions.len(),
        2,
        "Should only have generated two Brillig functions"
    );
    assert_eq!(
        debug[0].brillig_locations.len(),
        0,
        "Brillig stdlib functions do not have location information"
    );

    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w10
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    BLACKBOX::RANGE [(w0, 32)] []
    BLACKBOX::RANGE [(w1, 32)] []
    BLACKBOX::RANGE [(w2, 32)] []
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w1) 0 ]], outputs: [w3]
    EXPR [ (1, w1, w3) -1 ]
    BRILLIG CALL func 1: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w4, w5]
    BLACKBOX::RANGE [(w4, 32)] []
    BLACKBOX::RANGE [(w5, 32)] []
    EXPR [ (1, w1) (-1, w5) (-1, w6) -1 ]
    BLACKBOX::RANGE [(w6, 32)] []
    EXPR [ (-1, w1, w4) (1, w0) (-1, w5) 0 ]
    EXPR [ (-1, w2) (1, w4) 0 ]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w2) 0 ]], outputs: [w7]
    EXPR [ (1, w2, w7) -1 ]
    BRILLIG CALL func 1: inputs: [EXPR [ (1, w1) 0 ], EXPR [ (1, w2) 0 ]], outputs: [w8, w9]
    BLACKBOX::RANGE [(w9, 32)] []
    EXPR [ (1, w2) (-1, w9) (-1, w10) -1 ]
    BLACKBOX::RANGE [(w10, 32)] []
    EXPR [ (-1, w2, w8) (1, w1) (-1, w9) 0 ]
    EXPR [ (1, w8) -1 ]
    
    unconstrained func 0
    [Const { destination: Direct(21), bit_size: Integer(U32), value: 1 }, Const { destination: Direct(20), bit_size: Integer(U32), value: 0 }, CalldataCopy { destination_address: Direct(0), size_address: Direct(21), offset_address: Direct(20) }, Const { destination: Direct(2), bit_size: Field, value: 0 }, BinaryFieldOp { destination: Direct(3), op: Equals, lhs: Direct(0), rhs: Direct(2) }, JumpIf { condition: Direct(3), location: 8 }, Const { destination: Direct(1), bit_size: Field, value: 1 }, BinaryFieldOp { destination: Direct(0), op: Div, lhs: Direct(1), rhs: Direct(0) }, Stop { return_data: HeapVector { pointer: Direct(20), size: Direct(21) } }]
    unconstrained func 1
    [Const { destination: Direct(10), bit_size: Integer(U32), value: 2 }, Const { destination: Direct(11), bit_size: Integer(U32), value: 0 }, CalldataCopy { destination_address: Direct(0), size_address: Direct(10), offset_address: Direct(11) }, BinaryFieldOp { destination: Direct(2), op: IntegerDiv, lhs: Direct(0), rhs: Direct(1) }, BinaryFieldOp { destination: Direct(1), op: Mul, lhs: Direct(2), rhs: Direct(1) }, BinaryFieldOp { destination: Direct(1), op: Sub, lhs: Direct(0), rhs: Direct(1) }, Mov { destination: Direct(0), source: Direct(2) }, Stop { return_data: HeapVector { pointer: Direct(11), size: Direct(10) } }]
    ");
}

// Test that given both hardcoded Brillig directives and calls to normal Brillig functions,
// we generate a single bytecode for the directives and a single bytecode for the normal Brillig calls.
#[test]
fn brillig_stdlib_calls_with_regular_brillig_call() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v4 = div v0, v1
        constrain v4 == v2
        v5 = call f1(v0, v1) -> u32
        v6 = call f1(v0, v1) -> u32
        v7 = div v1, v2
        constrain v7 == u32 1
        return
    }
    brillig(inline) fn foo f1 {
      b0(v0: u32, v1: u32):
        v2 = eq v0, v1
        constrain v2 == u1 0
        return v0
    }
    ";
    let (program, debug) = ssa_to_acir_program_with_debug_info(src);

    // We expect 3 brillig functions:
    //   - Quotient (shared between both divisions)
    //   - Inversion, caused by division-by-zero check (shared between both divisions)
    //   - Custom brillig function `foo`
    assert_eq!(
        program.unconstrained_functions.len(),
        3,
        "Should only have generated three Brillig functions"
    );
    // We have one normal Brillig functions that was called twice.
    // We should have a single locations map for each function's debug metadata.
    assert_eq!(debug[0].brillig_locations.len(), 1);
    assert!(debug[0].brillig_locations.contains_key(&BrilligFunctionId(0)));

    // Brillig stdlib IDs are expected to always come at the end of the Brillig functions list.
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w12
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    BLACKBOX::RANGE [(w0, 32)] []
    BLACKBOX::RANGE [(w1, 32)] []
    BLACKBOX::RANGE [(w2, 32)] []
    BRILLIG CALL func 1: inputs: [EXPR [ (1, w1) 0 ]], outputs: [w3]
    EXPR [ (1, w1, w3) -1 ]
    BRILLIG CALL func 2: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w4, w5]
    BLACKBOX::RANGE [(w4, 32)] []
    BLACKBOX::RANGE [(w5, 32)] []
    EXPR [ (1, w1) (-1, w5) (-1, w6) -1 ]
    BLACKBOX::RANGE [(w6, 32)] []
    EXPR [ (-1, w1, w4) (1, w0) (-1, w5) 0 ]
    EXPR [ (-1, w2) (1, w4) 0 ]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w7]
    BLACKBOX::RANGE [(w7, 32)] []
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w8]
    BLACKBOX::RANGE [(w8, 32)] []
    BRILLIG CALL func 1: inputs: [EXPR [ (1, w2) 0 ]], outputs: [w9]
    EXPR [ (1, w2, w9) -1 ]
    BRILLIG CALL func 2: inputs: [EXPR [ (1, w1) 0 ], EXPR [ (1, w2) 0 ]], outputs: [w10, w11]
    BLACKBOX::RANGE [(w11, 32)] []
    EXPR [ (1, w2) (-1, w11) (-1, w12) -1 ]
    BLACKBOX::RANGE [(w12, 32)] []
    EXPR [ (-1, w2, w10) (1, w1) (-1, w11) 0 ]
    EXPR [ (1, w10) -1 ]
    
    unconstrained func 0
    [Const { destination: Direct(2), bit_size: Integer(U32), value: 1 }, Const { destination: Direct(1), bit_size: Integer(U32), value: 32839 }, Const { destination: Direct(0), bit_size: Integer(U32), value: 3 }, Const { destination: Relative(3), bit_size: Integer(U32), value: 2 }, Const { destination: Relative(4), bit_size: Integer(U32), value: 0 }, CalldataCopy { destination_address: Direct(32836), size_address: Relative(3), offset_address: Relative(4) }, Cast { destination: Direct(32836), source: Direct(32836), bit_size: Integer(U32) }, Cast { destination: Direct(32837), source: Direct(32837), bit_size: Integer(U32) }, Mov { destination: Relative(1), source: Direct(32836) }, Mov { destination: Relative(2), source: Direct(32837) }, Call { location: 16 }, Call { location: 17 }, Mov { destination: Direct(32838), source: Relative(1) }, Const { destination: Relative(2), bit_size: Integer(U32), value: 32838 }, Const { destination: Relative(3), bit_size: Integer(U32), value: 1 }, Stop { return_data: HeapVector { pointer: Relative(2), size: Relative(3) } }, Return, Call { location: 25 }, BinaryIntOp { destination: Relative(3), op: Equals, bit_size: U32, lhs: Relative(1), rhs: Relative(2) }, Const { destination: Relative(2), bit_size: Integer(U1), value: 0 }, BinaryIntOp { destination: Relative(4), op: Equals, bit_size: U1, lhs: Relative(3), rhs: Relative(2) }, JumpIf { condition: Relative(4), location: 24 }, Const { destination: Relative(5), bit_size: Integer(U32), value: 0 }, Trap { revert_data: HeapVector { pointer: Direct(1), size: Relative(5) } }, Return, Const { destination: Direct(32772), bit_size: Integer(U32), value: 30720 }, BinaryIntOp { destination: Direct(32771), op: LessThan, bit_size: U32, lhs: Direct(0), rhs: Direct(32772) }, JumpIf { condition: Direct(32771), location: 30 }, IndirectConst { destination_pointer: Direct(1), bit_size: Integer(U64), value: 15764276373176857197 }, Trap { revert_data: HeapVector { pointer: Direct(1), size: Direct(2) } }, Return]
    unconstrained func 1
    [Const { destination: Direct(21), bit_size: Integer(U32), value: 1 }, Const { destination: Direct(20), bit_size: Integer(U32), value: 0 }, CalldataCopy { destination_address: Direct(0), size_address: Direct(21), offset_address: Direct(20) }, Const { destination: Direct(2), bit_size: Field, value: 0 }, BinaryFieldOp { destination: Direct(3), op: Equals, lhs: Direct(0), rhs: Direct(2) }, JumpIf { condition: Direct(3), location: 8 }, Const { destination: Direct(1), bit_size: Field, value: 1 }, BinaryFieldOp { destination: Direct(0), op: Div, lhs: Direct(1), rhs: Direct(0) }, Stop { return_data: HeapVector { pointer: Direct(20), size: Direct(21) } }]
    unconstrained func 2
    [Const { destination: Direct(10), bit_size: Integer(U32), value: 2 }, Const { destination: Direct(11), bit_size: Integer(U32), value: 0 }, CalldataCopy { destination_address: Direct(0), size_address: Direct(10), offset_address: Direct(11) }, BinaryFieldOp { destination: Direct(2), op: IntegerDiv, lhs: Direct(0), rhs: Direct(1) }, BinaryFieldOp { destination: Direct(1), op: Mul, lhs: Direct(2), rhs: Direct(1) }, BinaryFieldOp { destination: Direct(1), op: Sub, lhs: Direct(0), rhs: Direct(1) }, Mov { destination: Direct(0), source: Direct(2) }, Stop { return_data: HeapVector { pointer: Direct(11), size: Direct(10) } }]
    ");
}

// Test that given both normal Brillig calls, Brillig stdlib calls, and non-inlined ACIR calls, that we accurately generate ACIR.
#[test]
fn brillig_stdlib_calls_with_multiple_acir_calls() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u32, v1: u32, v2: u32):
        v5 = div v0, v1
        constrain v5 == v2
        v6 = call f1(v0, v1) -> u32
        v7 = call f1(v0, v1) -> u32
        v8 = call f2(v0, v1) -> u32
        v9 = div v1, v2
        constrain v9 == u32 1
        return
    }
    brillig(inline) fn foo f1 {
      b0(v0: u32, v1: u32):
        v2 = eq v0, v1
        constrain v2 == u1 0
        return v0
    }
    acir(fold) fn foo f2 {
      b0(v0: u32, v1: u32):
        v2 = eq v0, v1
        constrain v2 == u1 0
        return v0
    }
    ";
    let (program, debug) = ssa_to_acir_program_with_debug_info(src);

    // We expect 3 brillig functions:
    //   - Quotient (shared between both divisions)
    //   - Inversion, caused by division-by-zero check (shared between both divisions)
    //   - Custom brillig function `foo`
    assert_eq!(
        program.unconstrained_functions.len(),
        3,
        "Should only have generated three Brillig functions"
    );

    let main_debug = &debug[0];
    assert_eq!(main_debug.brillig_locations.len(), 1);
    assert!(main_debug.brillig_locations.contains_key(&BrilligFunctionId(0)));

    let foo_debug = &debug[1];
    assert_eq!(foo_debug.brillig_locations.len(), 0);


    // TODO(https://github.com/noir-lang/noir/issues/9877): Update this snapshot once the linked issue is fixed.
    // `CALL func 2` in `func 0` is incorrect.
    assert_circuit_snapshot!(program, @r"
    func 0
    current witness: w13
    private parameters: [w0, w1, w2]
    public parameters: []
    return values: []
    BLACKBOX::RANGE [(w0, 32)] []
    BLACKBOX::RANGE [(w1, 32)] []
    BLACKBOX::RANGE [(w2, 32)] []
    BRILLIG CALL func 1: inputs: [EXPR [ (1, w1) 0 ]], outputs: [w3]
    EXPR [ (1, w1, w3) -1 ]
    BRILLIG CALL func 2: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w4, w5]
    BLACKBOX::RANGE [(w4, 32)] []
    BLACKBOX::RANGE [(w5, 32)] []
    EXPR [ (1, w1) (-1, w5) (-1, w6) -1 ]
    BLACKBOX::RANGE [(w6, 32)] []
    EXPR [ (-1, w1, w4) (1, w0) (-1, w5) 0 ]
    EXPR [ (-1, w2) (1, w4) 0 ]
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w7]
    BLACKBOX::RANGE [(w7, 32)] []
    BRILLIG CALL func 0: inputs: [EXPR [ (1, w0) 0 ], EXPR [ (1, w1) 0 ]], outputs: [w8]
    BLACKBOX::RANGE [(w8, 32)] []
    CALL func 2: PREDICATE: EXPR [ 1 ]
    inputs: [w0, w1], outputs: [w9]
    BRILLIG CALL func 1: inputs: [EXPR [ (1, w2) 0 ]], outputs: [w10]
    EXPR [ (1, w2, w10) -1 ]
    BRILLIG CALL func 2: inputs: [EXPR [ (1, w1) 0 ], EXPR [ (1, w2) 0 ]], outputs: [w11, w12]
    BLACKBOX::RANGE [(w12, 32)] []
    EXPR [ (1, w2) (-1, w12) (-1, w13) -1 ]
    BLACKBOX::RANGE [(w13, 32)] []
    EXPR [ (-1, w2, w11) (1, w1) (-1, w12) 0 ]
    EXPR [ (1, w11) -1 ]
    
    func 1
    current witness: w5
    private parameters: [w0, w1]
    public parameters: []
    return values: [w2]
    BLACKBOX::RANGE [(w0, 32)] []
    BLACKBOX::RANGE [(w1, 32)] []
    EXPR [ (1, w0) (-1, w1) (-1, w3) 0 ]
    BRILLIG CALL func 1: inputs: [EXPR [ (1, w3) 0 ]], outputs: [w4]
    EXPR [ (1, w3, w4) (1, w5) -1 ]
    EXPR [ (1, w3, w5) 0 ]
    EXPR [ (1, w5) 0 ]
    EXPR [ (-1, w0) (1, w2) 0 ]
    
    unconstrained func 0
    [Const { destination: Direct(2), bit_size: Integer(U32), value: 1 }, Const { destination: Direct(1), bit_size: Integer(U32), value: 32839 }, Const { destination: Direct(0), bit_size: Integer(U32), value: 3 }, Const { destination: Relative(3), bit_size: Integer(U32), value: 2 }, Const { destination: Relative(4), bit_size: Integer(U32), value: 0 }, CalldataCopy { destination_address: Direct(32836), size_address: Relative(3), offset_address: Relative(4) }, Cast { destination: Direct(32836), source: Direct(32836), bit_size: Integer(U32) }, Cast { destination: Direct(32837), source: Direct(32837), bit_size: Integer(U32) }, Mov { destination: Relative(1), source: Direct(32836) }, Mov { destination: Relative(2), source: Direct(32837) }, Call { location: 16 }, Call { location: 17 }, Mov { destination: Direct(32838), source: Relative(1) }, Const { destination: Relative(2), bit_size: Integer(U32), value: 32838 }, Const { destination: Relative(3), bit_size: Integer(U32), value: 1 }, Stop { return_data: HeapVector { pointer: Relative(2), size: Relative(3) } }, Return, Call { location: 25 }, BinaryIntOp { destination: Relative(3), op: Equals, bit_size: U32, lhs: Relative(1), rhs: Relative(2) }, Const { destination: Relative(2), bit_size: Integer(U1), value: 0 }, BinaryIntOp { destination: Relative(4), op: Equals, bit_size: U1, lhs: Relative(3), rhs: Relative(2) }, JumpIf { condition: Relative(4), location: 24 }, Const { destination: Relative(5), bit_size: Integer(U32), value: 0 }, Trap { revert_data: HeapVector { pointer: Direct(1), size: Relative(5) } }, Return, Const { destination: Direct(32772), bit_size: Integer(U32), value: 30720 }, BinaryIntOp { destination: Direct(32771), op: LessThan, bit_size: U32, lhs: Direct(0), rhs: Direct(32772) }, JumpIf { condition: Direct(32771), location: 30 }, IndirectConst { destination_pointer: Direct(1), bit_size: Integer(U64), value: 15764276373176857197 }, Trap { revert_data: HeapVector { pointer: Direct(1), size: Direct(2) } }, Return]
    unconstrained func 1
    [Const { destination: Direct(21), bit_size: Integer(U32), value: 1 }, Const { destination: Direct(20), bit_size: Integer(U32), value: 0 }, CalldataCopy { destination_address: Direct(0), size_address: Direct(21), offset_address: Direct(20) }, Const { destination: Direct(2), bit_size: Field, value: 0 }, BinaryFieldOp { destination: Direct(3), op: Equals, lhs: Direct(0), rhs: Direct(2) }, JumpIf { condition: Direct(3), location: 8 }, Const { destination: Direct(1), bit_size: Field, value: 1 }, BinaryFieldOp { destination: Direct(0), op: Div, lhs: Direct(1), rhs: Direct(0) }, Stop { return_data: HeapVector { pointer: Direct(20), size: Direct(21) } }]
    unconstrained func 2
    [Const { destination: Direct(10), bit_size: Integer(U32), value: 2 }, Const { destination: Direct(11), bit_size: Integer(U32), value: 0 }, CalldataCopy { destination_address: Direct(0), size_address: Direct(10), offset_address: Direct(11) }, BinaryFieldOp { destination: Direct(2), op: IntegerDiv, lhs: Direct(0), rhs: Direct(1) }, BinaryFieldOp { destination: Direct(1), op: Mul, lhs: Direct(2), rhs: Direct(1) }, BinaryFieldOp { destination: Direct(1), op: Sub, lhs: Direct(0), rhs: Direct(1) }, Mov { destination: Direct(0), source: Direct(2) }, Stop { return_data: HeapVector { pointer: Direct(11), size: Direct(10) } }]
    ");
}
