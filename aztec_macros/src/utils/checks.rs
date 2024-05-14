use noirc_frontend::{
    graph::CrateId,
    macros_api::{FileId, HirContext, MacroError},
};

use super::errors::AztecMacroError;

/// Creates an error alerting the user that they have not downloaded the Aztec-noir library
pub fn check_for_aztec_dependency(
    crate_id: &CrateId,
    context: &HirContext,
) -> Result<(), (MacroError, FileId)> {
    if has_aztec_dependency(crate_id, context) {
        Ok(())
    } else {
        Err((AztecMacroError::AztecDepNotFound.into(), context.crate_graph[crate_id].root_file_id))
    }
}

pub fn has_aztec_dependency(crate_id: &CrateId, context: &HirContext) -> bool {
    context.crate_graph[crate_id].dependencies.iter().any(|dep| dep.as_name() == "aztec")
}
