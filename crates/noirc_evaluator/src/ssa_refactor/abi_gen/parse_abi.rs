use noirc_abi::FunctionSignature;

#[derive(Debug)]
pub(crate) struct ProcessedAbiParam {
    pub name: String,
    /// Number of field elements needed to
    /// encode the Type.
    pub num_field_elements_needed: u32,
    /// Witness offset for this type.
    /// TODO: document more
    pub witness_start: u32,
}

pub(crate) fn parse_abi(function_signature: &FunctionSignature) -> (Vec<ProcessedAbiParam>, u32) {
    let mut processed_params = Vec::new();
    let mut witness_start = 0;
    for param in function_signature.0.iter() {
        let num_field_elements_needed = param.typ.field_count();
        processed_params.push(ProcessedAbiParam {
            name: param.name.clone(),
            num_field_elements_needed,
            witness_start,
        });
        witness_start += num_field_elements_needed;
    }
    (processed_params, witness_start)
}
