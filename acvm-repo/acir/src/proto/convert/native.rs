use acir_field::AcirField;
use color_eyre::eyre;
use noir_protobuf::{ProtoCodec, decode_vec_map_wrap};

use crate::{
    native_types,
    proto::acir::native::{Expression, Field, Witness},
};

use super::ProtoSchema;

impl<F: AcirField> ProtoCodec<F, Field> for ProtoSchema<F> {
    fn encode(value: &F) -> Field {
        Field { value: value.to_le_bytes() }
    }

    fn decode(value: &Field) -> eyre::Result<F> {
        Ok(F::from_le_bytes_reduce(&value.value))
    }
}

impl<F> ProtoCodec<native_types::Witness, Witness> for ProtoSchema<F> {
    fn encode(value: &native_types::Witness) -> Witness {
        Witness { index: value.0 }
    }

    fn decode(value: &Witness) -> eyre::Result<native_types::Witness> {
        Ok(native_types::Witness(value.index))
    }
}

impl<F> ProtoCodec<native_types::Expression<F>, Expression> for ProtoSchema<F>
where
    F: AcirField,
{
    fn encode(value: &native_types::Expression<F>) -> Expression {
        use crate::proto::acir::native::expression::*;
        Expression {
            mul_terms: value
                .mul_terms
                .iter()
                .map(|(q_m, wl, wr)| MulTerm {
                    q_m: Self::encode_some(q_m),
                    witness_left: Self::encode_some(wl),
                    witness_right: Self::encode_some(wr),
                })
                .collect(),
            linear_combinations: value
                .linear_combinations
                .iter()
                .map(|(q_l, w)| LinearCombination {
                    q_l: Self::encode_some(q_l),
                    witness: Self::encode_some(w),
                })
                .collect(),
            q_c: Self::encode_some(&value.q_c),
        }
    }

    fn decode(value: &Expression) -> eyre::Result<native_types::Expression<F>> {
        Ok(native_types::Expression {
            mul_terms: decode_vec_map_wrap(&value.mul_terms, "mul_terms", |mt| {
                let q_m = Self::decode_some_wrap(&mt.q_m, "q_m")?;
                let wl = Self::decode_some_wrap(&mt.witness_left, "witness_left")?;
                let wr = Self::decode_some_wrap(&mt.witness_right, "witness_right")?;
                Ok((q_m, wl, wr))
            })?,
            linear_combinations: decode_vec_map_wrap(
                &value.linear_combinations,
                "linear_combinations",
                |lc| {
                    let q_l = Self::decode_some_wrap(&lc.q_l, "q_l")?;
                    let w = Self::decode_some_wrap(&lc.witness, "witness")?;
                    Ok((q_l, w))
                },
            )?,
            q_c: Self::decode_some_wrap(&value.q_c, "q_c")?,
        })
    }
}
