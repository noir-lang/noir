use crate::{
    ssa::{
        acir_gen::{constraints::to_radix_base, InternalVar, InternalVarCache, MemoryMap},
        builtin,
        context::SsaContext,
        mem::ArrayId,
        node::{self, Instruction, Node, NodeId, ObjectType},
    },
    Evaluator,
};
use acvm::acir::{
    circuit::opcodes::{BlackBoxFuncCall, FunctionInput, Opcode as AcirOpcode},
    native_types::{Expression, Witness},
};

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
    memory_map: &mut MemoryMap,
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
    }
    // TODO: document why we only return something when outputs.len()==1
    // TODO what about outputs.len() > 1
    //if there are more than one witness returned, the result is inside ins.res_type as a pointer to an array
    (outputs.len() == 1).then_some(Expression::from(&outputs[0])).map(InternalVar::from)
}

// Transform the arguments of intrinsic functions into witnesses
fn prepare_inputs(
    var_cache: &mut InternalVarCache,
    memory_map: &mut MemoryMap,
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
    memory_map: &mut MemoryMap,
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
    memory_map: &mut MemoryMap,
    cfg: &SsaContext,
    evaluator: &mut Evaluator,
) -> Vec<FunctionInput> {
    let mut inputs = Vec::new();

    let array = &cfg.mem[array_id];
    let num_bits = array.element_type.bits();
    for i in 0..array.len {
        let mut arr_element = memory_map
            .load_array_element_constant_index(array, i)
            .expect("array index out of bounds");

        let witness = arr_element.get_or_compute_witness(evaluator, true).expect(
            "infallible: `None` can only be returned when we disallow constant Expressions.",
        );
        let func_input = FunctionInput { witness, num_bits };

        let address = array.adr + i;
        memory_map.insert(address, arr_element);

        inputs.push(func_input)
    }

    inputs
}

fn prepare_outputs(
    memory_map: &mut MemoryMap,
    pointer: NodeId,
    output_nb: u32,
    ctx: &SsaContext,
    evaluator: &mut Evaluator,
) -> Vec<Witness> {
    // Create fresh variables that will link to the output
    let mut outputs = Vec::with_capacity(output_nb as usize);
    for _ in 0..output_nb {
        let witness = evaluator.add_witness_to_cs();
        outputs.push(witness);
    }

    let l_obj = ctx.try_get_node(pointer).unwrap();
    if let node::ObjectType::Pointer(a) = l_obj.get_type() {
        memory_map.map_array(a, &outputs, ctx);
    }
    outputs
}
