use acir_field::AcirField;
use color_eyre::eyre;
use noir_protobuf::ProtoCodec;

use crate::{
    native_types,
    proto::acir::native::{Field, Witness},
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
