use super::{Barretenberg, Error, FIELD_BYTES, WASM_SCRATCH_BYTES};

pub(crate) trait SchnorrSig {
    fn construct_signature(
        &self,
        message: &[u8],
        private_key: [u8; 32],
    ) -> Result<([u8; 32], [u8; 32]), Error>;
    fn construct_public_key(&self, private_key: [u8; 32]) -> Result<[u8; 64], Error>;
    fn verify_signature(
        &self,
        pub_key: [u8; 64],
        sig_s: [u8; 32],
        sig_e: [u8; 32],
        message: &[u8],
    ) -> Result<bool, Error>;
}

impl SchnorrSig for Barretenberg {
    fn construct_signature(
        &self,
        message: &[u8],
        private_key: [u8; 32],
    ) -> Result<([u8; 32], [u8; 32]), Error> {
        let sig_s_ptr: usize = 0;
        let sig_e_ptr: usize = sig_s_ptr + FIELD_BYTES;
        let private_key_ptr: usize = sig_e_ptr + FIELD_BYTES;
        let message_ptr: usize = private_key_ptr + private_key.len();
        assert!(
            message_ptr + message.len() < WASM_SCRATCH_BYTES,
            "Message overran wasm scratch space"
        );

        self.transfer_to_heap(&private_key, private_key_ptr);
        self.transfer_to_heap(message, message_ptr);
        self.call_multiple(
            "construct_signature",
            vec![
                &message_ptr.into(),
                &message.len().into(),
                &private_key_ptr.into(),
                &sig_s_ptr.into(),
                &sig_e_ptr.into(),
            ],
        )?;

        let sig_s: [u8; FIELD_BYTES] = self.read_memory(sig_s_ptr);
        let sig_e: [u8; FIELD_BYTES] = self.read_memory(sig_e_ptr);

        Ok((sig_s, sig_e))
    }

    #[allow(dead_code)]
    fn construct_public_key(&self, private_key: [u8; 32]) -> Result<[u8; 64], Error> {
        let private_key_ptr: usize = 0;
        let result_ptr: usize = private_key_ptr + FIELD_BYTES;

        self.transfer_to_heap(&private_key, private_key_ptr);

        self.call_multiple(
            "compute_public_key",
            vec![&private_key_ptr.into(), &result_ptr.into()],
        )?;

        Ok(self.read_memory(result_ptr))
    }

    fn verify_signature(
        &self,
        pub_key: [u8; 64],
        sig_s: [u8; 32],
        sig_e: [u8; 32],
        message: &[u8],
    ) -> Result<bool, Error> {
        let public_key_ptr: usize = 0;
        let sig_s_ptr: usize = public_key_ptr + pub_key.len();
        let sig_e_ptr: usize = sig_s_ptr + sig_s.len();
        let message_ptr: usize = sig_e_ptr + sig_e.len();
        assert!(
            message_ptr + message.len() < WASM_SCRATCH_BYTES,
            "Message overran wasm scratch space"
        );

        self.transfer_to_heap(&pub_key, public_key_ptr);
        self.transfer_to_heap(&sig_s, sig_s_ptr);
        self.transfer_to_heap(&sig_e, sig_e_ptr);
        self.transfer_to_heap(message, message_ptr);

        let verified = self.call_multiple(
            "verify_signature",
            vec![
                &message_ptr.into(),
                &message.len().into(),
                &public_key_ptr.into(),
                &sig_s_ptr.into(),
                &sig_e_ptr.into(),
            ],
        )?;

        // Note, currently for Barretenberg plonk, if the signature fails
        // then the whole circuit fails.
        Ok(verified.try_into()?)
    }
}
