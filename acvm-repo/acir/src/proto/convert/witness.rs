use acir_field::AcirField;
use noir_protobuf::ProtoCodec;

use crate::native_types;
use crate::proto::acir::witness::{WitnessMap, WitnessStack};

use super::ProtoSchema;

impl<F> ProtoCodec<native_types::WitnessMap<F>, WitnessMap> for ProtoSchema<F>
where
    F: AcirField,
{
    fn encode(value: &native_types::WitnessMap<F>) -> WitnessMap {
        use crate::proto::acir::witness::witness_map::*;

        let values = value
            .clone()
            .into_iter()
            .map(|(w, f)| WitnessValue {
                witness: Self::encode_some(&w),
                field: Self::encode_some(&f),
            })
            .collect();

        WitnessMap { values }
    }

    fn decode(value: &WitnessMap) -> color_eyre::eyre::Result<native_types::WitnessMap<F>> {
        let mut wm = native_types::WitnessMap::default();
        for wv in &value.values {
            wm.insert(
                Self::decode_some_wrap(&wv.witness, "witness")?,
                Self::decode_some_wrap(&wv.field, "field")?,
            );
        }
        Ok(wm)
    }
}

impl<F> ProtoCodec<native_types::WitnessStack<F>, WitnessStack> for ProtoSchema<F>
where
    F: AcirField,
{
    fn encode(value: &native_types::WitnessStack<F>) -> WitnessStack {
        use crate::proto::acir::witness::witness_stack::*;

        let mut value = value.clone();
        let mut stack = Vec::new();
        while let Some(item) = value.pop() {
            stack.push(StackItem { index: item.index, witness: Self::encode_some(&item.witness) });
        }
        stack.reverse();

        WitnessStack { stack }
    }

    fn decode(value: &WitnessStack) -> color_eyre::eyre::Result<native_types::WitnessStack<F>> {
        let mut ws = native_types::WitnessStack::default();
        for item in &value.stack {
            ws.push(item.index, Self::decode_some_wrap(&item.witness, "witness")?);
        }
        Ok(ws)
    }
}
