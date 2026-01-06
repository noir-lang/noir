use noirc_errors::Location;

use crate::{NamedGeneric, Type, TypeBinding, ast::Ident};

/// An type incorrectly used as a program input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidType {
    Primitive(Type),
    Enum(Type),
    EmptyArray(Type),
    EmptyString(Type),
    Alias { alias_name: Ident, invalid_type: Box<InvalidType> },
    StructField { struct_name: Ident, field_name: Ident, invalid_type: Box<InvalidType> },
}

impl Type {
    /// Returns this type, or a nested one, that cannot be used as a parameter to `main`
    /// or a contract function.
    /// This is only Some for unsized types like vectors or vectors that do not make sense
    /// as a program input such as named generics or mutable references.
    ///
    /// This function should match the same check done in `create_value_from_type` in acir_gen.
    /// If this function does not catch a case where a type should be valid, it will later lead to a
    /// panic in that function instead of a user-facing compiler error message.
    ///
    /// Returns `None` if this type and its nested types are all valid program inputs.
    pub(crate) fn program_input_validity(&self, allow_empty_arrays: bool) -> Option<InvalidType> {
        match self {
            // Type::Error is allowed as usual since it indicates an error was already issued and
            // we don't need to issue further errors about this likely unresolved type
            // TypeVariable and Generic are allowed here too as they can only result from
            // generics being declared on the function itself, but we produce a different error in that case.
            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Constant(_, _)
            | Type::TypeVariable(_)
            | Type::NamedGeneric(_)
            | Type::Error => None,

            Type::Unit
            | Type::FmtString(_, _)
            | Type::Function(_, _, _, _)
            | Type::Reference(..)
            | Type::Forall(_, _)
            | Type::Quoted(_)
            | Type::Vector(_)
            | Type::TraitAsType(..) => Some(InvalidType::Primitive(self.clone())),

            Type::CheckedCast { to, .. } => to.program_input_validity(allow_empty_arrays),

            Type::Alias(alias, generics) => {
                let alias = alias.borrow();
                if let Some(invalid_type) =
                    alias.get_type(generics).program_input_validity(allow_empty_arrays)
                {
                    let alias_name = alias.name.clone();
                    Some(InvalidType::Alias { alias_name, invalid_type: Box::new(invalid_type) })
                } else {
                    None
                }
            }

            Type::Array(length, element) => {
                if !length_is_valid_for_entry_point(length, allow_empty_arrays) {
                    Some(InvalidType::Primitive(self.clone()))
                } else {
                    length
                        .program_input_validity(allow_empty_arrays)
                        .or_else(|| element.program_input_validity(allow_empty_arrays))
                }
            }
            Type::String(length) => {
                if !length_is_valid_for_entry_point(length, allow_empty_arrays) {
                    Some(InvalidType::EmptyString(self.clone()))
                } else {
                    length.program_input_validity(allow_empty_arrays)
                }
            }
            Type::Tuple(elements) => {
                for element in elements {
                    if let Some(invalid_type) = element.program_input_validity(allow_empty_arrays) {
                        return Some(invalid_type);
                    }
                }
                None
            }
            Type::DataType(definition, generics) => {
                let definition = definition.borrow();

                if let Some(fields) = definition.get_fields(generics) {
                    for (field_name, field, _) in fields {
                        if let Some(invalid_type) = field.program_input_validity(allow_empty_arrays)
                        {
                            let struct_name = definition.name.clone();
                            let mut fields_raw = definition.fields_raw().unwrap().iter();
                            let field = fields_raw.find(|field| field.name.as_str() == field_name);
                            return Some(InvalidType::StructField {
                                struct_name,
                                field_name: field.unwrap().name.clone(),
                                invalid_type: Box::new(invalid_type),
                            });
                        }
                    }
                    None
                } else {
                    // Arbitrarily disallow enums from program input, though we may support them later
                    Some(InvalidType::Enum(self.clone()))
                }
            }

            Type::InfixExpr(lhs, _, rhs, _) => lhs
                .program_input_validity(allow_empty_arrays)
                .or_else(|| rhs.program_input_validity(allow_empty_arrays)),
        }
    }

    /// Returns this type, or a nested one, if this type can be used as a parameter to an ACIR
    /// function that is not `main` or a contract function.
    /// This encapsulates functions for which we may not want to inline during compilation.
    ///
    /// The inputs allowed for a function entry point differ from those allowed as input to a program as there are
    /// certain types which through compilation we know what their size should be.
    /// This includes types such as numeric generics.
    pub(crate) fn non_inlined_function_input_validity(&self) -> Option<InvalidType> {
        match self {
            // Type::Error is allowed as usual since it indicates an error was already issued and
            // we don't need to issue further errors about this likely unresolved type
            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Constant(_, _)
            | Type::TypeVariable(_)
            | Type::NamedGeneric(_)
            | Type::InfixExpr(..)
            | Type::Error => None,

            Type::Unit
            | Type::FmtString(_, _)
            // To enable this we would need to determine the size of the closure outputs at compile-time.
            // This is possible as long as the output size is not dependent upon a witness condition.
            | Type::Function(_, _, _, _)
            | Type::Vector(_)
            | Type::Reference(..)
            | Type::Forall(_, _)
            // TODO: probably can allow code as it is all compile time
            | Type::Quoted(_)
            | Type::TraitAsType(..) => Some(InvalidType::Primitive(self.clone())),

            Type::CheckedCast { to, .. } => to.non_inlined_function_input_validity(),

            Type::Alias(alias, generics) => {
                let alias = alias.borrow();
                if let Some(invalid_type) = alias.get_type(generics).non_inlined_function_input_validity() {
                    let alias_name = alias.name.clone();
                    Some(InvalidType::Alias { alias_name, invalid_type: Box::new(invalid_type) })
                } else {
                    None
                }
            }

            Type::Array(length, element) => {
                length.non_inlined_function_input_validity().or_else(|| element.non_inlined_function_input_validity())
            }
            Type::String(length) => length.non_inlined_function_input_validity(),
            Type::Tuple(elements) => {
                for element in elements {
                    if let Some(invalid_type) = element.non_inlined_function_input_validity() {
                        return Some(invalid_type);
                    }
                }
                None
            },
            Type::DataType(definition, generics) => {
                                let definition = definition.borrow();

                if let Some(fields) = definition.get_fields(generics) {
                    for (field_name, field, _) in fields {
                        if let Some(invalid_type) = field.non_inlined_function_input_validity() {
                            let struct_name = definition.name.clone();
                            let mut fields_raw = definition.fields_raw().unwrap().iter();
                            let field = fields_raw.find(|field| field.name.as_str() == field_name);
                            return Some(InvalidType::StructField {
                                struct_name,
                                field_name: field.unwrap().name.clone(),
                                invalid_type: Box::new(invalid_type),
                            });
                        }
                    }
                    None
                } else {
                    Some(InvalidType::Enum(self.clone()))
                }
            }
        }
    }

    /// Returns true if a value of this type can safely pass between constrained and
    /// unconstrained functions (and vice-versa).
    pub(crate) fn is_valid_for_unconstrained_boundary(&self) -> bool {
        match self {
            Type::FieldElement
            | Type::Integer(_, _)
            | Type::Bool
            | Type::Unit
            | Type::Constant(_, _)
            | Type::Vector(_)
            | Type::Function(_, _, _, _)
            | Type::FmtString(_, _)
            | Type::InfixExpr(..)
            | Type::Error => true,

            Type::TypeVariable(type_var) | Type::NamedGeneric(NamedGeneric { type_var, .. }) => {
                if let TypeBinding::Bound(typ) = &*type_var.borrow() {
                    typ.is_valid_for_unconstrained_boundary()
                } else {
                    true
                }
            }

            Type::CheckedCast { to, .. } => to.is_valid_for_unconstrained_boundary(),

            // Quoted objects only exist at compile-time where the only execution
            // environment is the interpreter. In this environment, they are valid.
            Type::Quoted(_) => true,

            Type::Reference(..) | Type::Forall(_, _) | Type::TraitAsType(..) => false,

            Type::Alias(alias, generics) => {
                let alias = alias.borrow();
                alias.get_type(generics).is_valid_for_unconstrained_boundary()
            }

            Type::Array(length, element) => {
                length.is_valid_for_unconstrained_boundary()
                    && element.is_valid_for_unconstrained_boundary()
            }
            Type::String(length) => length.is_valid_for_unconstrained_boundary(),
            Type::Tuple(elements) => {
                elements.iter().all(|elem| elem.is_valid_for_unconstrained_boundary())
            }
            Type::DataType(definition, generics) => {
                if let Some(fields) = definition.borrow().get_fields(generics) {
                    fields
                        .into_iter()
                        .all(|(_, field, _)| field.is_valid_for_unconstrained_boundary())
                } else {
                    false
                }
            }
        }
    }
}

pub(crate) fn length_is_valid_for_entry_point(length: &Type, allow_empty: bool) -> bool {
    match length.evaluate_to_u32(Location::dummy()) {
        Ok(0) => allow_empty, // Zero is invalid unless allow_empty
        Ok(_) => true,        // Positive is always valid
        Err(_) => false,      // Failed to evaluate (like -1) is invalid
    }
}
