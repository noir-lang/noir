use crate::{
    ssa::{
        acir_gen::InternalVar,
        context::SsaContext,
        node::{Node, NodeId, NodeObject, ObjectType},
    },
    Evaluator,
};
use acvm::FieldElement;
use std::collections::HashMap;

#[derive(Default)]
pub struct InternalVarCache {
    inner: HashMap<NodeId, InternalVar>,
}

impl InternalVarCache {
    //This function stores the substitution with the arithmetic expression in the cache
    //When an instruction performs arithmetic operation, its output can be represented as an arithmetic expression of its arguments
    //Substitute a node object as an arithmetic expression
    // Returns `None` if `NodeId` represents an array pointer.
    pub(super) fn get_or_compute_internal_var(
        &mut self,
        id: NodeId,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
    ) -> Option<InternalVar> {
        if let Some(internal_var) = self.inner.get(&id) {
            return Some(internal_var.clone());
        }

        let mut var = match ctx.try_get_node(id)? {
            NodeObject::Const(c) => {
                let field_value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be());
                InternalVar::from_constant(field_value)
            }
            NodeObject::Obj(variable) => {
                let variable_type = variable.get_type();
                match variable_type {
                    ObjectType::Boolean
                    | ObjectType::NativeField
                    | ObjectType::Signed(..)
                    | ObjectType::Unsigned(..) => {
                        let witness =
                            variable.witness.unwrap_or_else(|| evaluator.add_witness_to_cs());
                        InternalVar::from_witness(witness)
                    }
                    ObjectType::Pointer(_) | ObjectType::NotAnObject => return None,
                    ObjectType::Function => {
                        unreachable!("ICE: functions should have been removed by this stage")
                    }
                }
            }
            NodeObject::Function(..) => {
                unreachable!("ICE: functions should have been removed by this stage")
            }
            // TODO: Why do we create a `Witness` for an instruction
            NodeObject::Instr(..) => {
                let witness = evaluator.add_witness_to_cs();
                InternalVar::from_witness(witness)
            }
        };

        var.set_id(id);
        self.inner.insert(id, var.clone());
        Some(var)
    }

    pub(super) fn get_or_compute_internal_var_unwrap(
        &mut self,
        id: NodeId,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
    ) -> InternalVar {
        self.get_or_compute_internal_var(id, evaluator, ctx)
            .expect("ICE: `NodeId` type cannot be converted into an `InternalVar`")
    }

    pub(super) fn update(&mut self, id: NodeId, var: InternalVar) {
        self.inner.insert(id, var);
    }
    pub(super) fn get(&mut self, id: &NodeId) -> Option<&InternalVar> {
        self.inner.get(&id)
    }
}
