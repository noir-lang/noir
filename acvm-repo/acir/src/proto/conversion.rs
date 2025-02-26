use color_eyre::eyre::{self, Context};
use iter_extended::{try_vecmap, vecmap};
use noir_protobuf::{from_proto, to_proto, ProtoCodec, ProtoRepr};

use super::program::Program;

impl<F> ProtoCodec for crate::circuit::Program<F> {
    type Repr = Program;

    fn encode(&self) -> Self::Repr {
        Program {
            functions: vecmap(&self.functions, to_proto),
            unconstrained_functions: vecmap(&self.unconstrained_functions, to_proto),
        }
    }

    fn decode(value: &Self::Repr) -> eyre::Result<Self> {
        Ok(Self {
            functions: try_vecmap(&value.functions, from_proto).wrap_err("functions")?,
            unconstrained_functions: try_vecmap(&value.unconstrained_functions, from_proto)
                .wrap_err("unconstrained_functions")?,
        })
    }
}

impl<F> ProtoCodec for crate::circuit::brillig::BrilligBytecode<F> {
    type Repr = super::brillig::BrilligBytecode;

    fn encode(&self) -> Self::Repr {
        todo!()
    }

    fn decode(value: &Self::Repr) -> eyre::Result<Self> {
        todo!()
    }
}

impl<F> ProtoCodec for crate::circuit::Circuit<F> {
    type Repr = super::acir::Circuit;

    fn encode(&self) -> Self::Repr {
        todo!()
    }

    fn decode(value: &Self::Repr) -> eyre::Result<Self> {
        todo!()
    }
}
