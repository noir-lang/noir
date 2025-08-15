use crate::{
    debug::{DebugInstrumenter, SourceFieldId, SourceVarId},
    hir_def::types::Type,
};
use noirc_errors::debug_info::{
    DebugFnId, DebugFunction, DebugFunctions, DebugTypeId, DebugTypes, DebugVarId, DebugVariable,
    DebugVariables,
};
use noirc_printable_type::PrintableType;
use std::collections::HashMap;

/// We keep a collection of the debug variables and their types in this
/// structure. The source_var_id refers to the ID given by the debug
/// instrumenter. This variable does not have a type yet and hence it
/// can be instantiated for multiple types if it's in the context of a generic
/// variable. The var_id refers to the ID of the instantiated variable which
/// will have a valid type.
#[derive(Debug, Clone, Default)]
pub struct DebugTypeTracker {
    // Variable names collected during instrumentation injection
    source_variables: HashMap<SourceVarId, String>,

    // Field names used for member access collected during instrumentation injection
    source_field_names: HashMap<SourceFieldId, String>,

    // Current instances of tracked variables from the ID given during
    // instrumentation. The tracked var_id will change for each source_var_id
    // when compiling generic functions.
    source_to_debug_vars: HashMap<SourceVarId, DebugVarId>,

    // All instances of tracked variables
    variables: HashMap<DebugVarId, (SourceVarId, DebugTypeId)>,

    // Function metadata collected during instrumentation injection
    functions: HashMap<DebugFnId, DebugFunction>,

    // Types of tracked variables and functions
    types: HashMap<DebugTypeId, PrintableType>,
    types_reverse: HashMap<PrintableType, DebugTypeId>,

    next_var_id: u32,
    next_type_id: u32,
}

impl DebugTypeTracker {
    pub fn build_from_debug_instrumenter(instrumenter: &DebugInstrumenter) -> Self {
        DebugTypeTracker {
            source_variables: instrumenter.variables.clone(),
            source_field_names: instrumenter.field_names.clone(),
            functions: instrumenter.functions.clone(),
            ..DebugTypeTracker::default()
        }
    }

    pub fn extract_vars_and_types(&self) -> (DebugVariables, DebugFunctions, DebugTypes) {
        let debug_variables = self
            .variables
            .clone()
            .into_iter()
            .map(|(var_id, (source_var_id, type_id))| {
                let var_name =
                    self.source_variables.get(&source_var_id).cloned().unwrap_or_else(|| {
                        unreachable!("failed to retrieve variable name for {source_var_id:?}");
                    });
                (var_id, DebugVariable { name: var_name, debug_type_id: type_id })
            })
            .collect();

        let debug_functions = self.functions.clone().into_iter().collect();
        let debug_types = self.types.clone().into_iter().collect();

        (debug_variables, debug_functions, debug_types)
    }

    pub fn resolve_field_index(
        &self,
        field_id: SourceFieldId,
        cursor_type: &PrintableType,
    ) -> Option<usize> {
        self.source_field_names
            .get(&field_id)
            .and_then(|field_name| get_field(cursor_type, field_name))
    }

    fn insert_type(&mut self, the_type: &Type) -> DebugTypeId {
        let printable_type: PrintableType = the_type.follow_bindings().into();
        self.types_reverse.get(&printable_type).copied().unwrap_or_else(|| {
            let type_id = DebugTypeId(self.next_type_id);
            self.next_type_id += 1;
            self.types_reverse.insert(printable_type.clone(), type_id);
            self.types.insert(type_id, printable_type);
            type_id
        })
    }

    pub fn insert_var(&mut self, source_var_id: SourceVarId, var_type: &Type) -> DebugVarId {
        if !self.source_variables.contains_key(&source_var_id) {
            unreachable!("cannot find source debug variable {source_var_id:?}");
        }

        let type_id = self.insert_type(var_type);

        // check if we need to instantiate the var with a new type
        let existing_var_id = self.source_to_debug_vars.get(&source_var_id).and_then(|var_id| {
            let (_, existing_type_id) = self.variables.get(var_id).unwrap();
            (*existing_type_id == type_id).then_some(var_id)
        });
        if let Some(var_id) = existing_var_id {
            *var_id
        } else {
            let var_id = DebugVarId(self.next_var_id);
            self.next_var_id += 1;
            self.variables.insert(var_id, (source_var_id, type_id));
            self.source_to_debug_vars.insert(source_var_id, var_id);
            var_id
        }
    }

    pub fn get_var_id(&self, source_var_id: SourceVarId) -> Option<DebugVarId> {
        self.source_to_debug_vars.get(&source_var_id).copied()
    }

    pub fn get_type(&self, source_var_id: SourceVarId) -> Option<&PrintableType> {
        self.source_to_debug_vars
            .get(&source_var_id)
            .and_then(|var_id| self.variables.get(var_id))
            .and_then(|(_, type_id)| self.types.get(type_id))
    }
}

fn get_field(printable_type: &PrintableType, field_name: &str) -> Option<usize> {
    match printable_type {
        PrintableType::Struct { fields, .. } => {
            fields.iter().position(|(name, _)| name == field_name)
        }
        PrintableType::Tuple { .. } | PrintableType::Array { .. } => {
            field_name.parse::<usize>().ok()
        }
        _ => None,
    }
}
