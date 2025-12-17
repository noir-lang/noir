use crate::{NamedGeneric, Type, TypeBinding, hir::def_map::ModuleDefId};

use crate::hir::printer::ItemPrinter;

impl ItemPrinter<'_, '_> {
    pub(crate) fn show_types_separated_by_comma(&mut self, types: &[Type]) {
        self.show_separated_by_comma(types, |this, typ| {
            this.show_type(typ);
        });
    }

    pub(crate) fn show_type(&mut self, typ: &Type) {
        self.show_type_impl(typ, false /* as expression */);
    }

    pub(super) fn show_type_as_expression(&mut self, typ: &Type) {
        self.show_type_impl(typ, true /* as expression */);
    }

    fn show_type_impl(&mut self, typ: &Type, as_expression: bool) {
        if self.self_type.as_ref() == Some(typ) {
            self.push_str("Self");
            return;
        }

        match typ {
            Type::Array(length, typ) => {
                if as_expression {
                    self.push('<');
                }

                self.push('[');
                self.show_type(typ);
                self.push_str("; ");
                self.show_type(length);
                self.push(']');

                if as_expression {
                    self.push('>');
                }
            }
            Type::List(typ) => {
                if as_expression {
                    self.push('<');
                }

                self.push('[');
                self.show_type(typ);
                self.push(']');

                if as_expression {
                    self.push('>');
                }
            }
            Type::String(length) => {
                self.push_str("str");
                if as_expression {
                    self.push_str("::");
                }
                self.push('<');
                self.show_type(length);
                self.push('>');
            }
            Type::FmtString(length, typ) => {
                self.push_str("fmtstr");
                if as_expression {
                    self.push_str("::");
                }
                self.push('<');
                self.show_type(length);
                self.push_str(", ");
                self.show_type(typ);
                self.push('>');
            }
            Type::Tuple(types) => {
                if as_expression {
                    self.push('<');
                }

                let len = types.len();
                self.push('(');
                self.show_types_separated_by_comma(types);
                if len == 1 {
                    self.push(',');
                }
                self.push(')');

                if as_expression {
                    self.push('>');
                }
            }
            Type::DataType(data_type, generics) => {
                let data_type = data_type.borrow();
                let use_import = true;
                self.show_reference_to_module_def_id(
                    ModuleDefId::TypeId(data_type.id),
                    data_type.visibility,
                    use_import,
                );
                if !generics.is_empty() {
                    if as_expression {
                        self.push_str("::");
                    }
                    self.push_str("<");
                    self.show_types_separated_by_comma(generics);
                    self.push('>');
                }
            }
            Type::Alias(type_alias, generics) => {
                let type_alias = type_alias.borrow();
                let use_import = true;
                self.show_reference_to_module_def_id(
                    ModuleDefId::TypeAliasId(type_alias.id),
                    type_alias.visibility,
                    use_import,
                );
                if !generics.is_empty() {
                    if as_expression {
                        self.push_str("::");
                    }
                    self.push_str("<");
                    self.show_types_separated_by_comma(generics);
                    self.push('>');
                }
            }
            Type::TypeVariable(type_variable) => match &*type_variable.borrow() {
                TypeBinding::Bound(typ) => {
                    self.show_type_impl(typ, as_expression);
                }
                TypeBinding::Unbound(..) => {
                    self.push('_');
                }
            },
            Type::TraitAsType(trait_id, _, generics) => {
                let trait_ = self.interner.get_trait(*trait_id);
                self.push_str("impl ");
                self.push_str(trait_.name.as_str());
                self.show_trait_generics(generics);
            }
            Type::NamedGeneric(NamedGeneric { name, type_var, .. }) => {
                if let TypeBinding::Bound(typ) = &*type_var.borrow() {
                    self.show_type_impl(typ, as_expression);
                } else {
                    self.push_str(name);
                }
            }
            Type::CheckedCast { from: _, to } => {
                self.show_type_impl(to, as_expression);
            }
            Type::Function(args, ret, env, unconstrained) => {
                if as_expression {
                    self.push('<');
                }

                if *unconstrained {
                    self.push_str("unconstrained ");
                }
                self.push_str("fn");
                if **env != Type::Unit {
                    self.push('[');
                    self.show_type(env);
                    self.push(']');
                }
                self.push('(');
                self.show_types_separated_by_comma(args);
                self.push(')');
                if **ret != Type::Unit {
                    self.push_str(" -> ");
                    self.show_type(ret);
                }

                if as_expression {
                    self.push('>');
                }
            }
            Type::Reference(typ, mutable) => {
                if *mutable {
                    self.push_str("&mut ");
                } else {
                    self.push('&');
                }
                self.show_type_impl(typ, as_expression);
            }
            Type::Forall(..) => {
                panic!("Should not need to print Type::Forall")
            }
            Type::Constant(field_element, _) => {
                self.push_str(&field_element.to_string());
            }
            Type::InfixExpr(lhs, op, rhs, _) => {
                self.show_type_maybe_in_parentheses(lhs);
                self.push(' ');
                self.push_str(&op.to_string());
                self.push(' ');
                self.show_type_maybe_in_parentheses(rhs);
            }
            Type::Unit => {
                if as_expression {
                    self.push_str("<()>");
                } else {
                    self.push_str("()");
                }
            }
            Type::Bool
            | Type::Integer(..)
            | Type::FieldElement
            | Type::Quoted(..)
            | Type::Error => self.push_str(&typ.to_string()),
        }
    }

    fn show_type_maybe_in_parentheses(&mut self, typ: &Type) {
        if type_needs_parentheses(typ) {
            self.push('(');
            self.show_type(typ);
            self.push(')');
        } else {
            self.show_type(typ);
        }
    }
}

fn type_needs_parentheses(typ: &Type) -> bool {
    match typ {
        Type::InfixExpr(..) | Type::Function(..) | Type::TraitAsType(..) => true,
        Type::TypeVariable(type_variable) => match &*type_variable.borrow() {
            TypeBinding::Bound(typ) => type_needs_parentheses(typ),
            TypeBinding::Unbound(..) => false,
        },
        Type::CheckedCast { from: _, to } => type_needs_parentheses(to),
        Type::FieldElement
        | Type::Array(..)
        | Type::List(..)
        | Type::Integer(..)
        | Type::Bool
        | Type::String(..)
        | Type::FmtString(..)
        | Type::Unit
        | Type::Tuple(..)
        | Type::DataType(..)
        | Type::Alias(..)
        | Type::NamedGeneric(..)
        | Type::Reference(..)
        | Type::Forall(..)
        | Type::Constant(..)
        | Type::Quoted(..)
        | Type::Error => false,
    }
}
