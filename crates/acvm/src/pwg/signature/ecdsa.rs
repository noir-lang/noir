use crate::pwg::input_to_value;
use acir::{circuit::gate::GadgetCall, native_types::Witness};
use noir_field::FieldElement;
use std::collections::BTreeMap;

pub fn secp256k1_prehashed(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    gadget_call: &GadgetCall,
) {
    let mut inputs_iter = gadget_call.inputs.iter();

    let mut pub_key_x = [0u8; 32];
    for i in 0..32 {
        let _x_i = inputs_iter.next().expect(&format!(
            "pub_key_x should be 32 bytes long, found only {} bytes",
            i
        ));
        let x_i = input_to_value(initial_witness, _x_i);
        pub_key_x[i] = *x_i.to_bytes().last().unwrap()
    }

    let mut pub_key_y = [0u8; 32];
    for i in 0..32 {
        let _y_i = inputs_iter
            .next()
            .unwrap_or_else(|| panic!("pub_key_y should be 32 bytes long, found only {} bytes", i));
        let y_i = input_to_value(initial_witness, _y_i);
        pub_key_y[i] = *y_i.to_bytes().last().unwrap()
    }

    let mut signature = [0u8; 64];
    for i in 0..64 {
        let _sig_i = inputs_iter.next().expect(&format!(
            "signature should be 64 bytes long, found only {} bytes",
            i
        ));
        let sig_i = input_to_value(initial_witness, _sig_i);
        signature[i] = *sig_i.to_bytes().last().unwrap()
    }

    let mut hashed_message = Vec::new();
    for msg in inputs_iter {
        let msg_i_field = input_to_value(initial_witness, msg);
        let msg_i = *msg_i_field.to_bytes().last().unwrap();
        hashed_message.push(msg_i);
    }

    let result =
        ecdsa_secp256k1::verify_prehashed(&hashed_message, &pub_key_x, &pub_key_y, &signature)
            .is_ok();

    let result = match result {
        true => FieldElement::one(),
        false => {
            dbg!("signature has failed to verify");
            FieldElement::zero()
        }
    };

    initial_witness.insert(gadget_call.outputs[0], result);
}

mod ecdsa_secp256k1 {
    use std::convert::TryInto;

    use k256::{ecdsa::Signature, Scalar};
    use k256::{
        elliptic_curve::sec1::{Coordinates, ToEncodedPoint},
        AffinePoint, EncodedPoint, ProjectivePoint, PublicKey,
    };
    // This method is used to generate test vectors
    // in noir.
    fn generate_proof_data() {
        use k256::ecdsa::{signature::Signer, SigningKey};

        use sha2::{Digest, Sha256};

        use std::convert::TryFrom;
        // Signing
        let signing_key = SigningKey::from_bytes(&[2u8; 32]).unwrap();
        let message =
            b"ECDSA proves knowledge of a secret number in the context of a single message";

        let mut hasher = Sha256::new();
        hasher.update(&message);
        let digest = hasher.finalize();

        let signature: Signature = signing_key.sign(message);
        // Verification
        use k256::ecdsa::{signature::Verifier, VerifyingKey};

        let verify_key = VerifyingKey::from(&signing_key);

        if let Coordinates::Uncompressed { x, y } = verify_key.to_encoded_point(false).coordinates()
        {
            let signature_bytes: &[u8] = signature.as_ref();
            assert!(Signature::try_from(signature_bytes).unwrap() == signature);
            dbg!(x, y, digest, signature);
            verify_prehashed(&digest, &x, &y, signature_bytes).unwrap();
        } else {
            unreachable!();
        }

        assert!(verify_key.verify(message, &signature).is_ok());
    }

    /// Verify an ECDSA signature, given the hashed message
    pub fn verify_prehashed(
        hashed_msg: &[u8],
        public_key_x_bytes: &[u8],
        public_key_y_bytes: &[u8],
        signature: &[u8],
    ) -> Result<(), ()> {
        use std::convert::TryFrom;
        // Convert the inputs into k256 data structures

        let signature = Signature::try_from(signature).unwrap();

        let pub_key_x_arr: [u8; 32] = {
            let pub_key_x_bytes: &[u8] = public_key_x_bytes;
            pub_key_x_bytes.try_into().unwrap()
        };
        let pub_key_y_arr: [u8; 32] = {
            let pub_key_y_bytes: &[u8] = public_key_y_bytes;
            pub_key_y_bytes.try_into().unwrap()
        };

        let point = EncodedPoint::from_affine_coordinates(
            &pub_key_x_arr.into(),
            &pub_key_y_arr.into(),
            true,
        );
        let pubkey = PublicKey::try_from(point).unwrap();

        let z = Scalar::from_bytes_reduced(hashed_msg.into());

        // Finished converting bytes into data structures

        let r = signature.r();
        let s = signature.s();

        // Ensure signature is "low S" normalized ala BIP 0062
        if s.is_high().into() {
            return Err(());
        }

        let s_inv = s.invert().unwrap();
        let u1 = z * s_inv;
        let u2 = *r * s_inv;

        let R: AffinePoint = ((ProjectivePoint::generator() * u1)
            + (ProjectivePoint::from(*pubkey.as_affine()) * u2))
            .to_affine();

        if let Coordinates::Uncompressed { x, y: _ } = R.to_encoded_point(false).coordinates() {
            if Scalar::from_bytes_reduced(&x).eq(&r) {
                return Ok(());
            }
        }
        Err(())
    }
}
