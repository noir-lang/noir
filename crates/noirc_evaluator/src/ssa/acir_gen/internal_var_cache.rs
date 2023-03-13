use crate::{
    ssa::{
        acir_gen::InternalVar,
        context::SsaContext,
        node::{Node, NodeId, NodeObject, ObjectType},
    },
    Evaluator,
};
use acvm::{acir::native_types::Witness, FieldElement};
use std::collections::HashMap;

use super::expression_to_witness;

#[derive(Default)]
pub struct InternalVarCache {
    inner: HashMap<NodeId, InternalVar>,
    constants: HashMap<FieldElement, InternalVar>,
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
                let mut result = InternalVar::from_constant(field_value);
                if let Some(c_var) = self.constants.get(&field_value) {
                    result = c_var.clone();
                }
                result.set_id(id);
                //map nodes of other types to the same var
                if let Some(constant_vars) = ctx.constants.get(&field_value) {
                    for c_var in constant_vars {
                        if let Some(id) = c_var.get_id() {
                            self.inner.insert(id, result.to_owned());
                        }
                    }
                }
                result.to_owned()
            }
            NodeObject::Variable(variable) => {
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
            // TODO: Why do we create a `Witness` for an instruction (Guillaume)
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
        self.inner.get(id)
    }

    // Transform a field element into a witness
    // It implements a 'get or create' pattern to ensure only one witness is created per element
    fn const_to_witness(
        &mut self,
        value: FieldElement,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
    ) -> Witness {
        if let Some(vars) = ctx.constants.get(&value) {
            //we have a constant node object for the value
            if !vars.is_empty() {
                let id = vars
                    .first()
                    .unwrap()
                    .get_id()
                    .expect("infaillible: SsaContext always set the id");
                let mut var = self.get_or_compute_internal_var_unwrap(id, evaluator, ctx);
                return Self::const_to_witness_helper(&mut var, evaluator);
            }
        }
        //if not, we use the constants map
        let var = self.constants.entry(value).or_insert(InternalVar::from_constant(value));
        Self::const_to_witness_helper(var, evaluator)
    }

    // Helper function which generates a witness for an InternalVar
    // Do not call outside const_to_witness()
    fn const_to_witness_helper(var: &mut InternalVar, evaluator: &mut Evaluator) -> Witness {
        var.get_or_compute_witness(evaluator)
            .unwrap_or_else(|| expression_to_witness(var.to_expression(), evaluator))
    }

    /// Get or compute a witness for an internal var
    /// WARNING: It generates a witness even if the internal var is constant, so it should be used only if the var is an input
    /// to some ACIR opcode which requires a witness
    pub fn get_or_compute_witness_unwrap(
        &mut self,
        var: &mut InternalVar,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
    ) -> Witness {
        if let Some(v) = var.to_const() {
            self.const_to_witness(v, evaluator, ctx)
        } else {
            var.get_or_compute_witness(evaluator).expect("infaillible non const expression")
        }
    }
}
