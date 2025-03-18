use std::marker::PhantomData;

use acir_field::AcirField;
use color_eyre::eyre::{self, Context};
use noir_protobuf::ProtoCodec;

use crate::circuit;
use crate::proto::program::Program;

mod acir;
mod brillig;
mod native;
mod witness;

pub(crate) struct ProtoSchema<F> {
    field: PhantomData<F>,
}

impl<F: AcirField> ProtoCodec<circuit::Program<F>, Program> for ProtoSchema<F> {
    fn encode(value: &circuit::Program<F>) -> Program {
        Program {
            functions: Self::encode_vec(&value.functions),
            unconstrained_functions: Self::encode_vec(&value.unconstrained_functions),
        }
    }

    fn decode(value: &Program) -> eyre::Result<circuit::Program<F>> {
        Ok(circuit::Program {
            functions: Self::decode_vec_wrap(&value.functions, "functions")?,
            unconstrained_functions: Self::decode_vec_wrap(
                &value.unconstrained_functions,
                "unconstrained_functions",
            )?,
        })
    }
}

impl<F> ProtoCodec<usize, u64> for ProtoSchema<F> {
    fn encode(value: &usize) -> u64 {
        *value as u64
    }

    fn decode(value: &u64) -> eyre::Result<usize> {
        (*value).try_into().wrap_err("failed to convert u64 to usize")
    }
}
