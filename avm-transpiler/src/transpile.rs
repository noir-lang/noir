use acvm::acir::brillig::Opcode as BrilligOpcode;
use acvm::acir::circuit::brillig::Brillig;

use acvm::brillig_vm::brillig::{
    BinaryFieldOp, BinaryIntOp, BlackBoxOp, HeapArray, MemoryAddress, ValueOrArray,
};
use acvm::FieldElement;

use crate::instructions::{
    AvmInstruction, AvmOperand, AvmTypeTag, ALL_DIRECT, FIRST_OPERAND_INDIRECT,
    SECOND_OPERAND_INDIRECT, ZEROTH_OPERAND_INDIRECT,
};
use crate::opcodes::AvmOpcode;
use crate::utils::{dbg_print_avm_program, dbg_print_brillig_program};

/// Transpile a Brillig program to AVM bytecode
pub fn brillig_to_avm(brillig: &Brillig) -> Vec<u8> {
    dbg_print_brillig_program(brillig);

    let mut avm_instrs: Vec<AvmInstruction> = Vec::new();

    // Map Brillig pcs to AVM pcs
    // (some Brillig instructions map to >1 AVM instruction)
    let brillig_pcs_to_avm_pcs = map_brillig_pcs_to_avm_pcs(avm_instrs.len(), brillig);

    // Transpile a Brillig instruction to one or more AVM instructions
    for brillig_instr in &brillig.bytecode {
        match brillig_instr {
            BrilligOpcode::BinaryFieldOp {
                destination,
                op,
                lhs,
                rhs,
            } => {
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
                    tag: if avm_opcode == AvmOpcode::FDIV {
                        None
                    } else {
                        Some(AvmTypeTag::FIELD)
                    },
                    operands: vec![
                        AvmOperand::U32 {
                            value: lhs.to_usize() as u32,
                        },
                        AvmOperand::U32 {
                            value: rhs.to_usize() as u32,
                        },
                        AvmOperand::U32 {
                            value: destination.to_usize() as u32,
                        },
                    ],
                });
            }
            BrilligOpcode::BinaryIntOp {
                destination,
                op,
                bit_size,
                lhs,
                rhs,
            } => {
                assert!(is_integral_bit_size(*bit_size), "BinaryIntOp bit size should be integral: {:?}", brillig_instr);
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
                    _ => panic!(
                        "Transpiler doesn't know how to process {:?}", brillig_instr
                    ),
                };
                avm_instrs.push(AvmInstruction {
                    opcode: avm_opcode,
                    indirect: Some(ALL_DIRECT),
                    tag: Some(tag_from_bit_size(*bit_size)),
                    operands: vec![
                        AvmOperand::U32 {
                            value: lhs.to_usize() as u32,
                        },
                        AvmOperand::U32 {
                            value: rhs.to_usize() as u32,
                        },
                        AvmOperand::U32 {
                            value: destination.to_usize() as u32,
                        },
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
                        }, AvmOperand::U32 {
                            value: *size as u32,
                        }, AvmOperand::U32 {
                            value: destination_address.to_usize() as u32, // dstOffset
                        }],
                        ..Default::default()
                    });
            }
            BrilligOpcode::Jump { location } => {
                let avm_loc = brillig_pcs_to_avm_pcs[*location];
                avm_instrs.push(AvmInstruction {
                    opcode: AvmOpcode::JUMP,
                    operands: vec![AvmOperand::U32 {
                        value: avm_loc as u32,
                    }],
                    ..Default::default()
                });
            }
            BrilligOpcode::JumpIf {
                condition,
                location,
            } => {
                let avm_loc = brillig_pcs_to_avm_pcs[*location];
                avm_instrs.push(AvmInstruction {
                    opcode: AvmOpcode::JUMPI,
                    indirect: Some(ALL_DIRECT),
                    operands: vec![
                        AvmOperand::U32 {
                            value: avm_loc as u32,
                        },
                        AvmOperand::U32 {
                            value: condition.to_usize() as u32,
                        },
                    ],
                    ..Default::default()
                });
            }
            BrilligOpcode::Const { destination, value, bit_size } => {
                handle_const(&mut avm_instrs, destination, value, bit_size);
            }
            BrilligOpcode::Mov {
                destination,
                source,
            } => {
                avm_instrs.push(generate_mov_instruction(Some(ALL_DIRECT), source.to_usize() as u32, destination.to_usize() as u32));
            }
            BrilligOpcode::ConditionalMov {
                source_a,
                source_b,
                condition,
                destination,
            } => {
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
            BrilligOpcode::Load {
                destination,
                source_pointer,
            } => {
                avm_instrs.push(generate_mov_instruction(Some(ZEROTH_OPERAND_INDIRECT), source_pointer.to_usize() as u32, destination.to_usize() as u32));
            }
            BrilligOpcode::Store {
                destination_pointer,
                source,
            } => {
                avm_instrs.push(generate_mov_instruction(Some(FIRST_OPERAND_INDIRECT), source.to_usize() as u32, destination_pointer.to_usize() as u32));
            }
            BrilligOpcode::Call { location } => {
                let avm_loc = brillig_pcs_to_avm_pcs[*location];
                avm_instrs.push(AvmInstruction {
                    opcode: AvmOpcode::INTERNALCALL,
                    operands: vec![AvmOperand::U32 {
                        value: avm_loc as u32,
                    }],
                    ..Default::default()
                });
            }
            BrilligOpcode::Return {} => avm_instrs.push(AvmInstruction {
                opcode: AvmOpcode::INTERNALRETURN,
                ..Default::default()
            }),
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
            BrilligOpcode::Trap { /*return_data_offset, return_data_size*/ } => {
                // TODO(https://github.com/noir-lang/noir/issues/3113): Trap should support return data
                avm_instrs.push(AvmInstruction {
                    opcode: AvmOpcode::REVERT,
                    indirect: Some(ALL_DIRECT),
                    operands: vec![
                        //AvmOperand::U32 { value: *return_data_offset as u32},
                        //AvmOperand::U32 { value: *return_data_size as u32},
                        AvmOperand::U32 { value: 0 },
                        AvmOperand::U32 { value: 0 },
                    ],
                    ..Default::default()
                });
            },
            BrilligOpcode::Cast { destination, source, bit_size } => {
                avm_instrs.push(generate_cast_instruction(source.to_usize() as u32, destination.to_usize() as u32, tag_from_bit_size(*bit_size)));
            }
            BrilligOpcode::ForeignCall { function, destinations, inputs, destination_value_types:_, input_value_types:_ } => {
                handle_foreign_call(&mut avm_instrs, function, destinations, inputs);
            },
            BrilligOpcode::BlackBox(operation) => handle_black_box_function(&mut avm_instrs, operation),
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
        operands: vec![
            AvmOperand::U32 { value: 0x18ca },
            AvmOperand::U32 { value: 0x18ca },
        ],
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
            handle_external_call(avm_instrs, destinations, inputs, AvmOpcode::STATICCALL)
        }
        "amvOpcodeEmitUnencryptedLog" => {
            handle_emit_unencrypted_log(avm_instrs, destinations, inputs)
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
        "avmOpcodeKeccak256" | "avmOpcodeSha256" => {
            handle_2_field_hash_instruction(avm_instrs, function, destinations, inputs)
        }
        "avmOpcodePoseidon" => {
            handle_single_field_hash_instruction(avm_instrs, function, destinations, inputs)
        }
        "storageRead" => handle_storage_read(avm_instrs, destinations, inputs),
        "storageWrite" => handle_storage_write(avm_instrs, destinations, inputs),
        // Getters.
        _ if inputs.is_empty() && destinations.len() == 1 => {
            handle_getter_instruction(avm_instrs, function, destinations, inputs)
        }
        // Anything else.
        _ => panic!(
            "Transpiler doesn't know how to process ForeignCall function {}",
            function
        ),
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
    if destinations.len() != 2 || inputs.len() != 4 {
        panic!(
            "Transpiler expects ForeignCall (Static)Call to have 2 destinations and 4 inputs, got {} and {}.
            Make sure your call instructions's input/return arrays have static length (`[Field; <size>]`)!",
            destinations.len(),
            inputs.len()
        );
    }
    let gas_offset_maybe = inputs[0];
    let gas_offset = match gas_offset_maybe {
        ValueOrArray::HeapArray(HeapArray { pointer, size }) => {
            assert!(size == 3, "Call instruction's gas input should be a HeapArray of size 3 (`[l1Gas, l2Gas, daGas]`)");
            pointer.0 as u32
        }
        ValueOrArray::HeapVector(_) => panic!("Call instruction's gas input must be a HeapArray, not a HeapVector. Make sure you are explicitly defining its size as 3 (`[l1Gas, l2Gas, daGas]`)!"),
        _ => panic!("Call instruction's gas input should be a HeapArray"),
    };
    let address_offset = match &inputs[1] {
        ValueOrArray::MemoryAddress(offset) => offset.to_usize() as u32,
        _ => panic!("Call instruction's target address input should be a basic MemoryAddress",),
    };
    let args_offset_maybe = inputs[2];
    let (args_offset, args_size) = match args_offset_maybe {
        ValueOrArray::HeapArray(HeapArray { pointer, size }) => (pointer.0 as u32, size as u32),
        ValueOrArray::HeapVector(_) => panic!("Call instruction's args must be a HeapArray, not a HeapVector. Make sure you are explicitly defining its size (`[arg0, arg1, ... argN]`)!"),
        _ => panic!("Call instruction's args input should be a HeapArray input"),
    };
    let temporary_function_selector_offset = match &inputs[3] {
        ValueOrArray::MemoryAddress(offset) => offset.to_usize() as u32,
        _ => panic!(
            "Call instruction's temporary function selector input should be a basic MemoryAddress",
        ),
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
        opcode: opcode,
        indirect: Some(0b01101), // (left to right) selector direct, ret offset INDIRECT, args offset INDIRECT, address offset direct, gas offset INDIRECT
        operands: vec![
            AvmOperand::U32 { value: gas_offset },
            AvmOperand::U32 {
                value: address_offset,
            },
            AvmOperand::U32 { value: args_offset },
            AvmOperand::U32 { value: args_size },
            AvmOperand::U32 { value: ret_offset },
            AvmOperand::U32 { value: ret_size },
            AvmOperand::U32 {
                value: success_offset,
            },
            AvmOperand::U32 {
                value: temporary_function_selector_offset,
            },
        ],
        ..Default::default()
    });
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
            AvmOperand::U32 {
                value: note_hash_offset_operand,
            },
            AvmOperand::U32 {
                value: leaf_index_offset_operand,
            },
            AvmOperand::U32 {
                value: exists_offset_operand,
            },
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
            "Transpiler expects ForeignCall::EMITUNENCRYPTEDLOG to have 0 destinations and 3 inputs, got {} and {}",
            destinations.len(),
            inputs.len()
        );
    }
    let (event_offset, message_array) = match &inputs[..] {
        [ValueOrArray::MemoryAddress(offset), ValueOrArray::HeapArray(array)] => {
            (offset.to_usize() as u32, array)
        }
        _ => panic!(
            "Unexpected inputs for ForeignCall::EMITUNENCRYPTEDLOG: {:?}",
            inputs
        ),
    };
    avm_instrs.push(AvmInstruction {
        opcode: AvmOpcode::EMITUNENCRYPTEDLOG,
        // The message array from Brillig is indirect.
        indirect: Some(FIRST_OPERAND_INDIRECT),
        operands: vec![
            AvmOperand::U32 {
                value: event_offset,
            },
            AvmOperand::U32 {
                value: message_array.pointer.to_usize() as u32,
            },
            AvmOperand::U32 {
                value: message_array.size as u32,
            },
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
    let function_name = if is_nullifier {
        "EMITNULLIFIER"
    } else {
        "EMITNOTEHASH"
    };

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
        opcode: if is_nullifier {
            AvmOpcode::EMITNULLIFIER
        } else {
            AvmOpcode::EMITNOTEHASH
        },
        indirect: Some(ALL_DIRECT),
        operands: vec![AvmOperand::U32 {
            value: offset_operand,
        }],
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
    if destinations.len() != 1 || inputs.len() != 1 {
        panic!("Transpiler expects ForeignCall::CHECKNULLIFIEREXISTS to have 1 destinations and 1 input, got {} and {}", destinations.len(), inputs.len());
    }
    let nullifier_offset_operand = match &inputs[0] {
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
            AvmOperand::U32 {
                value: nullifier_offset_operand,
            },
            AvmOperand::U32 {
                value: exists_offset_operand,
            },
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
            AvmOperand::U32 {
                value: msg_hash_offset_operand,
            },
            AvmOperand::U32 {
                value: msg_leaf_index_offset_operand,
            },
            AvmOperand::U32 {
                value: exists_offset_operand,
            },
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
    if destinations.len() != 0 || inputs.len() != 2 {
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
            AvmOperand::U32 {
                value: recipient_offset_operand,
            },
            AvmOperand::U32 {
                value: content_offset_operand,
            },
        ],
        ..Default::default()
    });
}

/// Two field hash instructions represent instruction's that's outputs are larger than a field element
///
/// This includes:
/// - keccak
/// - sha256
///
/// In the future the output of these may expand / contract depending on what is most efficient for the circuit
/// to reason about. In order to decrease user friction we will use two field outputs.
fn handle_2_field_hash_instruction(
    avm_instrs: &mut Vec<AvmInstruction>,
    function: &str,
    destinations: &[ValueOrArray],
    inputs: &[ValueOrArray],
) {
    // handle field returns differently
    let message_offset_maybe = inputs[0];
    let (message_offset, message_size) = match message_offset_maybe {
        ValueOrArray::HeapArray(HeapArray { pointer, size }) => (pointer.0, size),
        _ => panic!("Keccak | Sha256 address inputs destination should be a single value"),
    };

    assert!(destinations.len() == 1);
    let dest_offset_maybe = destinations[0];
    let dest_offset = match dest_offset_maybe {
        ValueOrArray::HeapArray(HeapArray { pointer, size }) => {
            assert!(size == 2);
            pointer.0
        }
        _ => panic!("Keccak | Poseidon address destination should be a single value"),
    };

    let opcode = match function {
        "avmOpcodeKeccak256" => AvmOpcode::KECCAK,
        "avmOpcodeSha256" => AvmOpcode::SHA256,
        _ => panic!(
            "Transpiler doesn't know how to process ForeignCall function {:?}",
            function
        ),
    };

    avm_instrs.push(AvmInstruction {
        opcode,
        indirect: Some(ZEROTH_OPERAND_INDIRECT | FIRST_OPERAND_INDIRECT),
        operands: vec![
            AvmOperand::U32 {
                value: dest_offset as u32,
            },
            AvmOperand::U32 {
                value: message_offset as u32,
            },
            AvmOperand::U32 {
                value: message_size as u32,
            },
        ],
        ..Default::default()
    });
}

/// A single field hash instruction includes hash functions that emit a single field element
/// directly onto the stack.
///
/// This includes (snark friendly functions):
/// - poseidon2
///
/// Pedersen is not implemented this way as the black box function representation has the correct api.
/// As the Poseidon BBF only deals with a single permutation, it is not quite suitable for our current avm
/// representation.
fn handle_single_field_hash_instruction(
    avm_instrs: &mut Vec<AvmInstruction>,
    function: &str,
    destinations: &[ValueOrArray],
    inputs: &[ValueOrArray],
) {
    // handle field returns differently
    let message_offset_maybe = inputs[0];
    let (message_offset, message_size) = match message_offset_maybe {
        ValueOrArray::HeapArray(HeapArray { pointer, size }) => (pointer.0, size),
        _ => panic!("Poseidon address inputs destination should be a single value"),
    };

    assert!(destinations.len() == 1);
    let dest_offset_maybe = destinations[0];
    let dest_offset = match dest_offset_maybe {
        ValueOrArray::MemoryAddress(dest_offset) => dest_offset.0,
        _ => panic!("Poseidon address destination should be a single value"),
    };

    let opcode = match function {
        "avmOpcodePoseidon" => AvmOpcode::POSEIDON,
        _ => panic!(
            "Transpiler doesn't know how to process ForeignCall function {:?}",
            function
        ),
    };

    avm_instrs.push(AvmInstruction {
        opcode,
        indirect: Some(FIRST_OPERAND_INDIRECT),
        operands: vec![
            AvmOperand::U32 {
                value: dest_offset as u32,
            },
            AvmOperand::U32 {
                value: message_offset as u32,
            },
            AvmOperand::U32 {
                value: message_size as u32,
            },
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
        "avmOpcodeOrigin" => AvmOpcode::ORIGIN,
        "avmOpcodeSender" => AvmOpcode::SENDER,
        "avmOpcodePortal" => AvmOpcode::PORTAL,
        "avmOpcodeFeePerL1Gas" => AvmOpcode::FEEPERL1GAS,
        "avmOpcodeFeePerL2Gas" => AvmOpcode::FEEPERL2GAS,
        "avmOpcodeFeePerDaGas" => AvmOpcode::FEEPERDAGAS,
        "avmOpcodeChainId" => AvmOpcode::CHAINID,
        "avmOpcodeVersion" => AvmOpcode::VERSION,
        "avmOpcodeBlockNumber" => AvmOpcode::BLOCKNUMBER,
        "avmOpcodeTimestamp" => AvmOpcode::TIMESTAMP,
        // "callStackDepth" => AvmOpcode::CallStackDepth,
        _ => panic!(
            "Transpiler doesn't know how to process ForeignCall function {:?}",
            function
        ),
    };

    avm_instrs.push(AvmInstruction {
        opcode,
        indirect: Some(ALL_DIRECT),
        operands: vec![AvmOperand::U32 {
            value: dest_offset as u32,
        }],
        ..Default::default()
    })
}

/// Handles Brillig's CONST opcode.
fn handle_const(
    avm_instrs: &mut Vec<AvmInstruction>,
    destination: &MemoryAddress,
    value: &FieldElement,
    bit_size: &u32,
) {
    let tag = tag_from_bit_size(*bit_size);
    let dest = destination.to_usize() as u32;

    if !matches!(tag, AvmTypeTag::FIELD) {
        avm_instrs.push(generate_set_instruction(tag, dest, value.to_u128()));
    } else {
        // We can't fit a field in an instruction. This should've been handled in Brillig.
        let field = value;
        if !field.fits_in_u128() {
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
                AvmTypeTag::UINT16 => AvmOperand::U16 {
                    value: value as u16,
                },
                AvmTypeTag::UINT32 => AvmOperand::U32 {
                    value: value as u32,
                },
                AvmTypeTag::UINT64 => AvmOperand::U64 {
                    value: value as u64,
                },
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
        operands: vec![
            AvmOperand::U32 { value: source },
            AvmOperand::U32 { value: destination },
        ],
    }
}

/// Generates an AVM MOV instruction.
fn generate_mov_instruction(indirect: Option<u8>, source: u32, dest: u32) -> AvmInstruction {
    AvmInstruction {
        opcode: AvmOpcode::MOV,
        indirect,
        operands: vec![
            AvmOperand::U32 { value: source },
            AvmOperand::U32 { value: dest },
        ],
        ..Default::default()
    }
}

/// Black box functions, for the meantime only covers pedersen operations as the blackbox function api suits our current needs.
/// (array goes in -> field element comes out)
fn handle_black_box_function(avm_instrs: &mut Vec<AvmInstruction>, operation: &BlackBoxOp) {
    match operation {
        BlackBoxOp::PedersenHash {
            inputs,
            domain_separator,
            output,
        } => {
            let message_offset = inputs.pointer.0;
            let message_size_offset = inputs.size.0;

            let index_offset = domain_separator.0;
            let dest_offset = output.0;

            avm_instrs.push(AvmInstruction {
                opcode: AvmOpcode::PEDERSEN,
                indirect: Some(SECOND_OPERAND_INDIRECT),
                operands: vec![
                    AvmOperand::U32 {
                        value: index_offset as u32,
                    },
                    AvmOperand::U32 {
                        value: dest_offset as u32,
                    },
                    AvmOperand::U32 {
                        value: message_offset as u32,
                    },
                    AvmOperand::U32 {
                        value: message_size_offset as u32,
                    },
                ],
                ..Default::default()
            });
        }
        _ => panic!(
            "Transpiler doesn't know how to process BlackBoxOp {:?}",
            operation
        ),
    }
}
/// Emit a storage write opcode
/// The current implementation writes an array of values into storage ( contiguous slots in memory )
fn handle_storage_write(
    avm_instrs: &mut Vec<AvmInstruction>,
    destinations: &Vec<ValueOrArray>,
    inputs: &Vec<ValueOrArray>,
) {
    assert!(inputs.len() == 2);
    assert!(destinations.len() == 1);

    let slot_offset_maybe = inputs[0];
    let slot_offset = match slot_offset_maybe {
        ValueOrArray::MemoryAddress(slot_offset) => slot_offset.0,
        _ => panic!("ForeignCall address destination should be a single value"),
    };

    let src_offset_maybe = inputs[1];
    let (src_offset, src_size) = match src_offset_maybe {
        ValueOrArray::HeapArray(HeapArray { pointer, size }) => (pointer.0, size),
        _ => panic!("Storage write address inputs should be an array of values"),
    };

    avm_instrs.push(AvmInstruction {
        opcode: AvmOpcode::SSTORE,
        indirect: Some(ZEROTH_OPERAND_INDIRECT),
        operands: vec![
            AvmOperand::U32 {
                value: src_offset as u32,
            },
            AvmOperand::U32 {
                value: src_size as u32,
            },
            AvmOperand::U32 {
                value: slot_offset as u32,
            },
        ],
        ..Default::default()
    })
}

/// Emit a storage read opcode
/// The current implementation reads an array of values from storage ( contiguous slots in memory )
fn handle_storage_read(
    avm_instrs: &mut Vec<AvmInstruction>,
    destinations: &Vec<ValueOrArray>,
    inputs: &Vec<ValueOrArray>,
) {
    // For the foreign calls we want to handle, we do not want inputs, as they are getters
    assert!(inputs.len() == 2); // output, len - but we dont use this len - its for the oracle
    assert!(destinations.len() == 1);

    let slot_offset_maybe = inputs[0];
    let slot_offset = match slot_offset_maybe {
        ValueOrArray::MemoryAddress(slot_offset) => slot_offset.0,
        _ => panic!("ForeignCall address destination should be a single value"),
    };

    let dest_offset_maybe = destinations[0];
    let (dest_offset, src_size) = match dest_offset_maybe {
        ValueOrArray::HeapArray(HeapArray { pointer, size }) => (pointer.0, size),
        _ => panic!("Storage write address inputs should be an array of values"),
    };

    avm_instrs.push(AvmInstruction {
        opcode: AvmOpcode::SLOAD,
        indirect: Some(SECOND_OPERAND_INDIRECT),
        operands: vec![
            AvmOperand::U32 {
                value: slot_offset as u32,
            },
            AvmOperand::U32 {
                value: src_size as u32,
            },
            AvmOperand::U32 {
                value: dest_offset as u32,
            },
        ],
        ..Default::default()
    })
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
fn map_brillig_pcs_to_avm_pcs(initial_offset: usize, brillig: &Brillig) -> Vec<usize> {
    let mut pc_map = vec![0; brillig.bytecode.len()];

    pc_map[0] = initial_offset;
    for i in 0..brillig.bytecode.len() - 1 {
        let num_avm_instrs_for_this_brillig_instr = match &brillig.bytecode[i] {
            BrilligOpcode::Const { bit_size: 254, .. } => 2,
            _ => 1,
        };
        // next Brillig pc will map to an AVM pc offset by the
        // number of AVM instructions generated for this Brillig one
        pc_map[i + 1] = pc_map[i] + num_avm_instrs_for_this_brillig_instr;
    }
    pc_map
}

fn is_integral_bit_size(bit_size: u32) -> bool {
    match bit_size {
        1 | 8 | 16 | 32 | 64 | 128 => true,
        _ => false,
    }
}

fn tag_from_bit_size(bit_size: u32) -> AvmTypeTag {
    match bit_size {
        1 => AvmTypeTag::UINT8, // temp workaround
        8 => AvmTypeTag::UINT8,
        16 => AvmTypeTag::UINT16,
        32 => AvmTypeTag::UINT32,
        64 => AvmTypeTag::UINT64,
        128 => AvmTypeTag::UINT128,
        254 => AvmTypeTag::FIELD,
        _ => panic!("The AVM doesn't support integer bit size {:?}", bit_size),
    }
}
