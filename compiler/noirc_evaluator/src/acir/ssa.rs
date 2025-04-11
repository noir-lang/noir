use std::collections::BTreeMap;

use acvm::{
    FieldElement,
    acir::circuit::{ErrorSelector, ExpressionWidth, brillig::BrilligBytecode},
};
use noirc_frontend::Type as HirType;

use crate::{
    brillig::{Brillig, BrilligOptions},
    errors::RuntimeError,
    ssa::ssa_gen::Ssa,
};

use super::{Context, GeneratedAcir, SharedContext, acir_context::BrilligStdLib};

pub type Artifacts = (
    Vec<GeneratedAcir<FieldElement>>,
    Vec<BrilligBytecode<FieldElement>>,
    Vec<String>,
    BTreeMap<ErrorSelector, HirType>,
);

impl Ssa {
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn into_acir(
        self,
        brillig: &Brillig,
        brillig_options: &BrilligOptions,
        expression_width: ExpressionWidth,
    ) -> Result<Artifacts, RuntimeError> {
        codegen_acir(self, brillig, BrilligStdLib::default(), brillig_options, expression_width)
    }
}

pub(super) fn codegen_acir(
    ssa: Ssa,
    brillig: &Brillig,
    brillig_stdlib: BrilligStdLib<FieldElement>,
    brillig_options: &BrilligOptions,
    expression_width: ExpressionWidth,
) -> Result<Artifacts, RuntimeError> {
    let mut acirs = Vec::new();
    // TODO: can we parallelize this?
    let mut shared_context =
        SharedContext { brillig_stdlib: brillig_stdlib.clone(), ..SharedContext::default() };

    for function in ssa.functions.values() {
        let context = Context::new(
            &mut shared_context,
            expression_width,
            brillig,
            brillig_stdlib.clone(),
            brillig_options,
        );

        if let Some(mut generated_acir) = context.convert_ssa_function(&ssa, function)? {
            // We want to be able to insert Brillig stdlib functions anywhere during the ACIR generation process (e.g. such as on the `GeneratedAcir`).
            // As we don't want a reference to the `SharedContext` on the generated ACIR itself,
            // we instead store the opcode location at which a Brillig call to a std lib function occurred.
            // We then defer resolving the function IDs of those Brillig functions to when we have generated Brillig
            // for all normal Brillig calls.
            for (opcode_location, brillig_stdlib_func) in
                &generated_acir.brillig_stdlib_func_locations
            {
                shared_context.generate_brillig_calls_to_resolve(
                    brillig_stdlib_func,
                    function.id(),
                    *opcode_location,
                );
            }

            // Fetch the Brillig stdlib calls to resolve for this function
            if let Some(calls_to_resolve) =
                shared_context.brillig_stdlib_calls_to_resolve.get(&function.id())
            {
                // Resolve the Brillig stdlib calls
                // We have to do a separate loop as the generated ACIR cannot be borrowed as mutable after an immutable borrow
                for (opcode_location, brillig_function_pointer) in calls_to_resolve {
                    generated_acir
                        .resolve_brillig_stdlib_call(*opcode_location, *brillig_function_pointer);
                }
            }

            generated_acir.name = function.name().to_owned();
            acirs.push(generated_acir);
        }
    }

    let (brillig_bytecode, brillig_names) = shared_context
        .generated_brillig
        .into_iter()
        .map(|brillig| (BrilligBytecode { bytecode: brillig.byte_code }, brillig.name))
        .unzip();

    Ok((acirs, brillig_bytecode, brillig_names, ssa.error_selector_to_type))
}
