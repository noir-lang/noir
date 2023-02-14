use crate::{
    ssa::{
        acir_gen::{
            constraints::{bound_constraint_with_offset, to_radix_base},
            expression_from_witness,
            operations::sort::evaluate_permutation,
            AcirMem, InternalVar, InternalVarCache,
        },
        builtin,
        context::SsaContext,
        mem::{ArrayId, Memory},
        node::{self, Instruction, Node, NodeId, ObjectType},
    },
    Evaluator,
};
use acvm::acir::{
    circuit::{
        directives::Directive,
        opcodes::{BlackBoxFuncCall, FunctionInput, Opcode as AcirOpcode},
    },
    native_types::{Expression, Witness},
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
    var_cache: &mut InternalVarCache,
    memory_map: &mut AcirMem,
    ctx: &SsaContext,
    evaluator: &mut Evaluator,
) -> Option<InternalVar> {
    use builtin::Opcode;

    let instruction_id = instruction.id;
    let res_type = instruction.res_type;

    let outputs;
    match opcode {
        Opcode::ToBits => {
            // TODO: document where `0` and `1` are coming from, for args[0], args[1]
            let bit_size = ctx.get_as_constant(args[1]).unwrap().to_u128() as u32;
            let l_c = var_cache.get_or_compute_internal_var_unwrap(args[0], evaluator, ctx);
            outputs = to_radix_base(l_c.expression(), 2, bit_size, evaluator);
            if let ObjectType::Pointer(a) = res_type {
                memory_map.map_array(a, &outputs, ctx);
            }
        }
        Opcode::ToRadix => {
            // TODO: document where `0`, `1` and `2` are coming from, for args[0],args[1], args[2]
            let radix = ctx.get_as_constant(args[1]).unwrap().to_u128() as u32;
            let limb_size = ctx.get_as_constant(args[2]).unwrap().to_u128() as u32;
            let l_c = var_cache.get_or_compute_internal_var_unwrap(args[0], evaluator, ctx);
            outputs = to_radix_base(l_c.expression(), radix, limb_size, evaluator);
            if let ObjectType::Pointer(a) = res_type {
                memory_map.map_array(a, &outputs, ctx);
            }
        }
        Opcode::LowLevel(op) => {
            let inputs = prepare_inputs(var_cache, memory_map, args, ctx, evaluator);
            let output_count = op.definition().output_size.0 as u32;
            outputs = prepare_outputs(memory_map, instruction_id, output_count, ctx, evaluator);

            let func_call = BlackBoxFuncCall {
                name: op,
                inputs,                   //witness + bit size
                outputs: outputs.clone(), //witness
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
                    memory_map.load_array_element_constant_index(array, i).unwrap().to_expression(),
                );
            }
            outputs = prepare_outputs(memory_map, instruction_id, array.len, ctx, evaluator);
            let out_expr: Vec<Expression> =
                outputs.iter().map(|w| expression_from_witness(*w)).collect();
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
            evaluator.opcodes.push(AcirOpcode::Directive(Directive::PermutationSort {
                inputs,
                tuple: 1,
                bits,
                sort_by: vec![0],
            }));
            if let node::ObjectType::Pointer(a) = res_type {
                memory_map.map_array(a, &outputs, ctx);
            } else {
                unreachable!();
            }
        }
    }

    // If more than witness is returned,
    // the result is inside the result type of `Instruction`
    // as a pointer to an array
    (outputs.len() == 1).then_some(Expression::from(&outputs[0])).map(InternalVar::from)
}

// Transform the arguments of intrinsic functions into witnesses
fn prepare_inputs(
    var_cache: &mut InternalVarCache,
    memory_map: &mut AcirMem,
    arguments: &[NodeId],
    cfg: &SsaContext,
    evaluator: &mut Evaluator,
) -> Vec<FunctionInput> {
    let mut inputs: Vec<FunctionInput> = Vec::new();

    for argument in arguments {
        inputs.extend(resolve_node_id(argument, var_cache, memory_map, cfg, evaluator))
    }
    inputs
}

fn resolve_node_id(
    node_id: &NodeId,
    var_cache: &mut InternalVarCache,
    memory_map: &mut AcirMem,
    cfg: &SsaContext,
    evaluator: &mut Evaluator,
) -> Vec<FunctionInput> {
    let node_object = cfg.try_get_node(*node_id).expect("could not find node for {node_id}");
    match node_object {
        node::NodeObject::Obj(v) => {
            let node_obj_type = node_object.get_type();
            match node_obj_type {
                // If the `Variable` represents a Pointer
                // Then we know that it is an `Array`
                node::ObjectType::Pointer(a) => resolve_array(a, memory_map, cfg, evaluator),
                // If it is not a pointer, we attempt to fetch the witness associated with it
                _ => match v.witness {
                    Some(w) => {
                        vec![FunctionInput { witness: w, num_bits: v.size_in_bits() }]
                    }
                    None => todo!("generate a witness"),
                },
            }
        }
        _ => {
            // Upon the case that the `NodeObject` is not a `Variable`,
            // we attempt to fetch an associated `InternalVar`.
            // Otherwise, this is a internal compiler error.
            let internal_var = var_cache.get(node_id);
            match internal_var {
                Some(var) => {
                    let witness = var
                        .clone()
                        .get_or_compute_witness(evaluator, false)
                        .expect("unexpected constant expression");
                    vec![FunctionInput { witness, num_bits: node_object.size_in_bits() }]
                }
                None => unreachable!("invalid input: {:?}", node_object),
            }
        }
    }
}

fn resolve_array(
    array_id: ArrayId,
    acir_mem: &mut AcirMem,
    cfg: &SsaContext,
    evaluator: &mut Evaluator,
) -> Vec<FunctionInput> {
    let mut inputs = Vec::new();

    let array = &cfg.mem[array_id];
    let num_bits = array.element_type.bits();
    for i in 0..array.len {
        let mut arr_element = acir_mem
            .load_array_element_constant_index(array, i)
            .expect("array index out of bounds");

        let witness = arr_element.get_or_compute_witness(evaluator, true).expect(
            "infallible: `None` can only be returned when we disallow constant Expressions.",
        );
        let func_input = FunctionInput { witness, num_bits };

        acir_mem.insert(array.id, i, arr_element);

        inputs.push(func_input)
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
    if let node::ObjectType::Pointer(a) = l_obj.get_type() {
        memory_map.map_array(a, &outputs, ctx);
    }
    outputs
}
