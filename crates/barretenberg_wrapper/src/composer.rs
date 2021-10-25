use crate::bindings::dsl_standard_format;

pub fn smart_contract(
    pippenger: *mut ::std::os::raw::c_void,
    g2_ptr: &[u8],
    cs_ptr: &[u8],
    output_buf: *mut *mut u8,
) -> u32 {
    unsafe {
        let size = dsl_standard_format::composer__smart_contract(
            pippenger,
            g2_ptr.as_ptr() as *const u8,
            cs_ptr.as_ptr() as *const u8,
            output_buf,
        );
        size
    }
}

pub fn get_circuit_size(cs_prt: *const u8) -> u32 {
    let size;
    unsafe {
        size = dsl_standard_format::composer__get_circuit_size(cs_prt);
    }
    size
    // TODO handle errors!! should default to 2^19 in case of error
}

pub fn create_proof(
    pippenger: *mut ::std::os::raw::c_void,
    cs_ptr: &[u8],
    g2_ptr: &[u8],
    witness_ptr: &[u8],
    proof_data_ptr: *mut *mut u8,
) -> u64 {
    unsafe {
        let proof_size = dsl_standard_format::composer__new_proof(
            pippenger,
            g2_ptr.as_ptr() as *const u8,
            cs_ptr.as_ptr() as *const u8,
            witness_ptr.as_ptr() as *const u8,
            proof_data_ptr as *const *mut u8 as *mut *mut u8,
        );
        proof_size
    }
}

pub fn verify(
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
    unsafe {
        if (public_inputs.len() > 0) {
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
    }

    verified
}
