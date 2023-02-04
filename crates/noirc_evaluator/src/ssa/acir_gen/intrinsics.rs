use std::collections::HashMap;

use acvm::acir::{circuit::opcodes::FunctionInput, native_types::Witness};

use crate::{
    ssa::{
        context::SsaContext,
        node::{self, Node, NodeId},
    },
    Evaluator,
};

use super::{map_array, InternalVar};

//Transform the arguments of intrinsic functions into witnesses
pub(crate) fn prepare_inputs(
    arith_cache: &mut HashMap<NodeId, InternalVar>,
    memory_map: &mut HashMap<u32, InternalVar>,
    args: &[NodeId],
    cfg: &SsaContext,
    evaluator: &mut Evaluator,
) -> Vec<FunctionInput> {
    let mut inputs: Vec<FunctionInput> = Vec::new();

    for a in args {
        let l_obj = cfg.try_get_node(*a).unwrap();
        match l_obj {
            node::NodeObject::Obj(v) => match l_obj.get_type() {
                node::ObjectType::Pointer(a) => {
                    let array = &cfg.mem[a];
                    let num_bits = array.element_type.bits();
                    for i in 0..array.len {
                        let address = array.adr + i;
                        if memory_map.contains_key(&address) {
                            if let Some(wit) = memory_map[&address].cached_witness() {
                                inputs.push(FunctionInput { witness: *wit, num_bits });
                            } else {
                                let mut var = memory_map[&address].clone();
                                if var.expression().is_const() {
                                    let w = evaluator.create_intermediate_variable(
                                        memory_map[&address].expression().clone(),
                                    );
                                    *var.cached_witness_mut() = Some(w);
                                }
                                let w =
                                    var.witness(evaluator).expect("unexpected constant expression");
                                memory_map.insert(address, var);

                                inputs.push(FunctionInput { witness: w, num_bits });
                            }
                        } else {
                            inputs.push(FunctionInput {
                                witness: array.values[i as usize].cached_witness().unwrap(),
                                num_bits,
                            });
                        }
                    }
                }
                _ => match v.witness {
                    Some(w) => {
                        inputs.push(FunctionInput { witness: w, num_bits: v.size_in_bits() });
                    }
                    None => todo!("generate a witness"),
                },
            },
            _ => {
                if arith_cache.contains_key(a) {
                    let mut var = arith_cache[a].clone();
                    let witness = var.cached_witness().unwrap_or_else(|| {
                        var.witness(evaluator).expect("unexpected constant expression")
                    });
                    inputs.push(FunctionInput { witness, num_bits: l_obj.size_in_bits() });
                } else {
                    unreachable!("invalid input: {:?}", l_obj)
                }
            }
        }
    }
    inputs
}

pub(crate) fn prepare_outputs(
    memory_map: &mut HashMap<u32, InternalVar>,
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
        map_array(memory_map, a, &outputs, ctx);
    }
    outputs
}
