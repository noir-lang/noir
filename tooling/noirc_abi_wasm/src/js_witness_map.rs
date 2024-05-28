//! This can most likely be imported from acvm_js to avoid redefining it here.

use acvm::{
    acir::native_types::{Witness, WitnessMap},
    AcirField, FieldElement,
};
use js_sys::{JsString, Map};
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

// WitnessMap
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Map, js_name = "WitnessMap", typescript_type = "WitnessMap")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsWitnessMap;

    #[wasm_bindgen(constructor, js_class = "Map")]
    pub fn new() -> JsWitnessMap;

}

impl Default for JsWitnessMap {
    fn default() -> Self {
        Self::new()
    }
}

impl From<WitnessMap<FieldElement>> for JsWitnessMap {
    fn from(witness_map: WitnessMap<FieldElement>) -> Self {
        let js_map = JsWitnessMap::new();
        for (key, value) in witness_map {
            js_map.set(
                &js_sys::Number::from(key.witness_index()),
                &field_element_to_js_string(&value),
            );
        }
        js_map
    }
}

impl From<JsWitnessMap> for WitnessMap<FieldElement> {
    fn from(js_map: JsWitnessMap) -> Self {
        let mut witness_map = WitnessMap::new();
        js_map.for_each(&mut |value, key| {
            let witness_index = Witness(key.as_f64().unwrap() as u32);
            let witness_value = js_value_to_field_element(value).unwrap();
            witness_map.insert(witness_index, witness_value);
        });
        witness_map
    }
}

pub(crate) fn js_value_to_field_element(js_value: JsValue) -> Result<FieldElement, JsString> {
    let hex_str = js_value.as_string().ok_or("failed to parse field element from non-string")?;

    FieldElement::from_hex(&hex_str)
        .ok_or_else(|| format!("Invalid hex string: '{}'", hex_str).into())
}

pub(crate) fn field_element_to_js_string(field_element: &FieldElement) -> JsString {
    // This currently maps `0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000`
    // to the bigint `-1n`. This fails when converting back to a `FieldElement`.
    // js_sys::BigInt::from_str(&value.to_hex()).unwrap()

    format!("0x{}", field_element.to_hex()).into()
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use std::collections::BTreeMap;

    use acvm::{
        acir::native_types::{Witness, WitnessMap},
        AcirField, FieldElement,
    };
    use wasm_bindgen::JsValue;

    use crate::JsWitnessMap;

    #[test]
    fn test_witness_map_to_js() {
        let witness_map = BTreeMap::from([
            (Witness(1), FieldElement::one()),
            (Witness(2), FieldElement::zero()),
            (Witness(3), -FieldElement::one()),
        ]);
        let witness_map = WitnessMap::from(witness_map);

        let js_map = JsWitnessMap::from(witness_map);

        assert_eq!(
            js_map.get(&JsValue::from(1)),
            JsValue::from_str("0x0000000000000000000000000000000000000000000000000000000000000001")
        );
        assert_eq!(
            js_map.get(&JsValue::from(2)),
            JsValue::from_str("0x0000000000000000000000000000000000000000000000000000000000000000")
        );
        assert_eq!(
            js_map.get(&JsValue::from(3)),
            // Equal to 21888242871839275222246405745257275088548364400416034343698204186575808495616,
            // which is field modulus - 1: https://docs.rs/ark-bn254/latest/ark_bn254/
            JsValue::from_str("0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000")
        );
    }
}
