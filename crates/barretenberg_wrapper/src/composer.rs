use crate::bindings::dsl_standard_format;

/// # Safety
/// pippenger must point to a valid Pippenger object
pub unsafe fn smart_contract(
    pippenger: *mut ::std::os::raw::c_void,
    g2_ptr: &[u8],
    cs_ptr: &[u8],
    output_buf: *mut *mut u8,
) -> u32 {
    dsl_standard_format::composer__smart_contract(
        pippenger,
        g2_ptr.as_ptr() as *const u8,
        cs_ptr.as_ptr() as *const u8,
        output_buf,
    )
}

/// # Safety
/// cs_prt must point to a valid constraints system structure of type standard_format
pub unsafe fn get_circuit_size(cs_prt: *const u8) -> u32 {
    dsl_standard_format::composer__get_circuit_size(cs_prt)
    // TODO test with a circuit of size 2^19 cf: https://github.com/noir-lang/noir/issues/12
}

/// # Safety
/// pippenger must point to a valid Pippenger object
pub unsafe fn create_proof(
    pippenger: *mut ::std::os::raw::c_void,
    cs_ptr: &[u8],
    g2_ptr: &[u8],
    witness_ptr: &[u8],
    proof_data_ptr: *mut *mut u8,
) -> u64 {
    dsl_standard_format::composer__new_proof(
        pippenger,
        g2_ptr.as_ptr() as *const u8,
        cs_ptr.as_ptr() as *const u8,
        witness_ptr.as_ptr() as *const u8,
        proof_data_ptr as *const *mut u8 as *mut *mut u8,
    )
}
/// # Safety
/// pippenger must point to a valid Pippenger object
pub unsafe fn verify(
    // XXX: Important: This assumes that the proof does not have the public inputs pre-pended to it
    // This is not the case, if you take the proof directly from Barretenberg
    pippenger: *mut ::std::os::raw::c_void,
    proof: &[u8],
    public_inputs: &[u8],
    cs_ptr: &[u8],
    g2_ptr: &[u8],
) -> bool {
    let proof_ptr = proof.as_ptr() as *const u8;
    let verified;
    if !public_inputs.is_empty() {
        verified = dsl_standard_format::composer__verify_proof_with_public_inputs(
            pippenger,
            g2_ptr.as_ptr() as *const u8,
            cs_ptr.as_ptr() as *const u8,
            public_inputs.as_ptr() as *const u8,
            proof_ptr as *mut u8,
            proof.len() as u32,
        );
    } else {
        verified = dsl_standard_format::composer__verify_proof(
            pippenger,
            g2_ptr.as_ptr() as *const u8,
            cs_ptr.as_ptr() as *const u8,
            proof_ptr as *mut u8,
            proof.len() as u32,
        );
    }

    verified
}
