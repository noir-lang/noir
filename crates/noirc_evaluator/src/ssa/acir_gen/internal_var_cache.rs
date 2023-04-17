use crate::{
    ssa::{
        acir_gen::InternalVar,
        context::SsaContext,
        node::{Node, NodeId, NodeObject, ObjectType},
    },
    Evaluator,
};
use acvm::{
    acir::native_types::{Expression, Witness},
    FieldElement,
};
use std::collections::HashMap;

use super::expression_to_witness;

#[derive(Default)]
pub(crate) struct InternalVarCache {
    /// Map node id to an InternalVar
    inner: HashMap<NodeId, InternalVar>,
    /// Map field values to an InternalVar, which lazily gets a witness when required
    /// This avoids us to re-create another witness for the same value
    /// A witness for a field value should be avoided but can be needed as an opcode input, for example the logic opcode.
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
                // use the InternalVar from constants if exists
                let field_value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be());
                let mut result = InternalVar::from_constant(field_value);
                if let Some(c_var) = self.constants.get(&field_value) {
                    result = c_var.clone();
                }

                //use witness from other nodes if exists
                let new_vec = Vec::new();
                let constant_ids = ctx.constants.get(&field_value).unwrap_or(&new_vec);
                let cached_witness =
                    constant_ids.iter().find_map(|c_id| match self.inner.get(c_id) {
                        Some(i_var) => *i_var.cached_witness(),
                        None => None,
                    });
                if let Some(w) = cached_witness {
                    result.set_witness(w);
                }

                result
            }
            NodeObject::Variable(variable) => {
                let variable_type = variable.get_type();
                match variable_type {
                    ObjectType::Numeric(..) => {
                        let witness =
                            variable.witness.unwrap_or_else(|| evaluator.add_witness_to_cs());
                        InternalVar::from_witness(witness)
                    }
                    ObjectType::ArrayPointer(_) | ObjectType::NotAnObject => return None,
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
        if let Some(ids) = ctx.constants.get(&value) {
            //we have a constant node object for the value
            if let Some(id) = ids.first() {
                let var = self.get_or_compute_internal_var_unwrap(*id, evaluator, ctx);
                if let Some(w) = var.cached_witness() {
                    return *w;
                }
            }
            // We generate a witness and assigns it
            let w = evaluator.create_intermediate_variable(Expression::from(value));
            for &id in ids {
                let mut cached_var = self.get_or_compute_internal_var_unwrap(id, evaluator, ctx);
                if let Some(cached_witness) = cached_var.cached_witness() {
                    assert_eq!(*cached_witness, w);
                } else {
                    cached_var.set_witness(w);
                }
                self.update(cached_var);
            }
            w
        } else {
            //if not, we use the constants map
            let var =
                self.constants.entry(value).or_insert_with(|| InternalVar::from_constant(value));
            Self::const_to_witness_helper(var, evaluator)
        }
    }

    // Helper function which generates a witness for an InternalVar
    // Do not call outside const_to_witness()
    fn const_to_witness_helper(var: &mut InternalVar, evaluator: &mut Evaluator) -> Witness {
        let w = Self::internal_get_or_compute_witness(var, evaluator);
        if w.is_none() {
            let witness = expression_to_witness(var.to_expression(), evaluator);
            var.set_witness(witness);
        }
        var.cached_witness().expect("Infallible, the witness is computed before")
    }

    /// Get or compute a witness for an internal var
    /// WARNING: It generates a witness even if the internal var is constant, so it should be used only if the var is an input
    /// to some ACIR opcode which requires a witness
    pub(crate) fn get_or_compute_witness_unwrap(
        &mut self,
        mut var: InternalVar,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
    ) -> Witness {
        if let Some(v) = var.to_const() {
            self.const_to_witness(v, evaluator, ctx)
        } else {
            let w = Self::internal_get_or_compute_witness(&mut var, evaluator)
                .expect("infallible non const expression");
            var.set_witness(w);
            self.update(var);
            w
        }
    }

    /// Get or compute a witness equating the internal var
    /// It returns None when the variable is a constant instead of creating a witness
    /// because we should not need a witness in that case
    /// If you really need one, you can use get_or_compute_witness_unwrap()
    pub(crate) fn get_or_compute_witness(
        &mut self,
        mut var: InternalVar,
        evaluator: &mut Evaluator,
    ) -> Option<Witness> {
        let w = Self::internal_get_or_compute_witness(&mut var, evaluator);
        if w.is_some() {
            assert!(var.cached_witness().is_some());
        } else {
            return None;
        };
        self.update(var);

        w
    }

    /// Generates a `Witness` that is equal to the `expression`.
    /// - If a `Witness` has previously been generated, we return it.
    /// - If the Expression represents a constant, we return None.
    fn internal_get_or_compute_witness(
        var: &mut InternalVar,
        evaluator: &mut Evaluator,
    ) -> Option<Witness> {
        // Check if we've already generated a `Witness` which is equal to
        // the stored `Expression`
        if let Some(witness) = var.cached_witness() {
            return Some(*witness);
        }

        // We do not generate a witness for constant values. It can only be done at the InternalVarCache level.
        if var.is_const_expression() {
            return None;
        }

        var.set_witness(expression_to_witness(var.expression().clone(), evaluator));

        *var.cached_witness()
    }

    pub(super) fn update(&mut self, var: InternalVar) {
        if let Some(id) = var.get_id() {
            self.inner.insert(id, var);
        } else if let Some(value) = var.to_const() {
            self.constants.insert(value, var);
        }
    }
}

#[cfg(test)]
mod test {

    use acvm::{acir::native_types::Witness, FieldElement};

    use crate::{
        ssa::{
            acir_gen::internal_var_cache::{InternalVar, InternalVarCache},
            context::SsaContext,
        },
        Evaluator,
    };

    // Check that only one witness is generated for const values
    #[test]
    fn test_const_witness() {
        let mut eval = Evaluator::default();
        let ctx = SsaContext::default();
        let mut var_cache = InternalVarCache::default();
        let v1 = var_cache.get_or_compute_internal_var_unwrap(ctx.one(), &mut eval, &ctx);
        let v2 = var_cache.get_or_compute_internal_var_unwrap(ctx.zero(), &mut eval, &ctx);
        let w1 = var_cache.get_or_compute_witness_unwrap(v1, &mut eval, &ctx);
        let w2 = var_cache.get_or_compute_witness_unwrap(v2, &mut eval, &ctx);
        let w11 = var_cache.get_or_compute_witness_unwrap(
            InternalVar::from_constant(FieldElement::one()),
            &mut eval,
            &ctx,
        );
        let w21 = var_cache.get_or_compute_witness_unwrap(
            InternalVar::from_constant(FieldElement::zero()),
            &mut eval,
            &ctx,
        );
        let two = FieldElement::one() + FieldElement::one();
        assert!(var_cache.constants.is_empty());
        assert_eq!(w1, w11);
        assert_eq!(w2, w21);
        var_cache.const_to_witness(two, &mut eval, &ctx);
        assert!(var_cache.constants.len() == 1);
        var_cache.const_to_witness(two, &mut eval, &ctx);
        assert!(var_cache.constants.len() == 1);
        var_cache.const_to_witness(FieldElement::one(), &mut eval, &ctx);
        assert!(var_cache.constants.len() == 1);
    }

    #[test]
    fn internal_var_from_witness() {
        let mut evaluator = Evaluator::default();
        let expected_witness = Witness(1234);
        // Initialize an InternalVar with a `Witness`
        let mut internal_var = InternalVar::from_witness(expected_witness);

        // We should get back the same `Witness`
        let got_witness =
            InternalVarCache::internal_get_or_compute_witness(&mut internal_var, &mut evaluator);
        match got_witness {
            Some(got_witness) => assert_eq!(got_witness, expected_witness),
            None => panic!("expected a `Witness` value"),
        }
    }
}
