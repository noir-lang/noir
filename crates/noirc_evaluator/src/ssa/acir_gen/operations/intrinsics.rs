use crate::{
    ssa::{
        acir_gen::{
            constraints::{bound_constraint_with_offset, to_radix_base},
            operations::sort::evaluate_permutation,
            Acir, AcirMem, InternalVar, InternalVarCache,
        },
        builtin,
        context::SsaContext,
        mem::Memory,
        node::{self, Instruction, Node, NodeId, ObjectType},
    },
    Evaluator,
};
use acvm::{
    acir::{
        circuit::{
            directives::{Directive, LogInfo},
            opcodes::{BlackBoxFuncCall, FunctionInput, Opcode as AcirOpcode},
        },
        native_types::{Expression, Witness},
        BlackBoxFunc,
    },
    FieldElement,
};
use iter_extended::vecmap;

// Generate constraints for two types of functions:
// - Builtin functions: These are functions that
// are implemented by the compiler.
// - ACIR black box functions. These are referred
// to as `LowLevel`
pub(crate) fn evaluate(
    args: &[NodeId],
    instruction: &Instruction,
    opcode: builtin::Opcode,
    acir_gen: &mut Acir,
    ctx: &SsaContext,
    evaluator: &mut Evaluator,
) -> Option<InternalVar> {
    use builtin::Opcode;

    let instruction_id = instruction.id;
    let res_type = instruction.res_type;

    let outputs;
    match opcode {
        Opcode::ToBits(endianness) => {
            // TODO: document where `0` and `1` are coming from, for args[0], args[1]
            let bit_size = ctx.get_as_constant(args[1]).unwrap().to_u128() as u32;
            let l_c =
                acir_gen.var_cache.get_or_compute_internal_var_unwrap(args[0], evaluator, ctx);
            outputs = to_radix_base(l_c.expression(), 2, bit_size, endianness, evaluator);
            if let ObjectType::ArrayPointer(a) = res_type {
                acir_gen.memory.map_array(a, &outputs, ctx);
            }
        }
        Opcode::ToRadix(endianness) => {
            // TODO: document where `0`, `1` and `2` are coming from, for args[0],args[1], args[2]
            let radix = ctx.get_as_constant(args[1]).unwrap().to_u128() as u32;
            let limb_size = ctx.get_as_constant(args[2]).unwrap().to_u128() as u32;
            let l_c =
                acir_gen.var_cache.get_or_compute_internal_var_unwrap(args[0], evaluator, ctx);
            outputs = to_radix_base(l_c.expression(), radix, limb_size, endianness, evaluator);
            if let ObjectType::ArrayPointer(a) = res_type {
                acir_gen.memory.map_array(a, &outputs, ctx);
            }
        }
        Opcode::Println(print_info) => {
            outputs = Vec::new(); // print statements do not output anything
            if print_info.show_output {
                evaluate_println(
                    &mut acir_gen.var_cache,
                    &mut acir_gen.memory,
                    print_info.is_string_output,
                    args,
                    ctx,
                    evaluator,
                );
            }
        }
        Opcode::LowLevel(op) => {
            outputs = match op {
                BlackBoxFunc::SHA256 | BlackBoxFunc::Blake2s => {
                    prepare_outputs(&mut acir_gen.memory, instruction_id, 32, ctx, evaluator)
                }
                BlackBoxFunc::Keccak256 => {
                    prepare_outputs(&mut acir_gen.memory, instruction_id, 32, ctx, evaluator)
                }
                BlackBoxFunc::Pedersen | BlackBoxFunc::FixedBaseScalarMul => {
                    prepare_outputs(&mut acir_gen.memory, instruction_id, 2, ctx, evaluator)
                }
                BlackBoxFunc::SchnorrVerify
                | BlackBoxFunc::EcdsaSecp256k1
                | BlackBoxFunc::EcdsaSecp256r1
                | BlackBoxFunc::HashToField128Security => {
                    prepare_outputs(&mut acir_gen.memory, instruction_id, 1, ctx, evaluator)
                }
                // There are some low level functions that have variable outputs and
                // should not have a set output count in Noir
                BlackBoxFunc::RecursiveAggregation => {
                    prepare_outputs_no_count(&mut acir_gen.memory, instruction_id, ctx, evaluator)
                }
                _ => panic!("Unsupported low level function {:?}", op),
            };
            let func_call = match op {
                BlackBoxFunc::SHA256 => BlackBoxFuncCall::SHA256 {
                    inputs: resolve_array(&args[0], acir_gen, ctx, evaluator),
                    outputs: outputs.to_vec(),
                },
                BlackBoxFunc::Blake2s => BlackBoxFuncCall::Blake2s {
                    inputs: resolve_array(&args[0], acir_gen, ctx, evaluator),
                    outputs: outputs.to_vec(),
                },
                BlackBoxFunc::Keccak256 => {
                    let msg_size = acir_gen
                        .var_cache
                        .get_or_compute_internal_var(args[1], evaluator, ctx)
                        .expect("ICE - could not get an expression for keccak message size");
                    let witness =
                        acir_gen.var_cache.get_or_compute_witness_unwrap(msg_size, evaluator, ctx);
                    let var_message_size = FunctionInput { witness, num_bits: 32 };
                    BlackBoxFuncCall::Keccak256VariableLength {
                        inputs: resolve_array(&args[0], acir_gen, ctx, evaluator),
                        var_message_size,
                        outputs: outputs.to_vec(),
                    }
                }
                BlackBoxFunc::Pedersen => {
                    let separator =
                        ctx.get_as_constant(args[1]).expect("domain separator to be comptime");
                    BlackBoxFuncCall::Pedersen {
                        inputs: resolve_array(&args[0], acir_gen, ctx, evaluator),
                        outputs: (outputs[0], outputs[1]),
                        domain_separator: separator.to_u128() as u32,
                    }
                }
                BlackBoxFunc::FixedBaseScalarMul => BlackBoxFuncCall::FixedBaseScalarMul {
                    input: resolve_variable(&args[0], acir_gen, ctx, evaluator).unwrap(),
                    outputs: (outputs[0], outputs[1]),
                },
                BlackBoxFunc::SchnorrVerify => BlackBoxFuncCall::SchnorrVerify {
                    public_key_x: resolve_variable(&args[0], acir_gen, ctx, evaluator).unwrap(),
                    public_key_y: resolve_variable(&args[1], acir_gen, ctx, evaluator).unwrap(),
                    signature: resolve_array(&args[2], acir_gen, ctx, evaluator),
                    message: resolve_array(&args[3], acir_gen, ctx, evaluator),
                    output: outputs[0],
                },
                BlackBoxFunc::EcdsaSecp256k1 => BlackBoxFuncCall::EcdsaSecp256k1 {
                    public_key_x: resolve_array(&args[0], acir_gen, ctx, evaluator),
                    public_key_y: resolve_array(&args[1], acir_gen, ctx, evaluator),
                    signature: resolve_array(&args[2], acir_gen, ctx, evaluator),
                    hashed_message: resolve_array(&args[3], acir_gen, ctx, evaluator),
                    output: outputs[0],
                },
                BlackBoxFunc::EcdsaSecp256r1 => BlackBoxFuncCall::EcdsaSecp256r1 {
                    public_key_x: resolve_array(&args[0], acir_gen, ctx, evaluator),
                    public_key_y: resolve_array(&args[1], acir_gen, ctx, evaluator),
                    signature: resolve_array(&args[2], acir_gen, ctx, evaluator),
                    hashed_message: resolve_array(&args[3], acir_gen, ctx, evaluator),
                    output: outputs[0],
                },
                BlackBoxFunc::HashToField128Security => BlackBoxFuncCall::HashToField128Security {
                    inputs: resolve_array(&args[0], acir_gen, ctx, evaluator),
                    output: outputs[0],
                },
                BlackBoxFunc::RecursiveAggregation => {
                    let has_previous_aggregation = evaluator.opcodes.iter().any(|op| {
                        matches!(
                            op,
                            AcirOpcode::BlackBoxFuncCall(
                                BlackBoxFuncCall::RecursiveAggregation { .. }
                            )
                        )
                    });

                    let input_aggregation_object = if !has_previous_aggregation {
                        None
                    } else {
                        Some(resolve_array(&args[4], acir_gen, ctx, evaluator))
                    };

                    BlackBoxFuncCall::RecursiveAggregation {
                        verification_key: resolve_array(&args[0], acir_gen, ctx, evaluator),
                        proof: resolve_array(&args[1], acir_gen, ctx, evaluator),
                        public_inputs: resolve_array(&args[2], acir_gen, ctx, evaluator),
                        key_hash: resolve_variable(&args[3], acir_gen, ctx, evaluator).unwrap(),
                        input_aggregation_object,
                        output_aggregation_object: outputs.to_vec(),
                    }
                }
                _ => panic!("Unsupported low level function {:?}", op),
            };
            evaluator.opcodes.push(AcirOpcode::BlackBoxFuncCall(func_call));
        }
        Opcode::Sort => {
            let mut in_expr = Vec::new();
            let array_id = Memory::deref(ctx, args[0]).unwrap();
            let array = &ctx.mem[array_id];
            let num_bits = array.element_type.bits();
            for i in 0..array.len {
                in_expr.push(
                    acir_gen
                        .memory
                        .load_array_element_constant_index(array, i)
                        .unwrap()
                        .to_expression(),
                );
            }
            outputs =
                prepare_outputs(&mut acir_gen.memory, instruction_id, array.len, ctx, evaluator);
            let out_expr: Vec<Expression> = outputs.iter().map(|w| (*w).into()).collect();
            for i in 0..(out_expr.len() - 1) {
                bound_constraint_with_offset(
                    &out_expr[i],
                    &out_expr[i + 1],
                    &Expression::zero(),
                    num_bits,
                    evaluator,
                );
            }
            let bits = evaluate_permutation(&in_expr, &out_expr, evaluator);
            let inputs = in_expr.iter().map(|a| vec![a.clone()]).collect();
            evaluator.push_opcode(AcirOpcode::Directive(Directive::PermutationSort {
                inputs,
                tuple: 1,
                bits,
                sort_by: vec![0],
            }));
            if let node::ObjectType::ArrayPointer(a) = res_type {
                acir_gen.memory.map_array(a, &outputs, ctx);
            } else {
                unreachable!();
            }
        }
    }

    // If more than witness is returned,
    // the result is inside the result type of `Instruction`
    // as a pointer to an array
    (outputs.len() == 1).then(|| InternalVar::from(outputs[0]))
}

fn resolve_variable(
    node_id: &NodeId,
    acir_gen: &mut Acir,
    cfg: &SsaContext,
    evaluator: &mut Evaluator,
) -> Option<FunctionInput> {
    let node_object = cfg.try_get_node(*node_id).expect("could not find node for {node_id}");
    match node_object {
        node::NodeObject::Variable(v) => {
            Some(FunctionInput { witness: v.witness?, num_bits: v.size_in_bits() })
        }
        _ => {
            // Upon the case that the `NodeObject` is not a `Variable`,
            // we attempt to fetch an associated `InternalVar`.
            // Otherwise, this is a internal compiler error.
            let internal_var = acir_gen.var_cache.get(node_id).expect("invalid input").clone();
            let witness = acir_gen.var_cache.get_or_compute_witness(internal_var, evaluator)?;
            Some(FunctionInput { witness, num_bits: node_object.size_in_bits() })
        }
    }
}

fn resolve_array(
    node_id: &NodeId,
    acir_gen: &mut Acir,
    cfg: &SsaContext,
    evaluator: &mut Evaluator,
) -> Vec<FunctionInput> {
    let node_object = cfg.try_get_node(*node_id).expect("could not find node for {node_id}");
    let array_id = match node_object {
        node::NodeObject::Variable(_) => {
            let node_obj_type = node_object.get_type();
            match node_obj_type {
                node::ObjectType::ArrayPointer(a) => a,
                _ => unreachable!(),
            }
        }
        _ => todo!("generate a witness"),
    };
    let mut inputs = Vec::new();

    let array = &cfg.mem[array_id];
    let num_bits = array.element_type.bits();
    for i in 0..array.len {
        let mut arr_element = acir_gen
            .memory
            .load_array_element_constant_index(array, i)
            .expect("array index out of bounds");
        let witness =
            acir_gen.var_cache.get_or_compute_witness_unwrap(arr_element.clone(), evaluator, cfg);
        let func_input = FunctionInput { witness, num_bits };
        arr_element.set_witness(witness);
        acir_gen.memory.insert(array.id, i, arr_element);

        inputs.push(func_input);
    }

    inputs
}

fn prepare_outputs(
    memory_map: &mut AcirMem,
    pointer: NodeId,
    output_nb: u32,
    ctx: &SsaContext,
    evaluator: &mut Evaluator,
) -> Vec<Witness> {
    // Create fresh variables that will link to the output
    let outputs = vecmap(0..output_nb, |_| evaluator.add_witness_to_cs());

    let l_obj = ctx.try_get_node(pointer).unwrap();
    if let node::ObjectType::ArrayPointer(a) = l_obj.get_type() {
        memory_map.map_array(a, &outputs, ctx);
    }
    outputs
}

fn prepare_outputs_no_count(
    memory_map: &mut AcirMem,
    pointer: NodeId,
    ctx: &SsaContext,
    evaluator: &mut Evaluator,
) -> Vec<Witness> {
    // Create fresh variables that will link to the output
    let l_obj = ctx.try_get_node(pointer).unwrap();
    if let node::ObjectType::ArrayPointer(a) = l_obj.get_type() {
        let mem_array = &ctx.mem[a];
        let output_nb = mem_array.len;
        let outputs = vecmap(0..output_nb, |_| evaluator.add_witness_to_cs());
        memory_map.map_array(a, &outputs, ctx);
        outputs
    } else {
        vec![evaluator.add_witness_to_cs()]
    }
}

fn evaluate_println(
    var_cache: &mut InternalVarCache,
    memory_map: &mut AcirMem,
    is_string_output: bool,
    args: &[NodeId],
    ctx: &SsaContext,
    evaluator: &mut Evaluator,
) {
    assert_eq!(args.len(), 1, "print statements can only support one argument");
    let node_id = args[0];

    let mut log_string = "".to_owned();
    let mut log_witnesses = Vec::new();

    let obj_type = ctx.object_type(node_id);
    match obj_type {
        ObjectType::ArrayPointer(array_id) => {
            let mem_array = &ctx.mem[array_id];
            let mut field_elements = Vec::new();
            for idx in 0..mem_array.len {
                let var = memory_map
                    .load_array_element_constant_index(mem_array, idx)
                    .expect("array element being logged does not exist in memory");
                let array_elem_expr = var.expression();
                if array_elem_expr.is_const() {
                    field_elements.push(array_elem_expr.q_c);
                } else {
                    let var = match var_cache.get(&node_id) {
                        Some(var) => var.clone(),
                        _ => InternalVar::from(array_elem_expr.clone()),
                    };
                    let w = var
                        .cached_witness()
                        .expect("array element to be logged is missing a witness");
                    log_witnesses.push(w);
                }
            }

            if is_string_output {
                let final_string = noirc_abi::decode_string_value(&field_elements);
                log_string.push_str(&final_string);
            } else if !field_elements.is_empty() {
                let fields = vecmap(field_elements, format_field_string);
                log_string = format!("[{}]", fields.join(", "));
            }
        }
        _ => match ctx.get_as_constant(node_id) {
            Some(field) => {
                log_string = format_field_string(field);
            }
            None => {
                let var = var_cache
                    .get(&node_id)
                    .unwrap_or_else(|| {
                        panic!(
                            "invalid input for print statement: {:?}",
                            ctx.try_get_node(node_id).expect("node is missing from SSA")
                        )
                    })
                    .clone();
                if let Some(field) = var.to_const() {
                    log_string = format_field_string(field);
                } else if let Some(w) = var_cache.get_or_compute_witness(var, evaluator) {
                    // We check whether there has already been a cached witness for this node. If not, we generate a new witness and include it in the logs
                    // TODO we need a witness because of the directive, but we should use an expression
                    log_witnesses.push(w);
                } else {
                    unreachable!(
                        "a witness should have been computed for the non-constant expression"
                    );
                }
            }
        },
    };

    // Only one of the witness vector or the output string should be non-empty
    assert!(log_witnesses.is_empty() ^ log_string.is_empty());

    let log_directive = if !log_string.is_empty() {
        Directive::Log(LogInfo::FinalizedOutput(log_string))
    } else {
        Directive::Log(LogInfo::WitnessOutput(log_witnesses))
    };

    evaluator.opcodes.push(AcirOpcode::Directive(log_directive));
}

/// This trims any leading zeroes.
/// A singular '0' will be prepended as well if the trimmed string has an odd length.
/// A hex string's length needs to be even to decode into bytes, as two digits correspond to
/// one byte.
fn format_field_string(field: FieldElement) -> String {
    let mut trimmed_field = field.to_hex().trim_start_matches('0').to_owned();
    if trimmed_field.len() % 2 != 0 {
        trimmed_field = "0".to_owned() + &trimmed_field;
    };
    "0x".to_owned() + &trimmed_field
}
