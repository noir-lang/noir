use crate::{
    ssa::{
        acir_gen::{InternalVar, MemoryMap},
        context::SsaContext,
        mem::ArrayId,
        node::{self, Node, NodeId},
    },
    Evaluator,
};
use acvm::acir::{circuit::opcodes::FunctionInput, native_types::Witness};
use std::collections::HashMap;

//Transform the arguments of intrinsic functions into witnesses
pub(crate) fn prepare_inputs(
    arith_cache: &mut HashMap<NodeId, InternalVar>,
    memory_map: &mut MemoryMap,
    args: &[NodeId],
    cfg: &SsaContext,
    evaluator: &mut Evaluator,
) -> Vec<FunctionInput> {
    let mut inputs: Vec<FunctionInput> = Vec::new();

    for a in args {
        inputs.extend(resolve_node_id(a, arith_cache, memory_map, cfg, evaluator))
    }
    inputs
}

fn resolve_node_id(
    node_id: &NodeId,
    arith_cache: &mut HashMap<NodeId, InternalVar>,
    memory_map: &mut MemoryMap,
    cfg: &SsaContext,
    evaluator: &mut Evaluator,
) -> Vec<FunctionInput> {
    let l_obj = cfg.try_get_node(*node_id).unwrap();
    match l_obj {
        node::NodeObject::Obj(v) => match l_obj.get_type() {
            node::ObjectType::Pointer(a) => resolve_array(a, memory_map, cfg, evaluator),
            _ => match v.witness {
                Some(w) => {
                    vec![FunctionInput { witness: w, num_bits: v.size_in_bits() }]
                }
                None => todo!("generate a witness"),
            },
        },
        _ => match arith_cache.get(node_id) {
            Some(_var) => {
                let mut var = _var.clone();
                let witness = var.cached_witness().unwrap_or_else(|| {
                    var.witness(evaluator).expect("unexpected constant expression")
                });
                vec![FunctionInput { witness, num_bits: l_obj.size_in_bits() }]
            }
            None => unreachable!("invalid input: {:?}", l_obj),
        },
    }
}

fn resolve_array(
    a: ArrayId,
    memory_map: &mut MemoryMap,
    cfg: &SsaContext,
    evaluator: &mut Evaluator,
) -> Vec<FunctionInput> {
    let mut inputs = Vec::new();
    let array = &cfg.mem[a];
    let num_bits = array.element_type.bits();
    for i in 0..array.len {
        let address = array.adr + i;

        let internal_var = match memory_map.internal_var(&address) {
            Some(var) => var,
            None => {
                inputs.push(FunctionInput {
                    witness: array.values[i as usize].cached_witness().unwrap(),
                    num_bits,
                });
                continue;
            }
        };

        let func_input = match internal_var.cached_witness() {
            Some(cached_witness) => FunctionInput { witness: *cached_witness, num_bits },
            None => {
                let mut var = internal_var.clone();
                if var.expression().is_const() {
                    // TODO Why is it acceptable that we create an
                    // TODO intermediate variable here for the constant
                    // TODO expression, but not in general?
                    let w =
                        evaluator.create_intermediate_variable(internal_var.expression().clone());
                    *var.cached_witness_mut() = Some(w);
                }
                let w = var.witness(evaluator).expect("unexpected constant expression");
                memory_map.insert(address, var);

                FunctionInput { witness: w, num_bits }
            }
        };
        inputs.push(func_input)
    }

    inputs
}

pub(crate) fn prepare_outputs(
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
