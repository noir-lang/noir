use crate::bindings::dsl_standard_format;

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
    //TODO do we do it here?? remove_public_inputs(self.constraint_system.public_inputs.len(), proof)
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
    // Prepend the public inputs to the proof.
    // This is how Barretenberg expects it to be.
    // This is non-standard however, so this Rust wrapper will strip the public inputs
    // from proofs created by Barretenberg. Then in Verify we prepend them again.
    //...todo DO WE DO IT HERE?
    //  let mut proof = proof.to_vec();

    //  let mut proof_with_pi = Vec::new();
    //  for assignment in public_inputs {
    //      proof_with_pi.push(assignment);//.....TODO
    //  }
    //todo..proof_with_pi.extend(proof);
    //proof = proof_with_pi;

    let proof_ptr = proof.as_ptr() as *const u8;
    let mut verified = false;
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
