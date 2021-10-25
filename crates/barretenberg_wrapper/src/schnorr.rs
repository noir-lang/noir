use crate::bindings::schnorr;

pub fn fixed_base(input: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let result = construct_public_key(input);
    (
        *slice_as_array!(&result[0..32], [u8; 32]).unwrap(),
        *slice_as_array!(&result[32..64], [u8; 32]).unwrap(),
    )
}

pub fn construct_public_key(private_key: &[u8; 32]) -> [u8; 64] {
    let mut result = [0_u8; 64];
    unsafe {
        let data = private_key.as_ptr() as *const u8;
        schnorr::compute_public_key(data, result.as_mut_ptr());
    }
    result
}

pub fn construct_signature(message: &[u8], private_key: [u8; 32]) -> ([u8; 32], [u8; 32]) {
    let mut s = [0_u8; 32];
    let mut e = [0_u8; 32];
    unsafe {
        schnorr::construct_signature(
            message.as_ptr() as *const u8,
            message.len() as u64,
            private_key.as_ptr() as *const u8,
            s.as_mut_ptr(),
            e.as_mut_ptr(),
        );
    }

    (s, e)
}

//n.b. sig: [u8; 64],
pub fn verify_signature(
    pub_key: [u8; 64],
    sig_s: [u8; 32],
    sig_e: [u8; 32],
    message: &[u8],
) -> bool {
    let r;
    unsafe {
        r = schnorr::verify_signature(
            message.as_ptr() as *const u8,
            message.len() as u64,
            pub_key.as_ptr() as *const u8,
            sig_s.as_ptr() as *const u8,
            sig_e.as_ptr() as *const u8,
        );
    }
    r
    // Note, currently for Barretenberg plonk, if the signature fails
    // then the whole circuit fails.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        let mut input = [0_u8; 32];
        input[31] = 1; //FieldElement::one();

        let (res_x, res_y) = fixed_base(&input);

        let x = "0000000000000000000000000000000000000000000000000000000000000001";
        let y = "0000000000000002cf135e7506a45d632d270d45f1181294833fc48d823f272c";

        assert_eq!(x, hex::encode(res_x));
        assert_eq!(y, hex::encode(res_y));
    }

    #[test]
    fn basic_interop_schnorr() {
        // First case should pass, standard procedure for Schnorr
        let private_key = [2; 32];
        let message = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

        let public_key = construct_public_key(&private_key);
        let (sig_s, sig_e) = construct_signature(&message, private_key);
        let result = verify_signature(public_key, sig_s, sig_e, &message);
        assert!(result);

        // Should fail, since the messages are different
        let private_key = [2; 32];
        let message = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

        let public_key = construct_public_key(&private_key);
        let (sig_s, sig_e) = construct_signature(&message, private_key);
        let result = verify_signature(public_key, sig_s, sig_e, &[0, 2]);
        assert!(!result);

        // Should fail, since the signature is not valid
        let private_key = [2; 32];
        let message = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let sig_s = [1; 32];
        let sig_e = [1; 32];
        let public_key = construct_public_key(&private_key);
        let result = verify_signature(public_key, sig_s, sig_e, &message);
        assert!(!result);

        // Should fail, since the public key does not match
        let private_key_a = [1; 32];
        let private_key_b = [2; 32];
        let message = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let public_key_b = construct_public_key(&private_key_b);
        let (siga_s, siga_e) = construct_signature(&message, private_key_a);
        let result = verify_signature(public_key_b, siga_s, siga_e, &message);
        assert!(!result);

        // Test the first case again, to check if memory is being freed and overwritten properly
        let private_key = [2; 32];
        let message = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

        let public_key = construct_public_key(&private_key);
        let (sig_s, sig_e) = construct_signature(&message, private_key);
        let result = verify_signature(public_key, sig_s, sig_e, &message);
        assert!(result);
    }
}
