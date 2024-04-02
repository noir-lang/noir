use acvm::{acir::native_types::WitnessStack, FieldElement};
use js_sys::{Array, JsString, Map, Object};
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

use crate::JsWitnessMap;

// witness_map: Map<number, string>;

#[wasm_bindgen(typescript_custom_section)]
const WITNESS_MAP: &'static str = r#"
export type StackItem = {
    index: number;
    witness: WitnessMap;
}

export type WitnessStack = Array<StackItem>;
"#;

// WitnessStack
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Array, js_name = "WitnessStack", typescript_type = "WitnessStack")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsWitnessStack;

    #[wasm_bindgen(constructor, js_class = "Array")]
    pub fn new() -> JsWitnessStack;

    #[wasm_bindgen(extends = Object, js_name = "StackItem", typescript_type = "StackItem")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsStackItem;

    #[wasm_bindgen(constructor, js_class = "Object")]
    pub fn new(index: JsValue, witness: JsWitnessMap) -> JsStackItem;
}

impl Default for JsWitnessStack {
    fn default() -> Self {
        Self::new()
    }
}

impl From<WitnessStack> for JsWitnessStack {
    fn from(mut witness_stack: WitnessStack) -> Self {
        let js_witness_stack = JsWitnessStack::new();
        while let Some(stack_item) = witness_stack.pop() {
            let witness_entry_name = JsString::from("witness");
            let index_entry_name = JsString::from("index");

            let js_map = JsWitnessMap::from(stack_item.witness);
            let js_index = JsValue::from_f64(stack_item.index.into());

            let entry_map = Map::new();
            entry_map.set(&JsValue::from_str("index"), &js_index);
            entry_map.set(&JsValue::from_str("witness"), &js_map);
            let stack_item = Object::from_entries(&entry_map).unwrap();

            js_witness_stack.push(&stack_item);
        }
        js_witness_stack
    }
}

impl From<JsWitnessStack> for WitnessStack {
    fn from(js_witness_stack: JsWitnessStack) -> Self {
        let mut witness_stack = WitnessStack::default();
        js_witness_stack.for_each(&mut |stack_item, _, _| {
            let values_array = Object::values(&Object::from(stack_item));
            let index = values_array.get(0).as_f64().unwrap() as u32;
            let js_witness_map: JsWitnessMap = values_array.get(1).into();
            witness_stack.push(index, js_witness_map.into());
        });
        witness_stack
    }
}
