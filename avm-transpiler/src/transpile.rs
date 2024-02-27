use acvm::acir::brillig::Opcode as BrilligOpcode;
use acvm::acir::circuit::brillig::Brillig;

use acvm::brillig_vm::brillig::{
    BinaryFieldOp, BinaryIntOp, BlackBoxOp, HeapArray, MemoryAddress, Value, ValueOrArray,
};

use crate::instructions::{
    AvmInstruction, AvmOperand, AvmTypeTag, ALL_DIRECT, FIRST_OPERAND_INDIRECT,
    ZEROTH_OPERAND_INDIRECT,
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
                    BinaryFieldOp::Div => AvmOpcode::DIV,
                    BinaryFieldOp::Equals => AvmOpcode::EQ,
                };
                avm_instrs.push(AvmInstruction {
                    opcode: avm_opcode,
                    indirect: Some(ALL_DIRECT),
                    tag: Some(AvmTypeTag::FIELD),
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
                let avm_opcode = match op {
                    BinaryIntOp::Add => AvmOpcode::ADD,
                    BinaryIntOp::Sub => AvmOpcode::SUB,
                    BinaryIntOp::Mul => AvmOpcode::MUL,
                    BinaryIntOp::UnsignedDiv => AvmOpcode::DIV,
                    BinaryIntOp::Equals => AvmOpcode::EQ,
                    BinaryIntOp::LessThan => AvmOpcode::LT,
                    BinaryIntOp::LessThanEquals => AvmOpcode::LTE,
                    BinaryIntOp::And => AvmOpcode::AND,
                    BinaryIntOp::Or => AvmOpcode::OR,
                    BinaryIntOp::Xor => AvmOpcode::XOR,
                    BinaryIntOp::Shl => AvmOpcode::SHL,
                    BinaryIntOp::Shr => AvmOpcode::SHR,
                    _ => panic!(
                        "Transpiler doesn't know how to process BinaryIntOp {:?}",
                        brillig_instr
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
    function: &String,
    destinations: &Vec<ValueOrArray>,
    inputs: &Vec<ValueOrArray>,
) {
    match function.as_str() {
        "emitNoteHash" | "emitNullifier" => handle_emit_note_hash_or_nullifier(
            function.as_str() == "emitNullifier",
            avm_instrs,
            destinations,
            inputs,
        ),
        "nullifierExists" => handle_nullifier_exists(avm_instrs, destinations, inputs),
        "keccak256" | "sha256" => {
            handle_2_field_hash_instruction(avm_instrs, function, destinations, inputs)
        }
        "poseidon" => {
            handle_single_field_hash_instruction(avm_instrs, function, destinations, inputs)
        }
        _ => handle_getter_instruction(avm_instrs, function, destinations, inputs),
    }
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

    if destinations.len() != 0 || inputs.len() != 1 {
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
    function: &String,
    destinations: &[ValueOrArray],
    inputs: &[ValueOrArray],
) {
    // handle field returns differently
    let hash_offset_maybe = inputs[0];
    let (hash_offset, hash_size) = match hash_offset_maybe {
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

    let opcode = match function.as_str() {
        "keccak256" => AvmOpcode::KECCAK,
        "sha256" => AvmOpcode::SHA256,
        _ => panic!(
            "Transpiler doesn't know how to process ForeignCall function {:?}",
            function
        ),
    };

    avm_instrs.push(AvmInstruction {
        opcode,
        indirect: Some(3), // 11 - addressing mode, indirect for input and output
        operands: vec![
            AvmOperand::U32 {
                value: dest_offset as u32,
            },
            AvmOperand::U32 {
                value: hash_offset as u32,
            },
            AvmOperand::U32 {
                value: hash_size as u32,
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
    function: &String,
    destinations: &[ValueOrArray],
    inputs: &[ValueOrArray],
) {
    // handle field returns differently
    let hash_offset_maybe = inputs[0];
    let (hash_offset, hash_size) = match hash_offset_maybe {
        ValueOrArray::HeapArray(HeapArray { pointer, size }) => (pointer.0, size),
        _ => panic!("Poseidon address inputs destination should be a single value"),
    };

    assert!(destinations.len() == 1);
    let dest_offset_maybe = destinations[0];
    let dest_offset = match dest_offset_maybe {
        ValueOrArray::MemoryAddress(dest_offset) => dest_offset.0,
        _ => panic!("Poseidon address destination should be a single value"),
    };

    let opcode = match function.as_str() {
        "poseidon" => AvmOpcode::POSEIDON,
        _ => panic!(
            "Transpiler doesn't know how to process ForeignCall function {:?}",
            function
        ),
    };

    avm_instrs.push(AvmInstruction {
        opcode,
        indirect: Some(1),
        operands: vec![
            AvmOperand::U32 {
                value: dest_offset as u32,
            },
            AvmOperand::U32 {
                value: hash_offset as u32,
            },
            AvmOperand::U32 {
                value: hash_size as u32,
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
    function: &String,
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

    let opcode = match function.as_str() {
        "address" => AvmOpcode::ADDRESS,
        "storageAddress" => AvmOpcode::STORAGEADDRESS,
        "origin" => AvmOpcode::ORIGIN,
        "sender" => AvmOpcode::SENDER,
        "portal" => AvmOpcode::PORTAL,
        "feePerL1Gas" => AvmOpcode::FEEPERL1GAS,
        "feePerL2Gas" => AvmOpcode::FEEPERL2GAS,
        "feePerDaGas" => AvmOpcode::FEEPERDAGAS,
        "chainId" => AvmOpcode::CHAINID,
        "version" => AvmOpcode::VERSION,
        "blockNumber" => AvmOpcode::BLOCKNUMBER,
        "timestamp" => AvmOpcode::TIMESTAMP,
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
    value: &Value,
    bit_size: &u32,
) {
    let tag = tag_from_bit_size(*bit_size);
    let dest = destination.to_usize() as u32;

    if !matches!(tag, AvmTypeTag::FIELD) {
        avm_instrs.push(generate_set_instruction(tag, dest, value.to_u128()));
    } else {
        // Handling fields is a bit more complex since we cannot fit a field in a single instruction.
        // We need to split the field into 128-bit chunks and set them individually.
        let field = value.to_field();
        if !field.fits_in_u128() {
            // If the field doesn't fit in 128 bits, we need scratch space. That's not trivial.
            // Will this ever happen? ACIR supports up to 126 bit fields.
            // However, it might be needed _inside_ the unconstrained function.
            panic!("SET: Field value doesn't fit in 128 bits, that's not supported yet!");
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
            domain_separator: _,
            output,
        } => {
            let hash_offset = inputs.pointer.0;
            let hash_size = inputs.size.0;

            let dest_offset = output.0;

            avm_instrs.push(AvmInstruction {
                opcode: AvmOpcode::PEDERSEN,
                indirect: Some(1),
                operands: vec![
                    AvmOperand::U32 {
                        value: dest_offset as u32,
                    },
                    AvmOperand::U32 {
                        value: hash_offset as u32,
                    },
                    AvmOperand::U32 {
                        value: hash_size as u32,
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
