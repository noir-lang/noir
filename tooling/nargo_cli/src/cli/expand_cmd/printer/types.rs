use noirc_frontend::{NamedGeneric, Type, TypeBinding, hir::def_map::ModuleDefId};

use super::ItemPrinter;

impl ItemPrinter<'_, '_> {
    pub(super) fn show_types_separated_by_comma(&mut self, types: &[Type]) {
        self.show_separated_by_comma(types, |this, typ| {
            this.show_type(typ);
        });
    }

    pub(super) fn show_type(&mut self, typ: &Type) {
        if self.self_type.as_ref() == Some(typ) {
            self.push_str("Self");
            return;
        }

        match typ {
            Type::Array(length, typ) => {
                self.push('[');
                self.show_type(typ);
                self.push_str("; ");
                self.show_type(length);
                self.push(']');
            }
            Type::Slice(typ) => {
                self.push('[');
                self.show_type(typ);
                self.push(']');
            }
            Type::FmtString(length, typ) => {
                self.push_str("fmtstr<");
                self.show_type(length);
                self.push_str(", ");
                self.show_type(typ);
                self.push('>');
            }
            Type::Tuple(types) => {
                let len = types.len();
                self.push('(');
                self.show_types_separated_by_comma(types);
                if len == 1 {
                    self.push(',');
                }
                self.push(')');
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
                    self.push_str("<");
                    self.show_types_separated_by_comma(generics);
                    self.push('>');
                }
            }
            Type::TypeVariable(type_variable) => match &*type_variable.borrow() {
                TypeBinding::Bound(typ) => {
                    self.show_type(typ);
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
                    self.show_type(typ);
                } else {
                    self.push_str(name);
                }
            }
            Type::CheckedCast { from: _, to } => {
                self.show_type(to);
            }
            Type::Function(args, ret, env, unconstrained) => {
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
            }
            Type::Reference(typ, mutable) => {
                if *mutable {
                    self.push_str("&mut ");
                } else {
                    self.push('&');
                }
                self.show_type(typ);
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
            Type::Unit
            | Type::Bool
            | Type::Integer(..)
            | Type::FieldElement
            | Type::String(_)
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
        | Type::Slice(..)
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
