//! If the cursor is at the end of a function parameter name, suggest parameter names (and their types)
//! that exists in the same module, impl or trait.
use std::collections::HashSet;

use noirc_frontend::{
    ParsedModule,
    ast::{NoirFunction, NoirTrait, Pattern, TraitItem, TypeImpl, UnresolvedTypeData},
    parser::ItemKind,
};

use crate::requests::completion::{NodeFinder, name_matches, variable_completion_item};

impl NodeFinder<'_> {
    pub(super) fn try_complete_function_param_in_parsed_module(
        &mut self,
        parsed_module: &ParsedModule,
    ) -> bool {
        let functions = parsed_module.items.iter().filter_map(|item| {
            if let ItemKind::Function(function) = &item.kind { Some(function) } else { None }
        });

        let function_and_name =
            find_function_and_parameter_name_at_byte_index(functions.clone(), self.byte_index);
        let Some((function, name)) = function_and_name else {
            return false;
        };

        let names_to_exclude = names_to_exclude(function, name);

        self.suggest_function_parameters(functions, name, names_to_exclude);

        true
    }

    pub(super) fn try_complete_function_param_in_type_impl(
        &mut self,
        type_impl: &TypeImpl,
    ) -> bool {
        let functions =
            type_impl.methods.iter().map(|(documented_method, _)| &documented_method.item);

        let function_and_name =
            find_function_and_parameter_name_at_byte_index(functions.clone(), self.byte_index);
        let Some((function, name)) = function_and_name else {
            return false;
        };

        let names_to_exclude = names_to_exclude(function, name);

        self.suggest_function_parameters(functions, name, names_to_exclude);

        true
    }

    pub(super) fn try_complete_function_param_in_trait(&mut self, trait_: &NoirTrait) -> bool {
        // Since NoirTrait doesn't hold `NoirFunction`s we have to repeat a bit the code here.
        let parameters_and_name = trait_.items.iter().find_map(|documented_item| {
            if let TraitItem::Function { parameters, .. } = &documented_item.item {
                for (name, _typ) in parameters {
                    if self.byte_index == name.span().end() as usize {
                        return Some((parameters, name.as_str()));
                    }
                }
            }
            None
        });
        let Some((parameters, name)) = parameters_and_name else {
            return false;
        };

        let mut names_to_exclude = HashSet::new();
        for (ident, _) in parameters {
            if ident.as_str() != name {
                names_to_exclude.insert(ident.to_string());
            }
        }

        let mut suggested = HashSet::new();
        for documented_item in &trait_.items {
            if let TraitItem::Function { parameters, .. } = &documented_item.item {
                for (ident, typ) in parameters {
                    if matches!(typ.typ, UnresolvedTypeData::Error) {
                        continue;
                    };
                    let param_name = ident.as_str();
                    if names_to_exclude.contains(param_name) {
                        continue;
                    }

                    if name_matches(param_name, name) {
                        let label = format!("{param_name}: {typ}");
                        if suggested.insert(label.clone()) {
                            let item = variable_completion_item(label, None);
                            self.completion_items.push(item);
                        }
                    }
                }
            }
        }

        true
    }

    fn suggest_function_parameters<'a>(
        &mut self,
        functions: impl Iterator<Item = &'a NoirFunction>,
        name: &str,
        names_to_exclude: HashSet<String>,
    ) {
        let mut suggested = HashSet::new();
        for function in functions {
            for parameter in function.parameters() {
                let Pattern::Identifier(ident) = &parameter.pattern else {
                    continue;
                };
                if matches!(parameter.typ.typ, UnresolvedTypeData::Error) {
                    continue;
                };
                let param_name = ident.as_str();
                if names_to_exclude.contains(param_name) {
                    continue;
                }

                if name_matches(param_name, name) {
                    let label = format!("{param_name}: {}", parameter.typ);
                    if suggested.insert(label.clone()) {
                        let item = variable_completion_item(label, None);
                        self.completion_items.push(item);
                    }
                }
            }
        }
    }
}

/// Tries to find a function parameter inside `functions` that is being autocompleted.
/// Returns that function together with the parameter name, if found.
fn find_function_and_parameter_name_at_byte_index<'a>(
    mut functions: impl Iterator<Item = &'a NoirFunction>,
    byte_index: usize,
) -> Option<(&'a NoirFunction, &'a str)> {
    functions.find_map(|function| {
        for parameter in function.parameters() {
            let Pattern::Identifier(ident) = &parameter.pattern else {
                return None;
            };
            if byte_index == ident.span().end() as usize {
                return Some((function, ident.as_str()));
            }
        }
        None
    })
}

// Don't suggest names of parameters that already exist in the given function,
// unless it's the name currently being completed.
fn names_to_exclude(function: &NoirFunction, name: &str) -> HashSet<String> {
    let mut names_to_exclude = HashSet::new();
    for parameter in function.parameters() {
        let Pattern::Identifier(ident) = &parameter.pattern else {
            continue;
        };
        if ident.as_str() != name {
            names_to_exclude.insert(ident.to_string());
        }
    }
    names_to_exclude
}
