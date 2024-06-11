use acvm::AcirField;
use noirc_errors::debug_info::{
    DebugFnId, DebugFunction, DebugInfo, DebugTypeId, DebugVarId, DebugVariable,
};
use noirc_printable_type::{decode_value, PrintableType, PrintableValue};
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct DebugVars<F> {
    variables: HashMap<DebugVarId, DebugVariable>,
    functions: HashMap<DebugFnId, DebugFunction>,
    types: HashMap<DebugTypeId, PrintableType>,
    frames: Vec<(DebugFnId, HashMap<DebugVarId, PrintableValue<F>>)>,
}

pub struct StackFrame<'a, F> {
    pub function_name: &'a str,
    pub function_params: Vec<&'a str>,
    pub variables: Vec<(&'a str, &'a PrintableValue<F>, &'a PrintableType)>,
}

impl<F: AcirField> DebugVars<F> {
    pub fn insert_debug_info(&mut self, info: &DebugInfo) {
        self.variables.extend(info.variables.clone());
        self.types.extend(info.types.clone());
        self.functions.extend(info.functions.clone());
    }

    pub fn get_variables(&self) -> Vec<StackFrame<F>> {
        self.frames.iter().map(|(fn_id, frame)| self.build_stack_frame(fn_id, frame)).collect()
    }

    pub fn current_stack_frame(&self) -> Option<StackFrame<F>> {
        self.frames.last().map(|(fn_id, frame)| self.build_stack_frame(fn_id, frame))
    }

    fn lookup_var(&self, var_id: DebugVarId) -> Option<(&str, &PrintableType)> {
        self.variables.get(&var_id).and_then(|debug_var| {
            let ptype = self.types.get(&debug_var.debug_type_id)?;
            Some((debug_var.name.as_str(), ptype))
        })
    }

    fn build_stack_frame<'a>(
        &'a self,
        fn_id: &DebugFnId,
        frame: &'a HashMap<DebugVarId, PrintableValue<F>>,
    ) -> StackFrame<F> {
        let debug_fn = &self.functions.get(fn_id).expect("failed to find function metadata");

        let params: Vec<&str> =
            debug_fn.arg_names.iter().map(|arg_name| arg_name.as_str()).collect();
        let vars: Vec<(&str, &PrintableValue<F>, &PrintableType)> = frame
            .iter()
            .filter_map(|(var_id, var_value)| {
                self.lookup_var(*var_id).map(|(name, typ)| (name, var_value, typ))
            })
            .collect();

        StackFrame {
            function_name: debug_fn.name.as_str(),
            function_params: params,
            variables: vars,
        }
    }

    pub fn assign_var(&mut self, var_id: DebugVarId, values: &[F]) {
        let type_id = &self.variables.get(&var_id).unwrap().debug_type_id;
        let ptype = self.types.get(type_id).unwrap();

        self.frames
            .last_mut()
            .expect("unexpected empty stack frames")
            .1
            .insert(var_id, decode_value(&mut values.iter().copied(), ptype));
    }

    pub fn assign_field(&mut self, var_id: DebugVarId, indexes: Vec<u32>, values: &[F]) {
        let current_frame = &mut self.frames.last_mut().expect("unexpected empty stack frames").1;
        let mut cursor: &mut PrintableValue<F> = current_frame
            .get_mut(&var_id)
            .unwrap_or_else(|| panic!("value unavailable for var_id {var_id:?}"));
        let cursor_type_id = &self
            .variables
            .get(&var_id)
            .unwrap_or_else(|| panic!("variable {var_id:?} not found"))
            .debug_type_id;
        let mut cursor_type = self
            .types
            .get(cursor_type_id)
            .unwrap_or_else(|| panic!("type unavailable for type id {cursor_type_id:?}"));
        for index in indexes.iter() {
            (cursor, cursor_type) = match (cursor, cursor_type) {
                (
                    PrintableValue::Vec { array_elements, is_slice },
                    PrintableType::Array { length, typ },
                ) => {
                    assert!(!*is_slice, "slice has array type");
                    if *index >= *length {
                        panic!("unexpected field index past array length")
                    }
                    if *length != array_elements.len() as u32 {
                        panic!("type/array length mismatch")
                    }
                    (array_elements.get_mut(*index as usize).unwrap(), &*Box::leak(typ.clone()))
                }
                (
                    PrintableValue::Vec { array_elements, is_slice },
                    PrintableType::Slice { typ },
                ) => {
                    assert!(*is_slice, "slice doesn't have slice type");
                    (array_elements.get_mut(*index as usize).unwrap(), &*Box::leak(typ.clone()))
                }
                (
                    PrintableValue::Struct(field_map),
                    PrintableType::Struct { name: _name, fields },
                ) => {
                    if *index as usize >= fields.len() {
                        panic!("unexpected field index past struct field length")
                    }
                    let (key, typ) = fields.get(*index as usize).unwrap();
                    (field_map.get_mut(key).unwrap(), typ)
                }
                (
                    PrintableValue::Vec { array_elements, is_slice },
                    PrintableType::Tuple { types },
                ) => {
                    assert!(!*is_slice, "slice has tuple type");
                    if *index >= types.len() as u32 {
                        panic!(
                            "unexpected field index ({index}) past tuple length ({})",
                            types.len()
                        );
                    }
                    if types.len() != array_elements.len() {
                        panic!("type/array length mismatch")
                    }
                    let typ = types.get(*index as usize).unwrap();
                    (array_elements.get_mut(*index as usize).unwrap(), typ)
                }
                _ => {
                    panic!("unexpected assign field of {cursor_type:?} type");
                }
            };
        }
        *cursor = decode_value(&mut values.iter().copied(), cursor_type);
    }

    pub fn assign_deref(&mut self, _var_id: DebugVarId, _values: &[F]) {
        unimplemented![]
    }

    pub fn get_type(&self, var_id: DebugVarId) -> Option<&PrintableType> {
        self.variables.get(&var_id).and_then(|debug_var| self.types.get(&debug_var.debug_type_id))
    }

    pub fn drop_var(&mut self, var_id: DebugVarId) {
        self.frames.last_mut().expect("unexpected empty stack frames").1.remove(&var_id);
    }

    pub fn push_fn(&mut self, fn_id: DebugFnId) {
        self.frames.push((fn_id, HashMap::default()));
    }

    pub fn pop_fn(&mut self) {
        self.frames.pop();
    }
}
