use crate::{circuit, proto::acir::circuit::Circuit};
use color_eyre::eyre;
use noir_protobuf::ProtoCodec;

use super::ProtoSchema;

impl<F> ProtoCodec<circuit::Circuit<F>, Circuit> for ProtoSchema<F> {
    fn encode(value: &circuit::Circuit<F>) -> Circuit {
        todo!()
    }

    fn decode(value: &Circuit) -> eyre::Result<circuit::Circuit<F>> {
        todo!()
    }
}
