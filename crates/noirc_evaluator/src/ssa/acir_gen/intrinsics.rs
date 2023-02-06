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
    let node_object = cfg.try_get_node(*node_id).expect("could not find node for {node_id}");
    // TODO `node::NodeObject::Obj` is not intuitive.
    // TODO should this be changed to `node::NodeObject::Variable` ?
    match node_object {
        node::NodeObject::Obj(v) => {
            let node_obj_type = node_object.get_type();
            match node_obj_type {
                // If the `Variable` represents a Pointer
                // Then we know that it is an `Array`
                // TODO: should change Pointer to `ArrayPointer`
                node::ObjectType::Pointer(a) => resolve_array(a, memory_map, cfg, evaluator),
                // If it is not a pointer, we attempt to fetch the witness associated with it
                // TODO Open an issue regarding the below todo panic
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
            let internal_var = arith_cache.get(node_id);
            match internal_var {
                Some(_var) => {
                    let mut var = _var.clone();
                    let witness = var.cached_witness().unwrap_or_else(|| {
                        var.witness(evaluator, false).expect("unexpected constant expression")
                    });
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
        let address = array.adr + i;
        // TODO why can't this method iterate the `Array` values only
        // TODO instead of checking in the memory_map first?

        let internal_var = match memory_map.internal_var(&address) {
            Some(var) => var,
            None => {
                inputs.push(FunctionInput {
                    // TODO: this .expect implies that you cannot pass arrays
                    // TODO with constants to ACIR low level functions
                    witness: array.values[i as usize]
                        .cached_witness()
                        .expect("unexpected constant expression."),
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
                let w = var.witness(evaluator, false).expect("unexpected constant expression");
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
