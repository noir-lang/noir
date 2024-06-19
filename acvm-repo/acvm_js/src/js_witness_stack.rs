use acvm::{acir::native_types::WitnessStack, FieldElement};
use js_sys::{Array, Map, Object};
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

use crate::JsWitnessMap;

#[wasm_bindgen(typescript_custom_section)]
const WITNESS_STACK: &'static str = r#"
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
    pub fn new() -> JsStackItem;
}

impl Default for JsWitnessStack {
    fn default() -> Self {
        Self::new()
    }
}

impl From<WitnessStack<FieldElement>> for JsWitnessStack {
    fn from(mut witness_stack: WitnessStack<FieldElement>) -> Self {
        let js_witness_stack = JsWitnessStack::new();
        while let Some(stack_item) = witness_stack.pop() {
            let js_map = JsWitnessMap::from(stack_item.witness);
            let js_index = JsValue::from_f64(stack_item.index.into());

            let entry_map = Map::new();
            entry_map.set(&JsValue::from_str("index"), &js_index);
            entry_map.set(&JsValue::from_str("witness"), &js_map);
            let stack_item = Object::from_entries(&entry_map).unwrap();

            js_witness_stack.push(&stack_item);
        }
        // `reverse()` returns an `Array` so we have to wrap it
        JsWitnessStack { obj: js_witness_stack.reverse() }
    }
}

impl From<JsWitnessStack> for WitnessStack<FieldElement> {
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
