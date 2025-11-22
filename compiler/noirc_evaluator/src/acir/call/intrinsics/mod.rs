use iter_extended::vecmap;

use crate::acir::{arrays, types::AcirValue};
use crate::errors::RuntimeError;
use crate::ssa::ir::{
    dfg::DataFlowGraph,
    instruction::{Hint, Intrinsic},
    types::Type,
    value::ValueId,
};

use super::Context;

mod slice_ops;

impl Context<'_> {
    /// Returns a vector of `AcirVar`s constrained to be result of the function call.
    ///
    /// The function being called is required to be intrinsic.
    pub(super) fn convert_ssa_intrinsic_call(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        result_ids: &[ValueId],
    ) -> Result<Vec<AcirValue>, RuntimeError> {
        match intrinsic {
            Intrinsic::Hint(Hint::BlackBox) => {
                // Identity function; at the ACIR level this is a no-op, it only affects the SSA.
                assert_eq!(
                    arguments.len(),
                    result_ids.len(),
                    "ICE: BlackBox input and output lengths should match."
                );
                Ok(arguments.iter().map(|v| self.convert_value(*v, dfg)).collect())
            }
            Intrinsic::BlackBox(black_box) => {
                // Slice arguments to blackbox functions would break the following logic (due to being split over two `ValueIds`)
                // No blackbox functions currently take slice arguments so we have an assertion here to catch if this changes in the future.
                assert!(
                    !arguments.iter().any(|arg| matches!(dfg.type_of_value(*arg), Type::Slice(_))),
                    "ICE: Slice arguments passed to blackbox function"
                );

                let inputs = vecmap(arguments, |arg| self.convert_value(*arg, dfg));

                let output_count = result_ids.iter().fold(0usize, |sum, result_id| {
                    sum + dfg.type_of_value(*result_id).flattened_size() as usize
                });

                let vars = self.acir_context.black_box_function(
                    black_box,
                    inputs,
                    None,
                    output_count,
                    Some(self.current_side_effects_enabled_var),
                )?;

                Ok(self.convert_vars_to_values(vars, dfg, result_ids))
            }
            Intrinsic::ToRadix(endian) => {
                let field = self.convert_value(arguments[0], dfg).into_var()?;
                let radix = self.convert_value(arguments[1], dfg).into_var()?;

                let Type::Array(result_type, array_length) = dfg.type_of_value(result_ids[0])
                else {
                    unreachable!("ICE: ToRadix result must be an array");
                };
                assert!(
                    result_type.len() == 1,
                    "ICE: ToRadix result type must have a single element type"
                );
                let Type::Numeric(numeric_type) = result_type[0] else {
                    unreachable!("ICE: ToRadix result element type must be numeric");
                };

                self.acir_context
                    .radix_decompose(endian, field, radix, array_length, numeric_type)
                    .map(|array| vec![array])
            }
            Intrinsic::ToBits(endian) => {
                let field = self.convert_value(arguments[0], dfg).into_var()?;

                let Type::Array(result_type, array_length) = dfg.type_of_value(result_ids[0])
                else {
                    unreachable!("ICE: ToBits result must be an array");
                };
                assert!(
                    result_type.len() == 1,
                    "ICE: ToBits result type must have a single element type"
                );
                let Type::Numeric(numeric_type) = result_type[0] else {
                    unreachable!("ICE: ToBits result element type must be numeric");
                };

                self.acir_context
                    .bit_decompose(endian, field, array_length, numeric_type)
                    .map(|array| vec![array])
            }

            Intrinsic::SlicePushBack => self.convert_slice_push_back(arguments, dfg, result_ids),
            Intrinsic::SlicePushFront => self.convert_slice_push_front(arguments, dfg),
            Intrinsic::SlicePopBack => self.convert_slice_pop_back(arguments, dfg, result_ids),
            Intrinsic::SlicePopFront => self.convert_slice_pop_front(arguments, dfg, result_ids),
            Intrinsic::SliceInsert => self.convert_slice_insert(arguments, dfg, result_ids),
            Intrinsic::SliceRemove => self.convert_slice_remove(arguments, dfg, result_ids),

            Intrinsic::AsWitness => {
                let arg = arguments[0];
                let input = self.convert_value(arg, dfg).into_var()?;
                Ok(self
                    .acir_context
                    .get_or_create_witness_var(input)
                    .map(|val| self.convert_vars_to_values(vec![val], dfg, result_ids))?)
            }
            Intrinsic::DerivePedersenGenerators => Err(RuntimeError::AssertConstantFailed {
                call_stack: self.acir_context.get_call_stack(),
            }),
            Intrinsic::ApplyRangeConstraint => {
                unreachable!(
                    "ICE: `Intrinsic::ApplyRangeConstraint` calls should be transformed into an `Instruction::RangeCheck`"
                );
            }
            Intrinsic::FieldLessThan => {
                unreachable!("FieldLessThan can only be called in unconstrained")
            }
            Intrinsic::IsUnconstrained
            | Intrinsic::ArrayLen
            | Intrinsic::ArrayAsStrUnchecked
            | Intrinsic::StrAsBytes
            | Intrinsic::StaticAssert
            | Intrinsic::AssertConstant
            | Intrinsic::ArrayRefCount
            | Intrinsic::SliceRefCount
            | Intrinsic::AsSlice => {
                unreachable!("Expected {intrinsic} to have been removing during SSA optimizations")
            }
        }
    }
}
