use iter_extended::vecmap;
use noirc_printable_type::PrintableType;

use crate::hir_def::expr::*;

use super::ast::{Expression, Ident};
use super::Monomorphizer;

const DEBUG_MEMBER_ASSIGN_PREFIX: &str = "__debug_member_assign_";
const DEBUG_VAR_ID_ARG_SLOT: usize = 0;
const DEBUG_VALUE_ARG_SLOT: usize = 1;
const DEBUG_MEMBER_FIELD_INDEX_ARG_SLOT: usize = 2;

impl<'interner> Monomorphizer<'interner> {
    /// Try to patch instrumentation code inserted for debugging. This will
    /// record tracked variables and their types, and assign them an ID to use
    /// at runtime.
    pub(super) fn patch_debug_instrumentation_call(
        &mut self,
        call: &HirCallExpression,
        arguments: &mut [Expression],
    ) {
        let original_func = Box::new(self.expr(call.func));
        if let Expression::Ident(Ident { name, .. }) = original_func.as_ref() {
            if name == "__debug_var_assign" {
                self.patch_debug_var_assign(call, arguments);
            } else if name == "__debug_var_drop" {
                self.patch_debug_var_drop(call, arguments);
            } else if let Some(arity) = name.strip_prefix(DEBUG_MEMBER_ASSIGN_PREFIX) {
                let arity = arity.parse::<usize>().expect("failed to parse member assign arity");
                self.patch_debug_member_assign(call, arguments, arity);
            }
        }
    }

    /// Update instrumentation code inserted on variable assignment. We need to
    /// register the variable instance, its type and replace the temporary ID
    /// (fe_var_id) with the ID of the registration. Multiple registrations of
    /// the same variable are possible if using generic functions, hence the
    /// temporary ID created when injecting the instrumentation code can map to
    /// multiple IDs at runtime.
    fn patch_debug_var_assign(&mut self, call: &HirCallExpression, arguments: &mut [Expression]) {
        let hir_arguments = vecmap(&call.arguments, |id| self.interner.expression(id));
        let Some(HirExpression::Literal(HirLiteral::Integer(fe_var_id, _))) = hir_arguments.get(DEBUG_VAR_ID_ARG_SLOT) else {
            unreachable!("Missing FE var ID in __debug_var_assign call");
        };
        let Some(HirExpression::Ident(HirIdent { id, .. })) = hir_arguments.get(DEBUG_VALUE_ARG_SLOT) else {
            unreachable!("Missing value identifier in __debug_var_assign call");
        };

        // update variable assignments
        let var_def = self.interner.definition(*id);
        let var_type = self.interner.id_type(call.arguments[DEBUG_VALUE_ARG_SLOT]);
        let fe_var_id = fe_var_id.to_u128() as u32;
        let var_id = if var_def.name != "__debug_expr" {
            self.debug_types.insert_var(fe_var_id, &var_def.name, var_type)
        } else {
            self.debug_types.get_var_id(fe_var_id).unwrap()
        };
        let interned_var_id = self.intern_var_id(var_id, &call.location);
        arguments[DEBUG_VAR_ID_ARG_SLOT] = self.expr(interned_var_id);
    }

    /// Update instrumentation code for a variable being dropped out of scope.
    /// Given the fe_var_id we search for the last assigned runtime variable ID
    /// and replace it instead.
    fn patch_debug_var_drop(&mut self, call: &HirCallExpression, arguments: &mut [Expression]) {
        let hir_arguments = vecmap(&call.arguments, |id| self.interner.expression(id));
        let Some(HirExpression::Literal(HirLiteral::Integer(fe_var_id, _))) = hir_arguments.get(DEBUG_VAR_ID_ARG_SLOT) else {
            unreachable!("Missing FE var ID in __debug_var_drop call");
        };
        // update variable drops (ie. when the var goes out of scope)
        let fe_var_id = fe_var_id.to_u128() as u32;
        if let Some(var_id) = self.debug_types.get_var_id(fe_var_id) {
            let interned_var_id = self.intern_var_id(var_id, &call.location);
            arguments[DEBUG_VAR_ID_ARG_SLOT] = self.expr(interned_var_id);
        }
    }

    /// Update instrumentation code inserted when assigning to a member of an
    /// existing variable. Same as above for replacing the fe_var_id, but also
    /// we need to resolve the path and the type of the member being assigned.
    /// For this last part, we need to resolve the mapping from field names in
    /// structs to positions in the runtime tuple, since all structs are
    /// replaced by tuples during compilation.
    fn patch_debug_member_assign(
        &mut self,
        call: &HirCallExpression,
        arguments: &mut [Expression],
        arity: usize,
    ) {
        let hir_arguments = vecmap(&call.arguments, |id| self.interner.expression(id));
        let Some(HirExpression::Literal(HirLiteral::Integer(fe_var_id, _))) = hir_arguments.get(DEBUG_VAR_ID_ARG_SLOT) else {
            unreachable!("Missing FE var ID in __debug_member_assign call");
        };
        let Some(HirExpression::Ident(HirIdent { id, .. })) = hir_arguments.get(DEBUG_VALUE_ARG_SLOT) else {
            unreachable!("Missing value identifier in __debug_member_assign call");
        };
        // update variable member assignments
        let var_def_name = self.interner.definition(*id).name.clone();
        let var_type = self.interner.id_type(call.arguments[DEBUG_VALUE_ARG_SLOT]);
        let fe_var_id = fe_var_id.to_u128() as u32;

        let mut cursor_type = self
            .debug_types
            .get_type(fe_var_id)
            .unwrap_or_else(|| panic!("type not found for fe_var_id={fe_var_id}"))
            .clone();
        for i in 0..arity {
            if let Some(HirExpression::Literal(HirLiteral::Integer(fe_i, i_neg))) =
                hir_arguments.get(DEBUG_MEMBER_FIELD_INDEX_ARG_SLOT + i)
            {
                let mut index = fe_i.to_i128();
                if *i_neg {
                    index = -index;
                }
                if index < 0 {
                    let index = index.unsigned_abs();
                    let field_name = self
                        .debug_field_names
                        .get(&(index as u32))
                        .unwrap_or_else(|| panic!("field name not available for {i:?}"));
                    let field_i = (get_field(&cursor_type, field_name)
                        .unwrap_or_else(|| panic!("failed to find field_name: {field_name}"))
                        as i128)
                        .unsigned_abs();
                    cursor_type = element_type_at_index(&cursor_type, field_i as usize);
                    let index_id = self.interner.push_expr(HirExpression::Literal(
                        HirLiteral::Integer(field_i.into(), false),
                    ));
                    self.interner.push_expr_type(&index_id, crate::Type::FieldElement);
                    self.interner.push_expr_location(
                        index_id,
                        call.location.span,
                        call.location.file,
                    );
                    arguments[DEBUG_MEMBER_FIELD_INDEX_ARG_SLOT + i] = self.expr(index_id);
                } else {
                    cursor_type = element_type_at_index(&cursor_type, 0);
                }
            } else {
                cursor_type = element_type_at_index(&cursor_type, 0);
            }
        }

        let var_id = if &var_def_name != "__debug_expr" {
            self.debug_types.insert_var(fe_var_id, &var_def_name, var_type)
        } else {
            self.debug_types.get_var_id(fe_var_id).unwrap()
        };
        let interned_var_id = self.intern_var_id(var_id, &call.location);
        arguments[DEBUG_VAR_ID_ARG_SLOT] = self.expr(interned_var_id);
    }
}

fn get_field(ptype: &PrintableType, field_name: &str) -> Option<usize> {
    match ptype {
        PrintableType::Struct { fields, .. } => {
            fields.iter().position(|(name, _)| name == field_name)
        }
        PrintableType::Tuple { .. } | PrintableType::Array { .. } => {
            field_name.parse::<usize>().ok()
        }
        _ => None,
    }
}

fn element_type_at_index(ptype: &PrintableType, i: usize) -> PrintableType {
    match ptype {
        PrintableType::Array { length: _length, typ } => (**typ).clone(),
        PrintableType::Tuple { types } => types[i].clone(),
        PrintableType::Struct { name: _name, fields } => fields[i].1.clone(),
        PrintableType::String { length: _length } => PrintableType::UnsignedInteger { width: 8 },
        _ => {
            panic!["expected type with sub-fields, found terminal type"]
        }
    }
}
