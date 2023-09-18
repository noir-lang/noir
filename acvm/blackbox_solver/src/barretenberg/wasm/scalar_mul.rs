use acir::FieldElement;
use num_bigint::BigUint;

use crate::barretenberg::wasm::FeatureError;

use super::{Barretenberg, Error, FIELD_BYTES};

pub(crate) trait ScalarMul {
    fn fixed_base(
        &self,
        low: &FieldElement,
        high: &FieldElement,
    ) -> Result<(FieldElement, FieldElement), Error>;
}

impl ScalarMul for Barretenberg {
    fn fixed_base(
        &self,
        low: &FieldElement,
        high: &FieldElement,
    ) -> Result<(FieldElement, FieldElement), Error> {
        let lhs_ptr: usize = 0;
        let result_ptr: usize = lhs_ptr + FIELD_BYTES;

        let low: u128 = low.try_into_u128().ok_or_else(|| {
            Error::FromFeature(FeatureError::InvalidGrumpkinScalarLimb {
                limb_as_hex: low.to_hex(),
            })
        })?;

        let high: u128 = high.try_into_u128().ok_or_else(|| {
            Error::FromFeature(FeatureError::InvalidGrumpkinScalarLimb {
                limb_as_hex: high.to_hex(),
            })
        })?;

        let mut bytes = high.to_be_bytes().to_vec();
        bytes.extend_from_slice(&low.to_be_bytes());

        // Check if this is smaller than the grumpkin modulus
        let grumpkin_integer = BigUint::from_bytes_be(&bytes);
        let grumpkin_modulus = BigUint::from_bytes_be(&[
            48, 100, 78, 114, 225, 49, 160, 41, 184, 80, 69, 182, 129, 129, 88, 93, 151, 129, 106,
            145, 104, 113, 202, 141, 60, 32, 140, 22, 216, 124, 253, 71,
        ]);

        if grumpkin_integer >= grumpkin_modulus {
            return Err(Error::FromFeature(FeatureError::InvalidGrumpkinScalar {
                scalar_as_hex: hex::encode(grumpkin_integer.to_bytes_be()),
            }));
        }

        self.transfer_to_heap(&bytes, lhs_ptr);
        self.call_multiple("compute_public_key", vec![&lhs_ptr.into(), &result_ptr.into()])?;

        let result_bytes: [u8; 2 * FIELD_BYTES] = self.read_memory(result_ptr);
        let (pubkey_x_bytes, pubkey_y_bytes) = result_bytes.split_at(FIELD_BYTES);

        assert!(pubkey_x_bytes.len() == FIELD_BYTES);
        assert!(pubkey_y_bytes.len() == FIELD_BYTES);

        let pubkey_x = FieldElement::from_be_bytes_reduce(pubkey_x_bytes);
        let pubkey_y = FieldElement::from_be_bytes_reduce(pubkey_y_bytes);
        Ok((pubkey_x, pubkey_y))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn smoke_test() -> Result<(), Error> {
        let barretenberg = Barretenberg::new();
        let input = FieldElement::one();

        let res = barretenberg.fixed_base(&input, &FieldElement::zero())?;
        let x = "0000000000000000000000000000000000000000000000000000000000000001";
        let y = "0000000000000002cf135e7506a45d632d270d45f1181294833fc48d823f272c";

        assert_eq!(x, res.0.to_hex());
        assert_eq!(y, res.1.to_hex());
        Ok(())
    }
    #[test]
    fn low_high_smoke_test() -> Result<(), Error> {
        let barretenberg = Barretenberg::new();
        let low = FieldElement::one();
        let high = FieldElement::from(2u128);

        let res = barretenberg.fixed_base(&low, &high)?;
        let x = "0702ab9c7038eeecc179b4f209991bcb68c7cb05bf4c532d804ccac36199c9a9";
        let y = "23f10e9e43a3ae8d75d24154e796aae12ae7af546716e8f81a2564f1b5814130";

        assert_eq!(x, res.0.to_hex());
        assert_eq!(y, res.1.to_hex());
        Ok(())
    }
}
