use super::{Barretenberg, Error};

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

#[cfg(feature = "native")]
impl SchnorrSig for Barretenberg {
    fn construct_signature(
        &self,
        message: &[u8],
        private_key: [u8; 32],
    ) -> Result<([u8; 32], [u8; 32]), Error> {
        Ok(barretenberg_sys::schnorr::construct_signature(
            message,
            private_key,
        ))
    }

    fn construct_public_key(&self, private_key: [u8; 32]) -> Result<[u8; 64], Error> {
        Ok(barretenberg_sys::schnorr::construct_public_key(
            &private_key,
        ))
    }

    fn verify_signature(
        &self,
        pub_key: [u8; 64],
        sig_s: [u8; 32],
        sig_e: [u8; 32],
        message: &[u8],
    ) -> Result<bool, Error> {
        Ok(barretenberg_sys::schnorr::verify_signature(
            pub_key, sig_s, sig_e, message,
        ))

        // Note, currently for Barretenberg plonk, if the signature fails
        // then the whole circuit fails.
    }
}

#[cfg(not(feature = "native"))]
impl SchnorrSig for Barretenberg {
    fn construct_signature(
        &self,
        message: &[u8],
        private_key: [u8; 32],
    ) -> Result<([u8; 32], [u8; 32]), Error> {
        use super::{wasm::WASM_SCRATCH_BYTES, FIELD_BYTES};

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
        use super::FIELD_BYTES;

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
        use super::wasm::WASM_SCRATCH_BYTES;

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

#[test]
fn basic_interop() -> Result<(), Error> {
    let barretenberg = Barretenberg::new();

    // First case should pass, standard procedure for Schnorr
    let private_key = [2; 32];
    let message = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    let public_key = barretenberg.construct_public_key(private_key)?;
    let (sig_s, sig_e) = barretenberg.construct_signature(&message, private_key)?;
    let valid_signature = barretenberg.verify_signature(public_key, sig_s, sig_e, &message)?;
    assert!(valid_signature);

    // Should fail, since the messages are different
    let private_key = [2; 32];
    let message = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    let public_key = barretenberg.construct_public_key(private_key)?;
    let (sig_s, sig_e) = barretenberg.construct_signature(&message, private_key)?;
    let valid_signature = barretenberg.verify_signature(public_key, sig_s, sig_e, &[0, 2])?;
    assert!(!valid_signature);

    // Should fail, since the signature is not valid
    let private_key = [2; 32];
    let message = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let sig_s = [1; 32];
    let sig_e = [1; 32];

    let public_key = barretenberg.construct_public_key(private_key)?;
    let valid_signature = barretenberg.verify_signature(public_key, sig_s, sig_e, &message)?;
    assert!(!valid_signature);

    // Should fail, since the public key does not match
    let private_key_a = [1; 32];
    let private_key_b = [2; 32];
    let message = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    let public_key_b = barretenberg.construct_public_key(private_key_b)?;
    let (sig_s, sig_e) = barretenberg.construct_signature(&message, private_key_a)?;
    let valid_signature = barretenberg.verify_signature(public_key_b, sig_s, sig_e, &message)?;
    assert!(!valid_signature);

    // Test the first case again, to check if memory is being freed and overwritten properly
    let private_key = [2; 32];
    let message = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    let public_key = barretenberg.construct_public_key(private_key)?;
    let (sig_s, sig_e) = barretenberg.construct_signature(&message, private_key)?;
    let valid_signature = barretenberg.verify_signature(public_key, sig_s, sig_e, &message)?;
    assert!(valid_signature);
    Ok(())
}
