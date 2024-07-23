use std::collections::BTreeMap;

use acvm::acir::brillig::{BitSize, IntegerBitSize, Opcode as BrilligOpcode};

use acvm::acir::circuit::OpcodeLocation;
use acvm::brillig_vm::brillig::{
    BinaryFieldOp, BinaryIntOp, BlackBoxOp, HeapArray, HeapVector, MemoryAddress, ValueOrArray,
};
use acvm::{AcirField, FieldElement};
use noirc_errors::debug_info::DebugInfo;
use noirc_errors::Location;

use crate::instructions::{
    AvmInstruction, AvmOperand, AvmTypeTag, ALL_DIRECT, FIRST_OPERAND_INDIRECT,
    SECOND_OPERAND_INDIRECT, ZEROTH_OPERAND_INDIRECT,
};
use crate::opcodes::AvmOpcode;
use crate::utils::{dbg_print_avm_program, dbg_print_brillig_program};

/// Transpile a Brillig program to AVM bytecode
pub fn brillig_to_avm(
    brillig_bytecode: &[BrilligOpcode<FieldElement>],
    brillig_pcs_to_avm_pcs: &[usize],
) -> Vec<u8> {
    dbg_print_brillig_program(brillig_bytecode);

    let mut avm_instrs: Vec<AvmInstruction> = Vec::new();

    // Transpile a Brillig instruction to one or more AVM instructions
    for brillig_instr in brillig_bytecode {
        match brillig_instr {
            BrilligOpcode::BinaryFieldOp { destination, op, lhs, rhs } => {
                let avm_opcode = match op {
                    BinaryFieldOp::Add => AvmOpcode::ADD,
                    BinaryFieldOp::Sub => AvmOpcode::SUB,
                    BinaryFieldOp::Mul => AvmOpcode::MUL,
                    BinaryFieldOp::Div => AvmOpcode::FDIV,
                    BinaryFieldOp::IntegerDiv => AvmOpcode::DIV,
                    BinaryFieldOp::Equals => AvmOpcode::EQ,
                    BinaryFieldOp::LessThan => AvmOpcode::LT,
                    BinaryFieldOp::LessThanEquals => AvmOpcode::LTE,
                };
                avm_instrs.push(AvmInstruction {
                    opcode: avm_opcode,
                    indirect: Some(ALL_DIRECT),
                    tag: if avm_opcode == AvmOpcode::FDIV { None } else { Some(AvmTypeTag::FIELD) },
                    operands: vec![
                        AvmOperand::U32 { value: lhs.to_usize() as u32 },
                        AvmOperand::U32 { value: rhs.to_usize() as u32 },
                        AvmOperand::U32 { value: destination.to_usize() as u32 },
                    ],
                });
            }
            BrilligOpcode::BinaryIntOp { destination, op, bit_size, lhs, rhs } => {
                assert!(
                    is_integral_bit_size(*bit_size),
                    "BinaryIntOp bit size should be integral: {:?}",
                    brillig_instr
                );
                let avm_opcode = match op {
                    BinaryIntOp::Add => AvmOpcode::ADD,
                    BinaryIntOp::Sub => AvmOpcode::SUB,
                    BinaryIntOp::Mul => AvmOpcode::MUL,
                    BinaryIntOp::Div => AvmOpcode::DIV,
                    BinaryIntOp::Equals => AvmOpcode::EQ,
                    BinaryIntOp::LessThan => AvmOpcode::LT,
                    BinaryIntOp::LessThanEquals => AvmOpcode::LTE,
                    BinaryIntOp::And => AvmOpcode::AND,
                    BinaryIntOp::Or => AvmOpcode::OR,
                    BinaryIntOp::Xor => AvmOpcode::XOR,
                    BinaryIntOp::Shl => AvmOpcode::SHL,
                    BinaryIntOp::Shr => AvmOpcode::SHR,
                };
                avm_instrs.push(AvmInstruction {
                    opcode: avm_opcode,
                    indirect: Some(ALL_DIRECT),
                    tag: Some(tag_from_bit_size(BitSize::Integer(*bit_size))),
                    operands: vec![
                        AvmOperand::U32 { value: lhs.to_usize() as u32 },
                        AvmOperand::U32 { value: rhs.to_usize() as u32 },
                        AvmOperand::U32 { value: destination.to_usize() as u32 },
                    ],
                });
            }
            BrilligOpcode::CalldataCopy { destination_address, size, offset } => {
                avm_instrs.push(AvmInstruction {
                    opcode: AvmOpcode::CALLDATACOPY,
                    indirect: Some(ALL_DIRECT),
                    operands: vec![
                        AvmOperand::U32 {
                            value: *offset as u32, // cdOffset (calldata offset)
                        },
                        AvmOperand::U32 { value: *size as u32 },
                        AvmOperand::U32 {
                            value: destination_address.to_usize() as u32, // dstOffset
                        },
                    ],
                    ..Default::default()
                });
            }
            BrilligOpcode::Jump { location } => {
                let avm_loc = brillig_pcs_to_avm_pcs[*location];
                avm_instrs.push(AvmInstruction {
                    opcode: AvmOpcode::JUMP,
                    operands: vec![AvmOperand::U32 { value: avm_loc as u32 }],
                    ..Default::default()
                });
            }
            BrilligOpcode::JumpIf { condition, location } => {
                let avm_loc = brillig_pcs_to_avm_pcs[*location];
                avm_instrs.push(AvmInstruction {
                    opcode: AvmOpcode::JUMPI,
                    indirect: Some(ALL_DIRECT),
                    operands: vec![
                        AvmOperand::U32 { value: avm_loc as u32 },
                        AvmOperand::U32 { value: condition.to_usize() as u32 },
                    ],
                    ..Default::default()
                });
            }
            BrilligOpcode::Const { destination, value, bit_size } => {
                handle_const(&mut avm_instrs, destination, value, bit_size);
            }
            BrilligOpcode::Mov { destination, source } => {
                avm_instrs.push(generate_mov_instruction(
                    Some(ALL_DIRECT),
                    source.to_usize() as u32,
                    destination.to_usize() as u32,
                ));
            }
            BrilligOpcode::ConditionalMov { source_a, source_b, condition, destination } => {
                avm_instrs.push(AvmInstruction {
                    opcode: AvmOpcode::CMOV,
                    indirect: Some(ALL_DIRECT),
                    operands: vec![
                        AvmOperand::U32 { value: source_a.to_usize() as u32 },
                        AvmOperand::U32 { value: source_b.to_usize() as u32 },
                        AvmOperand::U32 { value: condition.to_usize() as u32 },
                        AvmOperand::U32 { value: destination.to_usize() as u32 },
                    ],
                    ..Default::default()
                });
            }
            BrilligOpcode::Load { destination, source_pointer } => {
                avm_instrs.push(generate_mov_instruction(
                    Some(ZEROTH_OPERAND_INDIRECT),
                    source_pointer.to_usize() as u32,
                    destination.to_usize() as u32,
                ));
            }
            BrilligOpcode::Store { destination_pointer, source } => {
                avm_instrs.push(generate_mov_instruction(
                    Some(FIRST_OPERAND_INDIRECT),
                    source.to_usize() as u32,
                    destination_pointer.to_usize() as u32,
                ));
            }
            BrilligOpcode::Call { location } => {
                let avm_loc = brillig_pcs_to_avm_pcs[*location];
                avm_instrs.push(AvmInstruction {
                    opcode: AvmOpcode::INTERNALCALL,
                    operands: vec![AvmOperand::U32 { value: avm_loc as u32 }],
                    ..Default::default()
                });
            }
            BrilligOpcode::Return {} => avm_instrs
                .push(AvmInstruction { opcode: AvmOpcode::INTERNALRETURN, ..Default::default() }),
            BrilligOpcode::Stop { return_data_offset, return_data_size } => {
                avm_instrs.push(AvmInstruction {
                    opcode: AvmOpcode::RETURN,
                    indirect: Some(ALL_DIRECT),
                    operands: vec![
                        AvmOperand::U32 { value: *return_data_offset as u32 },
                        AvmOperand::U32 { value: *return_data_size as u32 },
                    ],
                    ..Default::default()
                });
            }
            BrilligOpcode::Trap { revert_data } => {
                avm_instrs.push(AvmInstruction {
                    opcode: AvmOpcode::REVERT,
                    indirect: Some(ZEROTH_OPERAND_INDIRECT),
                    operands: vec![
                        AvmOperand::U32 { value: revert_data.pointer.0 as u32 },
                        AvmOperand::U32 { value: revert_data.size as u32 },
                    ],
                    ..Default::default()
                });
            }
            BrilligOpcode::Cast { destination, source, bit_size } => {
                handle_cast(&mut avm_instrs, source, destination, *bit_size);
            }
            BrilligOpcode::ForeignCall {
                function,
                destinations,
                inputs,
                destination_value_types: _,
                input_value_types: _,
            } => {
                handle_foreign_call(&mut avm_instrs, function, destinations, inputs);
            }
            BrilligOpcode::BlackBox(operation) => {
                handle_black_box_function(&mut avm_instrs, operation);
            }
            _ => panic!(
                "Transpiler doesn't know how to process {:?} brillig instruction",
                brillig_instr
            ),
        }
    }

    // TEMPORARY: Add a "magic number" instruction to the end of the program.
    // This makes it possible to know that the bytecode corresponds to the AVM.
    // We are adding a MOV instruction that moves a value to itself.
    // This should therefore not affect the program's execution.
    avm_instrs.push(AvmInstruction {
        opcode: AvmOpcode::MOV,
        indirect: Some(ALL_DIRECT),
        operands: vec![AvmOperand::U32 { value: 0x18ca }, AvmOperand::U32 { value: 0x18ca }],
        ..Default::default()
    });

    dbg_print_avm_program(&avm_instrs);

    // Constructing bytecode from instructions
    let mut bytecode = Vec::new();
    for instruction in avm_instrs {
        bytecode.extend_from_slice(&instruction.to_bytes());
    }
    bytecode
}

/// Handle brillig foreign calls
/// Examples:
/// - Tree access opcodes
/// - Hashing/gadget opcodes
/// - Environment getter opcodes
/// - TODO: support for avm external calls through this function
fn handle_foreign_call(
    avm_instrs: &mut Vec<AvmInstruction>,
    function: &str,
    destinations: &Vec<ValueOrArray>,
    inputs: &Vec<ValueOrArray>,
) {
    match function {
        "avmOpcodeCall" => handle_external_call(avm_instrs, destinations, inputs, AvmOpcode::CALL),
        "avmOpcodeStaticCall" => {
            handle_external_call(avm_instrs, destinations, inputs, AvmOpcode::STATICCALL);
        }
        "avmOpcodeEmitUnencryptedLog" => {
            handle_emit_unencrypted_log(avm_instrs, destinations, inputs);
        }
        "avmOpcodeNoteHashExists" => handle_note_hash_exists(avm_instrs, destinations, inputs),
        "avmOpcodeEmitNoteHash" | "avmOpcodeEmitNullifier" => handle_emit_note_hash_or_nullifier(
            function == "avmOpcodeEmitNullifier",
            avm_instrs,
            destinations,
            inputs,
        ),
        "avmOpcodeNullifierExists" => handle_nullifier_exists(avm_instrs, destinations, inputs),
        "avmOpcodeL1ToL2MsgExists" => handle_l1_to_l2_msg_exists(avm_instrs, destinations, inputs),
        "avmOpcodeSendL2ToL1Msg" => handle_send_l2_to_l1_msg(avm_instrs, destinations, inputs),
        "avmOpcodeGetContractInstance" => {
            handle_get_contract_instance(avm_instrs, destinations, inputs);
        }
        "avmOpcodeStorageRead" => handle_storage_read(avm_instrs, destinations, inputs),
        "avmOpcodeStorageWrite" => handle_storage_write(avm_instrs, destinations, inputs),
        "debugLog" => handle_debug_log(avm_instrs, destinations, inputs),
        // Getters.
        _ if inputs.is_empty() && destinations.len() == 1 => {
            handle_getter_instruction(avm_instrs, function, destinations, inputs);
        }
        // Anything else.
        _ => panic!("Transpiler doesn't know how to process ForeignCall function {}", function),
    }
}

/// Handle an AVM CALL
/// (an external 'call' brillig foreign call was encountered)
/// Adds the new instruction to the avm instructions list.
fn handle_external_call(
    avm_instrs: &mut Vec<AvmInstruction>,
    destinations: &Vec<ValueOrArray>,
    inputs: &Vec<ValueOrArray>,
    opcode: AvmOpcode,
) {
    if destinations.len() != 2 || inputs.len() != 5 {
        panic!(
            "Transpiler expects ForeignCall (Static)Call to have 2 destinations and 5 inputs, got {} and {}.
            Make sure your call instructions's input/return arrays have static length (`[Field; <size>]`)!",
            destinations.len(),
            inputs.len()
        );
    }
    let gas = inputs[0];
    let gas_offset = match gas {
        ValueOrArray::HeapArray(HeapArray { pointer, size }) => {
            assert!(size == 2, "Call instruction's gas input should be a HeapArray of size 2 (`[l2Gas, daGas]`)");
            pointer.0 as u32
        }
        ValueOrArray::HeapVector(_) => panic!("Call instruction's gas input must be a HeapArray, not a HeapVector. Make sure you are explicitly defining its size as 2 (`[l2Gas, daGas]`)!"),
        _ => panic!("Call instruction's gas input should be a HeapArray"),
    };
    let address_offset = match &inputs[1] {
        ValueOrArray::MemoryAddress(offset) => offset.to_usize() as u32,
        _ => panic!("Call instruction's target address input should be a basic MemoryAddress",),
    };
    // The args are a slice, and this is represented as a (Field, HeapVector).
    // The field is the length (memory address) and the HeapVector has the data and length again.
    // This is an ACIR internal representation detail that leaks to the SSA.
    // Observe that below, we use `inputs[3]` and therefore skip the length field.
    let args = inputs[3];
    let (args_offset, args_size_offset) = match args {
        ValueOrArray::HeapVector(HeapVector { pointer, size }) => (pointer.0 as u32, size.0 as u32),
        _ => panic!("Call instruction's args input should be a HeapVector input"),
    };
    let function_selector_offset = match &inputs[4] {
        ValueOrArray::MemoryAddress(offset) => offset.to_usize() as u32,
        _ => panic!("Call instruction's function selector input should be a basic MemoryAddress",),
    };

    let ret_offset_maybe = destinations[0];
    let (ret_offset, ret_size) = match ret_offset_maybe {
        ValueOrArray::HeapArray(HeapArray { pointer, size }) => (pointer.0 as u32, size as u32),
        ValueOrArray::HeapVector(_) => panic!("Call instruction's return data must be a HeapArray, not a HeapVector. Make sure you are explicitly defining its size (`let returnData: [Field; <size>] = ...`)!"),
        _ => panic!("Call instruction's returnData destination should be a HeapArray input"),
    };
    let success_offset = match &destinations[1] {
        ValueOrArray::MemoryAddress(offset) => offset.to_usize() as u32,
        _ => panic!("Call instruction's success destination should be a basic MemoryAddress",),
    };
    avm_instrs.push(AvmInstruction {
        opcode,
        // (left to right)
        //   * selector direct
        //   * success offset direct
        //   * (n/a) ret size is an immeadiate
        //   * ret offset INDIRECT
        //   * arg size offset direct
        //   * args offset INDIRECT
        //   * address offset direct
        //   * gas offset INDIRECT
        indirect: Some(0b00010101),
        operands: vec![
            AvmOperand::U32 { value: gas_offset },
            AvmOperand::U32 { value: address_offset },
            AvmOperand::U32 { value: args_offset },
            AvmOperand::U32 { value: args_size_offset },
            AvmOperand::U32 { value: ret_offset },
            AvmOperand::U32 { value: ret_size },
            AvmOperand::U32 { value: success_offset },
            AvmOperand::U32 { value: function_selector_offset },
        ],
        ..Default::default()
    });
}

fn handle_cast(
    avm_instrs: &mut Vec<AvmInstruction>,
    source: &MemoryAddress,
    destination: &MemoryAddress,
    bit_size: BitSize,
) {
    let source_offset = source.to_usize() as u32;
    let dest_offset = destination.to_usize() as u32;

    if bit_size == BitSize::Integer(IntegerBitSize::U1) {
        assert!(
            matches!(tag_from_bit_size(bit_size), AvmTypeTag::UINT8),
            "If u1 doesn't map to u8 anymore, change this code!"
        );
        avm_instrs.extend([
            // We cast to Field to be able to use toradix.
            generate_cast_instruction(source_offset, dest_offset, AvmTypeTag::FIELD),
            // Toradix with radix 2 and 1 limb is the same as modulo 2.
            // We need to insert an instruction explicitly because we want to fine-tune 'indirect'.
            AvmInstruction {
                opcode: AvmOpcode::TORADIXLE,
                indirect: Some(ALL_DIRECT),
                tag: None,
                operands: vec![
                    AvmOperand::U32 { value: dest_offset },
                    AvmOperand::U32 { value: dest_offset },
                    AvmOperand::U32 { value: /*radix=*/ 2},
                    AvmOperand::U32 { value: /*limbs=*/ 1},
                ],
            },
            // Then we cast back to u8 (which is what we use for u1).
            generate_cast_instruction(dest_offset, dest_offset, AvmTypeTag::UINT8),
        ]);
    } else {
        let tag = tag_from_bit_size(bit_size);
        avm_instrs.push(generate_cast_instruction(source_offset, dest_offset, tag));
    }
}

/// Handle an AVM NOTEHASHEXISTS instruction
/// Adds the new instruction to the avm instructions list.
fn handle_note_hash_exists(
    avm_instrs: &mut Vec<AvmInstruction>,
    destinations: &Vec<ValueOrArray>,
    inputs: &Vec<ValueOrArray>,
) {
    let (note_hash_offset_operand, leaf_index_offset_operand) = match &inputs[..] {
        [
            ValueOrArray::MemoryAddress(nh_offset),
            ValueOrArray::MemoryAddress(li_offset)
        ] => (nh_offset.to_usize() as u32, li_offset.to_usize() as u32),
        _ => panic!(
            "Transpiler expects ForeignCall::NOTEHASHEXISTS to have 2 inputs of type MemoryAddress, got {:?}", inputs
        ),
    };
    let exists_offset_operand = match &destinations[..] {
        [ValueOrArray::MemoryAddress(offset)] => offset.to_usize() as u32,
        _ => panic!(
            "Transpiler expects ForeignCall::NOTEHASHEXISTS to have 1 output of type MemoryAddress, got {:?}", destinations
        ),
    };
    avm_instrs.push(AvmInstruction {
        opcode: AvmOpcode::NOTEHASHEXISTS,
        indirect: Some(ALL_DIRECT),
        operands: vec![
            AvmOperand::U32 { value: note_hash_offset_operand },
            AvmOperand::U32 { value: leaf_index_offset_operand },
            AvmOperand::U32 { value: exists_offset_operand },
        ],
        ..Default::default()
    });
}

fn handle_emit_unencrypted_log(
    avm_instrs: &mut Vec<AvmInstruction>,
    destinations: &Vec<ValueOrArray>,
    inputs: &Vec<ValueOrArray>,
) {
    if !destinations.is_empty() || inputs.len() != 2 {
        panic!(
            "Transpiler expects ForeignCall::EMITUNENCRYPTEDLOG to have 0 destinations and 2 inputs, got {} and {}",
            destinations.len(),
            inputs.len()
        );
    }

    // The fields are a slice, and this is represented as a (length: Field, slice: HeapVector).
    // The length field is redundant and we skipt it.
    let (message_offset, message_size_offset) = match &inputs[1] {
        ValueOrArray::HeapVector(vec) => (vec.pointer.to_usize() as u32, vec.size.0 as u32),
        _ => panic!("Unexpected inputs for ForeignCall::EMITUNENCRYPTEDLOG: {:?}", inputs),
    };
    avm_instrs.push(AvmInstruction {
        opcode: AvmOpcode::EMITUNENCRYPTEDLOG,
        // The message array from Brillig is indirect.
        indirect: Some(ZEROTH_OPERAND_INDIRECT),
        operands: vec![
            AvmOperand::U32 { value: message_offset },
            AvmOperand::U32 { value: message_size_offset },
        ],
        ..Default::default()
    });
}

/// Handle an AVM EMITNOTEHASH or EMITNULLIFIER instruction
/// (an emitNoteHash or emitNullifier brillig foreign call was encountered)
/// Adds the new instruction to the avm instructions list.
fn handle_emit_note_hash_or_nullifier(
    is_nullifier: bool, // false for note hash, true for nullifier
    avm_instrs: &mut Vec<AvmInstruction>,
    destinations: &Vec<ValueOrArray>,
    inputs: &Vec<ValueOrArray>,
) {
    let function_name = if is_nullifier { "EMITNULLIFIER" } else { "EMITNOTEHASH" };

    if !destinations.is_empty() || inputs.len() != 1 {
        panic!(
            "Transpiler expects ForeignCall::{} to have 0 destinations and 1 input, got {} and {}",
            function_name,
            destinations.len(),
            inputs.len()
        );
    }
    let offset_operand = match &inputs[0] {
        ValueOrArray::MemoryAddress(offset) => offset.to_usize() as u32,
        _ => panic!(
            "Transpiler does not know how to handle ForeignCall::{} with HeapArray/Vector inputs",
            function_name
        ),
    };
    avm_instrs.push(AvmInstruction {
        opcode: if is_nullifier { AvmOpcode::EMITNULLIFIER } else { AvmOpcode::EMITNOTEHASH },
        indirect: Some(ALL_DIRECT),
        operands: vec![AvmOperand::U32 { value: offset_operand }],
        ..Default::default()
    });
}

/// Handle an AVM NULLIFIEREXISTS instruction
/// (a nullifierExists brillig foreign call was encountered)
/// Adds the new instruction to the avm instructions list.
fn handle_nullifier_exists(
    avm_instrs: &mut Vec<AvmInstruction>,
    destinations: &Vec<ValueOrArray>,
    inputs: &Vec<ValueOrArray>,
) {
    if destinations.len() != 1 || inputs.len() != 2 {
        panic!("Transpiler expects ForeignCall::CHECKNULLIFIEREXISTS to have 1 destinations and 2 inputs, got {} and {}", destinations.len(), inputs.len());
    }
    let nullifier_offset_operand = match &inputs[0] {
        ValueOrArray::MemoryAddress(offset) => offset.to_usize() as u32,
        _ => panic!("Transpiler does not know how to handle ForeignCall::EMITNOTEHASH with HeapArray/Vector inputs"),
    };
    let address_offset_operand = match &inputs[1] {
        ValueOrArray::MemoryAddress(offset) => offset.to_usize() as u32,
        _ => panic!("Transpiler does not know how to handle ForeignCall::EMITNOTEHASH with HeapArray/Vector inputs"),
    };
    let exists_offset_operand = match &destinations[0] {
        ValueOrArray::MemoryAddress(offset) => offset.to_usize() as u32,
        _ => panic!("Transpiler does not know how to handle ForeignCall::EMITNOTEHASH with HeapArray/Vector inputs"),
    };
    avm_instrs.push(AvmInstruction {
        opcode: AvmOpcode::NULLIFIEREXISTS,
        indirect: Some(ALL_DIRECT),
        operands: vec![
            AvmOperand::U32 { value: nullifier_offset_operand },
            AvmOperand::U32 { value: address_offset_operand },
            AvmOperand::U32 { value: exists_offset_operand },
        ],
        ..Default::default()
    });
}

/// Handle an AVM L1TOL2MSGEXISTS instruction
/// (a l1ToL2MsgExists brillig foreign call was encountered)
/// Adds the new instruction to the avm instructions list.
fn handle_l1_to_l2_msg_exists(
    avm_instrs: &mut Vec<AvmInstruction>,
    destinations: &Vec<ValueOrArray>,
    inputs: &Vec<ValueOrArray>,
) {
    if destinations.len() != 1 || inputs.len() != 2 {
        panic!(
            "Transpiler expects ForeignCall::L1TOL2MSGEXISTS to have 1 destinations and 2 input, got {} and {}",
            destinations.len(),
            inputs.len()
        );
    }
    let msg_hash_offset_operand = match &inputs[0] {
        ValueOrArray::MemoryAddress(offset) => offset.to_usize() as u32,
        _ => panic!(
            "Transpiler does not know how to handle ForeignCall::L1TOL2MSGEXISTS with HeapArray/Vector inputs",
        ),
    };
    let msg_leaf_index_offset_operand = match &inputs[1] {
        ValueOrArray::MemoryAddress(offset) => offset.to_usize() as u32,
        _ => panic!(
            "Transpiler does not know how to handle ForeignCall::L1TOL2MSGEXISTS with HeapArray/Vector inputs",
        ),
    };
    let exists_offset_operand = match &destinations[0] {
        ValueOrArray::MemoryAddress(offset) => offset.to_usize() as u32,
        _ => panic!(
            "Transpiler does not know how to handle ForeignCall::L1TOL2MSGEXISTS with HeapArray/Vector inputs",
        ),
    };
    avm_instrs.push(AvmInstruction {
        opcode: AvmOpcode::L1TOL2MSGEXISTS,
        indirect: Some(ALL_DIRECT),
        operands: vec![
            AvmOperand::U32 { value: msg_hash_offset_operand },
            AvmOperand::U32 { value: msg_leaf_index_offset_operand },
            AvmOperand::U32 { value: exists_offset_operand },
        ],
        ..Default::default()
    });
}

/// Handle an AVM SENDL2TOL1MSG
/// (a sendL2ToL1Msg brillig foreign call was encountered)
/// Adds the new instruction to the avm instructions list.
fn handle_send_l2_to_l1_msg(
    avm_instrs: &mut Vec<AvmInstruction>,
    destinations: &Vec<ValueOrArray>,
    inputs: &Vec<ValueOrArray>,
) {
    if !destinations.is_empty() || inputs.len() != 2 {
        panic!(
            "Transpiler expects ForeignCall::SENDL2TOL1MSG to have 0 destinations and 2 inputs, got {} and {}",
            destinations.len(),
            inputs.len()
        );
    }
    let recipient_offset_operand = match &inputs[0] {
        ValueOrArray::MemoryAddress(offset) => offset.to_usize() as u32,
        _ => panic!(
            "Transpiler does not know how to handle ForeignCall::SENDL2TOL1MSG with HeapArray/Vector inputs",
        ),
    };
    let content_offset_operand = match &inputs[1] {
        ValueOrArray::MemoryAddress(offset) => offset.to_usize() as u32,
        _ => panic!(
            "Transpiler does not know how to handle ForeignCall::SENDL2TOL1MSG with HeapArray/Vector inputs",
        ),
    };
    avm_instrs.push(AvmInstruction {
        opcode: AvmOpcode::SENDL2TOL1MSG,
        indirect: Some(ALL_DIRECT),
        operands: vec![
            AvmOperand::U32 { value: recipient_offset_operand },
            AvmOperand::U32 { value: content_offset_operand },
        ],
        ..Default::default()
    });
}

/// Getter Instructions are instructions that take NO inputs, and return information
/// from the current execution context.
///
/// This includes:
/// - Global variables
/// - Caller
/// - storage address
/// - ...
fn handle_getter_instruction(
    avm_instrs: &mut Vec<AvmInstruction>,
    function: &str,
    destinations: &Vec<ValueOrArray>,
    inputs: &Vec<ValueOrArray>,
) {
    // For the foreign calls we want to handle, we do not want inputs, as they are getters
    assert!(inputs.is_empty());
    assert!(destinations.len() == 1);

    let dest_offset_maybe = destinations[0];
    let dest_offset = match dest_offset_maybe {
        ValueOrArray::MemoryAddress(dest_offset) => dest_offset.0,
        _ => panic!("ForeignCall address destination should be a single value"),
    };

    let opcode = match function {
        "avmOpcodeAddress" => AvmOpcode::ADDRESS,
        "avmOpcodeStorageAddress" => AvmOpcode::STORAGEADDRESS,
        "avmOpcodeSender" => AvmOpcode::SENDER,
        "avmOpcodeFeePerL2Gas" => AvmOpcode::FEEPERL2GAS,
        "avmOpcodeFeePerDaGas" => AvmOpcode::FEEPERDAGAS,
        "avmOpcodeTransactionFee" => AvmOpcode::TRANSACTIONFEE,
        "avmOpcodeChainId" => AvmOpcode::CHAINID,
        "avmOpcodeVersion" => AvmOpcode::VERSION,
        "avmOpcodeBlockNumber" => AvmOpcode::BLOCKNUMBER,
        "avmOpcodeTimestamp" => AvmOpcode::TIMESTAMP,
        "avmOpcodeL2GasLeft" => AvmOpcode::L2GASLEFT,
        "avmOpcodeDaGasLeft" => AvmOpcode::DAGASLEFT,
        "avmOpcodeFunctionSelector" => AvmOpcode::FUNCTIONSELECTOR,
        // "callStackDepth" => AvmOpcode::CallStackDepth,
        _ => panic!("Transpiler doesn't know how to process ForeignCall function {:?}", function),
    };

    avm_instrs.push(AvmInstruction {
        opcode,
        indirect: Some(ALL_DIRECT),
        operands: vec![AvmOperand::U32 { value: dest_offset as u32 }],
        ..Default::default()
    });
}

/// Handles Brillig's CONST opcode.
fn handle_const(
    avm_instrs: &mut Vec<AvmInstruction>,
    destination: &MemoryAddress,
    value: &FieldElement,
    bit_size: &BitSize,
) {
    let tag = tag_from_bit_size(*bit_size);
    let dest = destination.to_usize() as u32;

    if !matches!(tag, AvmTypeTag::FIELD) {
        avm_instrs.push(generate_set_instruction(tag, dest, value.to_u128()));
    } else {
        // We can't fit a field in an instruction. This should've been handled in Brillig.
        let field = value;
        if field.num_bits() > 128 {
            panic!("SET: Field value doesn't fit in 128 bits, that's not supported!");
        }
        avm_instrs.extend([
            generate_set_instruction(AvmTypeTag::UINT128, dest, field.to_u128()),
            generate_cast_instruction(dest, dest, AvmTypeTag::FIELD),
        ]);
    }
}

/// Generates an AVM SET instruction.
fn generate_set_instruction(tag: AvmTypeTag, dest: u32, value: u128) -> AvmInstruction {
    AvmInstruction {
        opcode: AvmOpcode::SET,
        indirect: Some(ALL_DIRECT),
        tag: Some(tag),
        operands: vec![
            // const
            match tag {
                AvmTypeTag::UINT8 => AvmOperand::U8 { value: value as u8 },
                AvmTypeTag::UINT16 => AvmOperand::U16 { value: value as u16 },
                AvmTypeTag::UINT32 => AvmOperand::U32 { value: value as u32 },
                AvmTypeTag::UINT64 => AvmOperand::U64 { value: value as u64 },
                AvmTypeTag::UINT128 => AvmOperand::U128 { value },
                _ => panic!("Invalid type tag {:?} for set", tag),
            },
            // dest offset
            AvmOperand::U32 { value: dest },
        ],
    }
}

/// Generates an AVM CAST instruction.
fn generate_cast_instruction(source: u32, destination: u32, dst_tag: AvmTypeTag) -> AvmInstruction {
    AvmInstruction {
        opcode: AvmOpcode::CAST,
        indirect: Some(ALL_DIRECT),
        tag: Some(dst_tag),
        operands: vec![AvmOperand::U32 { value: source }, AvmOperand::U32 { value: destination }],
    }
}

/// Generates an AVM MOV instruction.
fn generate_mov_instruction(indirect: Option<u8>, source: u32, dest: u32) -> AvmInstruction {
    AvmInstruction {
        opcode: AvmOpcode::MOV,
        indirect,
        operands: vec![AvmOperand::U32 { value: source }, AvmOperand::U32 { value: dest }],
        ..Default::default()
    }
}

/// Black box functions, for the meantime only covers pedersen operations as the blackbox function api suits our current needs.
/// (array goes in -> field element comes out)
fn handle_black_box_function(avm_instrs: &mut Vec<AvmInstruction>, operation: &BlackBoxOp) {
    match operation {
        BlackBoxOp::Sha256 { message, output } => {
            let message_offset = message.pointer.0;
            let message_size_offset = message.size.0;
            let dest_offset = output.pointer.0;
            assert_eq!(output.size, 32, "SHA256 output size must be 32!");

            avm_instrs.push(AvmInstruction {
                opcode: AvmOpcode::SHA256,
                indirect: Some(ZEROTH_OPERAND_INDIRECT | FIRST_OPERAND_INDIRECT),
                operands: vec![
                    AvmOperand::U32 { value: dest_offset as u32 },
                    AvmOperand::U32 { value: message_offset as u32 },
                    AvmOperand::U32 { value: message_size_offset as u32 },
                ],
                ..Default::default()
            });
        }
        BlackBoxOp::PedersenHash { inputs, domain_separator, output } => {
            let message_offset = inputs.pointer.0;
            let message_size_offset = inputs.size.0;

            let index_offset = domain_separator.0;
            let dest_offset = output.0;

            avm_instrs.push(AvmInstruction {
                opcode: AvmOpcode::PEDERSEN,
                indirect: Some(SECOND_OPERAND_INDIRECT),
                operands: vec![
                    AvmOperand::U32 { value: index_offset as u32 },
                    AvmOperand::U32 { value: dest_offset as u32 },
                    AvmOperand::U32 { value: message_offset as u32 },
                    AvmOperand::U32 { value: message_size_offset as u32 },
                ],
                ..Default::default()
            });
        }
        BlackBoxOp::Poseidon2Permutation {
            message,
            output,
            len: _, // we don't use this.
        } => {
            // We'd love to validate the input size, but it's not known at compile time.
            assert_eq!(output.size, 4, "Poseidon2Permutation output size must be 4!");
            let input_state_offset = message.pointer.0;
            let output_state_offset = output.pointer.0;

            avm_instrs.push(AvmInstruction {
                opcode: AvmOpcode::POSEIDON2,
                indirect: Some(ZEROTH_OPERAND_INDIRECT | FIRST_OPERAND_INDIRECT),
                operands: vec![
                    AvmOperand::U32 { value: input_state_offset as u32 },
                    AvmOperand::U32 { value: output_state_offset as u32 },
                ],
                ..Default::default()
            });
        }
        BlackBoxOp::Keccak256 { message, output } => {
            let message_offset = message.pointer.0;
            let message_size_offset = message.size.0;
            let dest_offset = output.pointer.0;
            assert_eq!(output.size, 32, "Keccak256 output size must be 32!");

            avm_instrs.push(AvmInstruction {
                opcode: AvmOpcode::KECCAK,
                indirect: Some(ZEROTH_OPERAND_INDIRECT | FIRST_OPERAND_INDIRECT),
                operands: vec![
                    AvmOperand::U32 { value: dest_offset as u32 },
                    AvmOperand::U32 { value: message_offset as u32 },
                    AvmOperand::U32 { value: message_size_offset as u32 },
                ],
                ..Default::default()
            });
        }
        BlackBoxOp::Keccakf1600 { message, output } => {
            let message_offset = message.pointer.0;
            let message_size_offset = message.size.0;
            let dest_offset = output.pointer.0;
            assert_eq!(output.size, 25, "Keccakf1600 output size must be 25!");

            avm_instrs.push(AvmInstruction {
                opcode: AvmOpcode::KECCAKF1600,
                indirect: Some(ZEROTH_OPERAND_INDIRECT | FIRST_OPERAND_INDIRECT),
                operands: vec![
                    AvmOperand::U32 { value: dest_offset as u32 },
                    AvmOperand::U32 { value: message_offset as u32 },
                    AvmOperand::U32 { value: message_size_offset as u32 },
                ],
                ..Default::default()
            });
        }
        BlackBoxOp::ToRadix { input, radix, output } => {
            let num_limbs = output.size as u32;
            let input_offset = input.0 as u32;
            let output_offset = output.pointer.0 as u32;
            assert!(radix <= &256u32, "Radix must be less than or equal to 256");

            avm_instrs.push(AvmInstruction {
                opcode: AvmOpcode::TORADIXLE,
                indirect: Some(FIRST_OPERAND_INDIRECT),
                tag: None,
                operands: vec![
                    AvmOperand::U32 { value: input_offset },
                    AvmOperand::U32 { value: output_offset },
                    AvmOperand::U32 { value: *radix },
                    AvmOperand::U32 { value: num_limbs },
                ],
            });
        }
        // This will be changed to utilise relative memory offsets
        BlackBoxOp::EmbeddedCurveAdd {
            input1_x: p1_x_offset,
            input1_y: p1_y_offset,
            input1_infinite: p1_infinite_offset,
            input2_x: p2_x_offset,
            input2_y: p2_y_offset,
            input2_infinite: p2_infinite_offset,
            result,
        } => avm_instrs.push(AvmInstruction {
            opcode: AvmOpcode::ECADD,
            // The result (SIXTH operand) is indirect.
            indirect: Some(0b1000000),
            operands: vec![
                AvmOperand::U32 { value: p1_x_offset.0 as u32 },
                AvmOperand::U32 { value: p1_y_offset.0 as u32 },
                AvmOperand::U32 { value: p1_infinite_offset.0 as u32 },
                AvmOperand::U32 { value: p2_x_offset.0 as u32 },
                AvmOperand::U32 { value: p2_y_offset.0 as u32 },
                AvmOperand::U32 { value: p2_infinite_offset.0 as u32 },
                AvmOperand::U32 { value: result.pointer.0 as u32 },
            ],
            ..Default::default()
        }),
        // Temporary while we dont have efficient noir implementations
        BlackBoxOp::MultiScalarMul { points, scalars, outputs } => {
            // The length of the scalars vector is 2x the length of the points vector due to limb
            // decomposition
            let points_offset = points.pointer.0;
            let num_points = points.size.0;
            let scalars_offset = scalars.pointer.0;
            // Output array is fixed to 3
            assert_eq!(outputs.size, 3, "Output array size must be equal to 3");
            let outputs_offset = outputs.pointer.0;
            avm_instrs.push(AvmInstruction {
                opcode: AvmOpcode::MSM,
                indirect: Some(
                    ZEROTH_OPERAND_INDIRECT | FIRST_OPERAND_INDIRECT | SECOND_OPERAND_INDIRECT,
                ),
                operands: vec![
                    AvmOperand::U32 { value: points_offset as u32 },
                    AvmOperand::U32 { value: scalars_offset as u32 },
                    AvmOperand::U32 { value: outputs_offset as u32 },
                    AvmOperand::U32 { value: num_points as u32 },
                ],
                ..Default::default()
            });
        }
        _ => panic!("Transpiler doesn't know how to process {:?}", operation),
    }
}

fn handle_debug_log(
    avm_instrs: &mut Vec<AvmInstruction>,
    destinations: &Vec<ValueOrArray>,
    inputs: &Vec<ValueOrArray>,
) {
    if !destinations.is_empty() || inputs.len() != 3 {
        panic!(
            "Transpiler expects ForeignCall::DEBUGLOG to have 0 destinations and 3 inputs, got {} and {}",
            destinations.len(),
            inputs.len()
        );
    }
    let (message_offset, message_size) = match &inputs[0] {
        ValueOrArray::HeapArray(HeapArray { pointer, size }) => (pointer.0 as u32, *size as u32),
        _ => panic!("Message for ForeignCall::DEBUGLOG should be a HeapArray."),
    };
    // The fields are a slice, and this is represented as a (length: Field, slice: HeapVector).
    // The length field is redundant and we skipt it.
    let (fields_offset_ptr, fields_size_ptr) = match &inputs[2] {
        ValueOrArray::HeapVector(HeapVector { pointer, size }) => (pointer.0 as u32, size.0 as u32),
        _ => panic!("List of fields for ForeignCall::DEBUGLOG should be a HeapVector (slice)."),
    };
    avm_instrs.push(AvmInstruction {
        opcode: AvmOpcode::DEBUGLOG,
        // (left to right)
        //  * fields_size_ptr direct
        //  * fields_offset_ptr INDIRECT
        //  * (N/A) message_size is an immediate
        //  * message_offset direct
        indirect: Some(0b011),
        operands: vec![
            AvmOperand::U32 { value: message_offset },
            AvmOperand::U32 { value: message_size },
            // indirect
            AvmOperand::U32 { value: fields_offset_ptr },
            // indirect
            AvmOperand::U32 { value: fields_size_ptr },
        ],
        ..Default::default()
    });
}

/// Emit a storage write opcode
/// The current implementation writes an array of values into storage ( contiguous slots in memory )
fn handle_storage_write(
    avm_instrs: &mut Vec<AvmInstruction>,
    destinations: &Vec<ValueOrArray>,
    inputs: &Vec<ValueOrArray>,
) {
    assert!(inputs.len() == 2);
    assert!(destinations.len() == 0);

    let slot_offset_maybe = inputs[0];
    let slot_offset = match slot_offset_maybe {
        ValueOrArray::MemoryAddress(slot_offset) => slot_offset.0,
        _ => panic!("ForeignCall address destination should be a single value"),
    };

    let src_offset_maybe = inputs[1];
    let (src_offset, size) = match src_offset_maybe {
        ValueOrArray::HeapArray(HeapArray { pointer, size }) => (pointer.0, size),
        _ => panic!("Storage write address inputs should be an array of values"),
    };

    avm_instrs.push(AvmInstruction {
        opcode: AvmOpcode::SSTORE,
        indirect: Some(ZEROTH_OPERAND_INDIRECT),
        operands: vec![
            AvmOperand::U32 { value: src_offset as u32 },
            AvmOperand::U32 { value: size as u32 },
            AvmOperand::U32 { value: slot_offset as u32 },
        ],
        ..Default::default()
    });
}

/// Emit a GETCONTRACTINSTANCE opcode
fn handle_get_contract_instance(
    avm_instrs: &mut Vec<AvmInstruction>,
    destinations: &Vec<ValueOrArray>,
    inputs: &Vec<ValueOrArray>,
) {
    assert!(inputs.len() == 1);
    assert!(destinations.len() == 1);

    let address_offset_maybe = inputs[0];
    let address_offset = match address_offset_maybe {
        ValueOrArray::MemoryAddress(slot_offset) => slot_offset.0,
        _ => panic!("GETCONTRACTINSTANCE address should be a single value"),
    };

    let dest_offset_maybe = destinations[0];
    let dest_offset = match dest_offset_maybe {
        ValueOrArray::HeapArray(HeapArray { pointer, .. }) => pointer.0,
        _ => panic!("GETCONTRACTINSTANCE destination should be an array"),
    };

    avm_instrs.push(AvmInstruction {
        opcode: AvmOpcode::GETCONTRACTINSTANCE,
        indirect: Some(FIRST_OPERAND_INDIRECT),
        operands: vec![
            AvmOperand::U32 { value: address_offset as u32 },
            AvmOperand::U32 { value: dest_offset as u32 },
        ],
        ..Default::default()
    });
}

/// Emit a storage read opcode
/// The current implementation reads an array of values from storage ( contiguous slots in memory )
fn handle_storage_read(
    avm_instrs: &mut Vec<AvmInstruction>,
    destinations: &Vec<ValueOrArray>,
    inputs: &Vec<ValueOrArray>,
) {
    // For the foreign calls we want to handle, we do not want inputs, as they are getters
    assert!(inputs.len() == 2); // output, len. The latter is not used by the AVM, but required in the oracle call so that TXE knows how many slots to read.
    assert!(destinations.len() == 1); // return values

    let slot_offset_maybe = inputs[0];
    let slot_offset = match slot_offset_maybe {
        ValueOrArray::MemoryAddress(slot_offset) => slot_offset.0,
        _ => panic!("ForeignCall address destination should be a single value"),
    };

    let dest_offset_maybe = destinations[0];
    let (dest_offset, size) = match dest_offset_maybe {
        ValueOrArray::HeapArray(HeapArray { pointer, size }) => (pointer.0, size),
        _ => panic!("Storage write address inputs should be an array of values"),
    };

    avm_instrs.push(AvmInstruction {
        opcode: AvmOpcode::SLOAD,
        indirect: Some(FIRST_OPERAND_INDIRECT),
        operands: vec![
            AvmOperand::U32 { value: slot_offset as u32 },
            AvmOperand::U32 { value: size as u32 },
            AvmOperand::U32 { value: dest_offset as u32 },
        ],
        ..Default::default()
    });
}

/// Patch a Noir function's debug info with updated PCs since transpilation injects extra
/// instructions in some cases.
pub fn patch_debug_info_pcs(
    mut debug_infos: Vec<DebugInfo>,
    brillig_pcs_to_avm_pcs: &[usize],
) -> Vec<DebugInfo> {
    for patched_debug_info in debug_infos.iter_mut() {
        // create a new map with all of its keys (OpcodeLocations) patched
        let mut patched_locations: BTreeMap<OpcodeLocation, Vec<Location>> = BTreeMap::new();
        for (original_opcode_location, source_locations) in patched_debug_info.locations.iter() {
            match original_opcode_location {
                OpcodeLocation::Brillig { acir_index, brillig_index } => {
                    let avm_opcode_location = OpcodeLocation::Brillig {
                        acir_index: *acir_index,
                        // patch the PC
                        brillig_index: brillig_pcs_to_avm_pcs[*brillig_index],
                    };
                    patched_locations.insert(avm_opcode_location, source_locations.clone());
                }
                OpcodeLocation::Acir(_) => (),
            }
        }
        // patch debug_info entry
        patched_debug_info.locations = patched_locations;
    }
    debug_infos
}

/// Compute an array that maps each Brillig pc to an AVM pc.
/// This must be done before transpiling to properly transpile jump destinations.
/// This is necessary for two reasons:
/// 1. The transpiler injects `initial_offset` instructions at the beginning of the program.
/// 2. Some brillig instructions (_e.g._ Stop, or certain ForeignCalls) map to multiple AVM instructions
/// args:
///     initial_offset: how many AVM instructions were inserted at the start of the program
///     brillig: the Brillig program
/// returns: an array where each index is a Brillig pc,
///     and each value is the corresponding AVM pc.
pub fn map_brillig_pcs_to_avm_pcs(brillig_bytecode: &[BrilligOpcode<FieldElement>]) -> Vec<usize> {
    let mut pc_map = vec![0; brillig_bytecode.len()];

    pc_map[0] = 0; // first PC is always 0 as there are no instructions inserted by AVM at start
    for i in 0..brillig_bytecode.len() - 1 {
        let num_avm_instrs_for_this_brillig_instr = match &brillig_bytecode[i] {
            BrilligOpcode::Const { bit_size: BitSize::Field, .. } => 2,
            BrilligOpcode::Cast { bit_size: BitSize::Integer(IntegerBitSize::U1), .. } => 3,
            _ => 1,
        };
        // next Brillig pc will map to an AVM pc offset by the
        // number of AVM instructions generated for this Brillig one
        pc_map[i + 1] = pc_map[i] + num_avm_instrs_for_this_brillig_instr;
    }
    pc_map
}

fn is_integral_bit_size(bit_size: IntegerBitSize) -> bool {
    matches!(
        bit_size,
        IntegerBitSize::U1
            | IntegerBitSize::U8
            | IntegerBitSize::U16
            | IntegerBitSize::U32
            | IntegerBitSize::U64
            | IntegerBitSize::U128
    )
}

fn tag_from_bit_size(bit_size: BitSize) -> AvmTypeTag {
    match bit_size {
        BitSize::Integer(IntegerBitSize::U1) => AvmTypeTag::UINT8, // temp workaround
        BitSize::Integer(IntegerBitSize::U8) => AvmTypeTag::UINT8,
        BitSize::Integer(IntegerBitSize::U16) => AvmTypeTag::UINT16,
        BitSize::Integer(IntegerBitSize::U32) => AvmTypeTag::UINT32,
        BitSize::Integer(IntegerBitSize::U64) => AvmTypeTag::UINT64,
        BitSize::Integer(IntegerBitSize::U128) => AvmTypeTag::UINT128,
        BitSize::Field => AvmTypeTag::FIELD,
        _ => panic!("The AVM doesn't support integer bit size {:?}", bit_size),
    }
}
