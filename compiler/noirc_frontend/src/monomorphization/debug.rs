use acvm::acir::AcirField;
use iter_extended::vecmap;
use noirc_errors::debug_info::DebugVarId;
use noirc_errors::Location;
use noirc_printable_type::PrintableType;

use crate::debug::{SourceFieldId, SourceVarId};
use crate::hir_def::expr::*;
use crate::node_interner::ExprId;

use super::ast::{Expression, Ident};
use super::{MonomorphizationError, Monomorphizer};

const DEBUG_MEMBER_ASSIGN_PREFIX: &str = "__debug_member_assign_";
const DEBUG_VAR_ID_ARG_SLOT: usize = 0;
const DEBUG_VALUE_ARG_SLOT: usize = 1;
const DEBUG_MEMBER_FIELD_INDEX_ARG_SLOT: usize = 2;

impl From<u128> for SourceVarId {
    fn from(var_id: u128) -> Self {
        Self(var_id as u32)
    }
}

impl From<u128> for SourceFieldId {
    fn from(field_id: u128) -> Self {
        Self(field_id as u32)
    }
}

impl<'interner> Monomorphizer<'interner> {
    /// Patch instrumentation calls inserted for debugging. This will record
    /// tracked variables and their types, and assign them an ID to use at
    /// runtime. This ID is different from the source ID assigned at
    /// instrumentation time because at that point we haven't fully resolved the
    /// types for generic functions. So a single generic function may be
    /// instantiated multiple times with its tracked variables being of
    /// different types for each instance at runtime.
    pub(super) fn patch_debug_instrumentation_call(
        &mut self,
        call: &HirCallExpression,
        arguments: &mut [Expression],
    ) -> Result<(), MonomorphizationError> {
        let original_func = Box::new(self.expr(call.func)?);
        if let Expression::Ident(Ident { name, .. }) = original_func.as_ref() {
            if name == "__debug_var_assign" {
                self.patch_debug_var_assign(call, arguments)?;
            } else if name == "__debug_var_drop" {
                self.patch_debug_var_drop(call, arguments)?;
            } else if let Some(arity) = name.strip_prefix(DEBUG_MEMBER_ASSIGN_PREFIX) {
                let arity = arity.parse::<usize>().expect("failed to parse member assign arity");
                self.patch_debug_member_assign(call, arguments, arity)?;
            }
        }
        Ok(())
    }

    /// Update instrumentation code inserted on variable assignment. We need to
    /// register the variable instance, its type and replace the source_var_id
    /// with the ID of the registration. Multiple registrations of the same
    /// variable are possible if using generic functions, hence the temporary ID
    /// created when injecting the instrumentation code can map to multiple IDs
    /// at runtime.
    fn patch_debug_var_assign(
        &mut self,
        call: &HirCallExpression,
        arguments: &mut [Expression],
    ) -> Result<(), MonomorphizationError> {
        let hir_arguments = vecmap(&call.arguments, |id| self.interner.expression(id));
        let var_id_arg = hir_arguments.get(DEBUG_VAR_ID_ARG_SLOT);
        let Some(HirExpression::Literal(HirLiteral::Integer(source_var_id, _))) = var_id_arg else {
            unreachable!("Missing source_var_id in __debug_var_assign call");
        };

        // instantiate tracked variable for the value type and associate it with
        // the ID used by the injected instrumentation code
        let var_type = self.interner.id_type(call.arguments[DEBUG_VALUE_ARG_SLOT]);
        let source_var_id = source_var_id.to_u128().into();
        // then update the ID used for tracking at runtime
        let var_id = self.debug_type_tracker.insert_var(source_var_id, &var_type);
        let interned_var_id = self.intern_var_id(var_id, &call.location);
        arguments[DEBUG_VAR_ID_ARG_SLOT] = self.expr(interned_var_id)?;
        Ok(())
    }

    /// Update instrumentation code for a variable being dropped out of scope.
    /// Given the source_var_id we search for the last assigned debug var_id and
    /// replace it instead.
    fn patch_debug_var_drop(
        &mut self,
        call: &HirCallExpression,
        arguments: &mut [Expression],
    ) -> Result<(), MonomorphizationError> {
        let hir_arguments = vecmap(&call.arguments, |id| self.interner.expression(id));
        let var_id_arg = hir_arguments.get(DEBUG_VAR_ID_ARG_SLOT);
        let Some(HirExpression::Literal(HirLiteral::Integer(source_var_id, _))) = var_id_arg else {
            unreachable!("Missing source_var_id in __debug_var_drop call");
        };
        // update variable ID for tracked drops (ie. when the var goes out of scope)
        let source_var_id = source_var_id.to_u128().into();
        let var_id = self
            .debug_type_tracker
            .get_var_id(source_var_id)
            .unwrap_or_else(|| unreachable!("failed to find debug variable"));
        let interned_var_id = self.intern_var_id(var_id, &call.location);
        arguments[DEBUG_VAR_ID_ARG_SLOT] = self.expr(interned_var_id)?;
        Ok(())
    }

    /// Update instrumentation code inserted when assigning to a member of an
    /// existing variable. Same as above for replacing the source_var_id, but also
    /// we need to resolve the path and the type of the member being assigned.
    /// For this last part, we need to resolve the mapping from field names in
    /// structs to positions in the runtime tuple, since all structs are
    /// replaced by tuples during compilation.
    fn patch_debug_member_assign(
        &mut self,
        call: &HirCallExpression,
        arguments: &mut [Expression],
        arity: usize,
    ) -> Result<(), MonomorphizationError> {
        let hir_arguments = vecmap(&call.arguments, |id| self.interner.expression(id));
        let var_id_arg = hir_arguments.get(DEBUG_VAR_ID_ARG_SLOT);
        let Some(HirExpression::Literal(HirLiteral::Integer(source_var_id, _))) = var_id_arg else {
            unreachable!("Missing source_var_id in __debug_member_assign call");
        };
        // update variable member assignments
        let source_var_id = source_var_id.to_u128().into();

        let var_type = self
            .debug_type_tracker
            .get_type(source_var_id)
            .unwrap_or_else(|| panic!("type not found for {source_var_id:?}"))
            .clone();
        let mut cursor_type = &var_type;
        for i in 0..arity {
            if let Some(HirExpression::Literal(HirLiteral::Integer(fe_i, i_neg))) =
                hir_arguments.get(DEBUG_MEMBER_FIELD_INDEX_ARG_SLOT + i)
            {
                let index = fe_i.to_i128().unsigned_abs();
                if *i_neg {
                    // We use negative indices at instrumentation time to indicate
                    // and reference member accesses by name which cannot be
                    // resolved until we have a type. This strategy is also used
                    // for tuple member access because syntactically they are
                    // the same as named field accesses.
                    let field_index = self
                        .debug_type_tracker
                        .resolve_field_index(index.into(), cursor_type)
                        .unwrap_or_else(|| {
                            unreachable!("failed to resolve {i}-th member indirection on type {cursor_type:?}")
                        });

                    cursor_type = element_type_at_index(cursor_type, field_index);
                    let index_id = self.interner.push_expr(HirExpression::Literal(
                        HirLiteral::Integer(field_index.into(), false),
                    ));
                    self.interner.push_expr_type(index_id, crate::Type::FieldElement);
                    self.interner.push_expr_location(
                        index_id,
                        call.location.span,
                        call.location.file,
                    );
                    arguments[DEBUG_MEMBER_FIELD_INDEX_ARG_SLOT + i] = self.expr(index_id)?;
                } else {
                    // array/string element using constant index
                    cursor_type = element_type_at_index(cursor_type, index as usize);
                }
            } else {
                // array element using non-constant index
                cursor_type = element_type_at_index(cursor_type, 0);
            }
        }

        let var_id = self
            .debug_type_tracker
            .get_var_id(source_var_id)
            .unwrap_or_else(|| unreachable!("failed to find debug variable"));
        let interned_var_id = self.intern_var_id(var_id, &call.location);
        arguments[DEBUG_VAR_ID_ARG_SLOT] = self.expr(interned_var_id)?;
        Ok(())
    }

    fn intern_var_id(&mut self, var_id: DebugVarId, location: &Location) -> ExprId {
        let var_id_literal = HirLiteral::Integer((var_id.0 as u128).into(), false);
        let expr_id = self.interner.push_expr(HirExpression::Literal(var_id_literal));
        self.interner.push_expr_type(expr_id, crate::Type::FieldElement);
        self.interner.push_expr_location(expr_id, location.span, location.file);
        expr_id
    }
}

fn element_type_at_index(ptype: &PrintableType, i: usize) -> &PrintableType {
    match ptype {
        PrintableType::Array { length: _length, typ } => typ.as_ref(),
        PrintableType::Slice { typ } => typ.as_ref(),
        PrintableType::Tuple { types } => &types[i],
        PrintableType::Struct { name: _name, fields } => &fields[i].1,
        PrintableType::String { length: _length } => &PrintableType::UnsignedInteger { width: 8 },
        other => {
            panic!["expected type with sub-fields, found terminal type: {other:?}"]
        }
    }
}
