use acvm::{
    acir::native_types::{Witness, WitnessMap},
    acir::AcirField,
    FieldElement,
};
use js_sys::{JsString, Map, Object};
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

#[wasm_bindgen(typescript_custom_section)]
const WITNESS_MAP: &'static str = r#"
// Map from witness index to hex string value of witness.
export type WitnessMap = Map<number, string>;

/**
 * An execution result containing two witnesses.
 * 1. The full solved witness of the execution.
 * 2. The return witness which contains the given public return values within the full witness.
 */
export type SolvedAndReturnWitness = {
    solvedWitness: WitnessMap;
    returnWitness: WitnessMap;
}
"#;

// WitnessMap
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Map, js_name = "WitnessMap", typescript_type = "WitnessMap")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsWitnessMap;

    #[wasm_bindgen(constructor, js_class = "Map")]
    pub fn new() -> JsWitnessMap;

    #[wasm_bindgen(extends = Object, js_name = "SolvedAndReturnWitness", typescript_type = "SolvedAndReturnWitness")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsSolvedAndReturnWitness;

    #[wasm_bindgen(constructor, js_class = "Object")]
    pub fn new() -> JsSolvedAndReturnWitness;
}

impl Default for JsWitnessMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for JsSolvedAndReturnWitness {
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

impl From<(WitnessMap<FieldElement>, WitnessMap<FieldElement>)> for JsSolvedAndReturnWitness {
    fn from(witness_maps: (WitnessMap<FieldElement>, WitnessMap<FieldElement>)) -> Self {
        let js_solved_witness = JsWitnessMap::from(witness_maps.0);
        let js_return_witness = JsWitnessMap::from(witness_maps.1);

        let entry_map = Map::new();
        entry_map.set(&JsValue::from_str("solvedWitness"), &js_solved_witness);
        entry_map.set(&JsValue::from_str("returnWitness"), &js_return_witness);

        let solved_and_return_witness = Object::from_entries(&entry_map).unwrap();
        JsSolvedAndReturnWitness { obj: solved_and_return_witness }
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
