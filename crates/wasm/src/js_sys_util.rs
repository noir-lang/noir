use std::collections::BTreeMap;

use acvm::{acir::native_types::Witness, FieldElement};

pub(crate) fn witness_map_to_js_map(witness_map: BTreeMap<Witness, FieldElement>) -> js_sys::Map {
    let js_map = js_sys::Map::new();
    for (witness, field_value) in witness_map.iter() {
        let js_idx = js_sys::Number::from(witness.0);
        let mut hex_str = "0x".to_owned();
        hex_str.push_str(&field_value.to_hex());
        let js_hex_str = js_sys::JsString::from(hex_str);
        js_map.set(&js_idx, &js_hex_str);
    }
    js_map
}
