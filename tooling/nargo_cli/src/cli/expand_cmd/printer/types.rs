use std::collections::HashSet;

use noirc_frontend::{DataType, Type, TypeBinding, hir::def_map::ModuleDefId};

use super::Printer;

impl Printer<'_, '_, '_> {
    pub(super) fn show_types_separated_by_comma(&mut self, types: &[Type]) {
        for (index, typ) in types.iter().enumerate() {
            if index != 0 {
                self.push_str(", ");
            }
            self.show_type(typ);
        }
    }

    pub(super) fn show_type(&mut self, typ: &Type) {
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
                for (index, typ) in types.iter().enumerate() {
                    if index != 0 {
                        self.push_str(", ");
                    }
                    self.show_type(typ);
                }
                if len == 1 {
                    self.push(',');
                }
                self.push(')');
            }
            Type::DataType(data_type, generics) => {
                let data_type = data_type.borrow();
                let use_import = true;
                self.show_reference_to_module_def_id(ModuleDefId::TypeId(data_type.id), use_import);
                if !generics.is_empty() {
                    self.push_str("<");
                    for (index, generic) in generics.iter().enumerate() {
                        if index != 0 {
                            self.push_str(", ");
                        }
                        self.show_type(generic);
                    }
                    self.push('>');
                }
            }
            Type::Alias(type_alias, generics) => {
                let type_alias = type_alias.borrow();
                let use_import = true;
                self.show_reference_to_module_def_id(
                    ModuleDefId::TypeAliasId(type_alias.id),
                    use_import,
                );
                if !generics.is_empty() {
                    self.push_str("<");
                    for (index, generic) in generics.iter().enumerate() {
                        if index != 0 {
                            self.push_str(", ");
                        }
                        self.show_type(generic);
                    }
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
            Type::TraitAsType(..) => {
                panic!("Trait as type should not happen")
            }
            Type::NamedGeneric(_type_variable, name) => {
                self.push_str(name);
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
                for (index, arg) in args.iter().enumerate() {
                    if index != 0 {
                        self.push_str(", ");
                    }
                    self.show_type(arg);
                }
                self.push_str(") -> ");
                self.show_type(ret);
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
                self.push('(');
                self.show_type(lhs);
                self.push(' ');
                self.push_str(&op.to_string());
                self.push(' ');
                self.show_type(rhs);
                self.push(')');
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

    pub(super) fn type_only_mention_types_outside_current_crate(&self, typ: &Type) -> bool {
        match typ {
            Type::Array(length, typ) => {
                self.type_only_mention_types_outside_current_crate(length)
                    && self.type_only_mention_types_outside_current_crate(typ)
            }
            Type::Slice(typ) => self.type_only_mention_types_outside_current_crate(typ),
            Type::FmtString(length, typ) => {
                self.type_only_mention_types_outside_current_crate(length)
                    && self.type_only_mention_types_outside_current_crate(typ)
            }
            Type::Tuple(types) => {
                types.iter().all(|typ| self.type_only_mention_types_outside_current_crate(typ))
            }
            Type::DataType(data_type, generics) => {
                let data_type = data_type.borrow();
                data_type.id.krate() != self.crate_id
                    && generics
                        .iter()
                        .all(|typ| self.type_only_mention_types_outside_current_crate(typ))
            }
            Type::Alias(_type_alias, generics) => {
                generics.iter().all(|typ| self.type_only_mention_types_outside_current_crate(typ))
            }
            Type::TraitAsType(trait_id, _, generics) => {
                let trait_ = self.interner.get_trait(*trait_id);
                trait_.id.0.krate != self.crate_id
                    && generics
                        .ordered
                        .iter()
                        .all(|typ| self.type_only_mention_types_outside_current_crate(typ))
                    && generics.named.iter().all(|named_type| {
                        self.type_only_mention_types_outside_current_crate(&named_type.typ)
                    })
            }
            Type::CheckedCast { from, to: _ } => {
                self.type_only_mention_types_outside_current_crate(from)
            }
            Type::Function(args, ret, env, _) => {
                args.iter().all(|typ| self.type_only_mention_types_outside_current_crate(typ))
                    && self.type_only_mention_types_outside_current_crate(ret)
                    && self.type_only_mention_types_outside_current_crate(env)
            }
            Type::Reference(typ, _) => self.type_only_mention_types_outside_current_crate(typ),
            Type::Forall(_, typ) => self.type_only_mention_types_outside_current_crate(typ),
            Type::InfixExpr(lhs, _, rhs, _) => {
                self.type_only_mention_types_outside_current_crate(lhs)
                    && self.type_only_mention_types_outside_current_crate(rhs)
            }
            Type::Unit
            | Type::Bool
            | Type::Integer(..)
            | Type::FieldElement
            | Type::String(_)
            | Type::Quoted(_)
            | Type::Constant(..)
            | Type::TypeVariable(..)
            | Type::NamedGeneric(..)
            | Type::Error => true,
        }
    }
}

pub(super) fn gather_named_type_vars(typ: &Type, type_vars: &mut HashSet<String>) {
    match typ {
        Type::Array(length, typ) => {
            gather_named_type_vars(length, type_vars);
            gather_named_type_vars(typ, type_vars);
        }
        Type::Slice(typ) => {
            gather_named_type_vars(typ, type_vars);
        }
        Type::FmtString(length, typ) => {
            gather_named_type_vars(length, type_vars);
            gather_named_type_vars(typ, type_vars);
        }
        Type::Tuple(types) => {
            for typ in types {
                gather_named_type_vars(typ, type_vars);
            }
        }
        Type::DataType(_, generics) | Type::Alias(_, generics) => {
            for typ in generics {
                gather_named_type_vars(typ, type_vars);
            }
        }
        Type::TraitAsType(_, _, trait_generics) => {
            for typ in &trait_generics.ordered {
                gather_named_type_vars(typ, type_vars);
            }
            for named_type in &trait_generics.named {
                gather_named_type_vars(&named_type.typ, type_vars);
            }
        }
        Type::NamedGeneric(_, name) => {
            type_vars.insert(name.to_string());
        }
        Type::CheckedCast { from, to: _ } => {
            gather_named_type_vars(from, type_vars);
        }
        Type::Function(args, ret, env, _) => {
            for typ in args {
                gather_named_type_vars(typ, type_vars);
            }
            gather_named_type_vars(ret, type_vars);
            gather_named_type_vars(env, type_vars);
        }
        Type::Reference(typ, _) => {
            gather_named_type_vars(typ, type_vars);
        }
        Type::Forall(_, typ) => {
            gather_named_type_vars(typ, type_vars);
        }
        Type::InfixExpr(lhs, _, rhs, _) => {
            gather_named_type_vars(lhs, type_vars);
            gather_named_type_vars(rhs, type_vars);
        }
        Type::Unit
        | Type::FieldElement
        | Type::Integer(..)
        | Type::Bool
        | Type::String(_)
        | Type::Quoted(_)
        | Type::Constant(..)
        | Type::TypeVariable(_)
        | Type::Error => (),
    }
}

pub(super) fn type_mentions_data_type(typ: &Type, data_type: &DataType) -> bool {
    match typ {
        Type::Array(length, typ) => {
            type_mentions_data_type(length, data_type) && type_mentions_data_type(typ, data_type)
        }
        Type::Slice(typ) => type_mentions_data_type(typ, data_type),
        Type::FmtString(length, typ) => {
            type_mentions_data_type(length, data_type) || type_mentions_data_type(typ, data_type)
        }
        Type::Tuple(types) => types.iter().any(|typ| type_mentions_data_type(typ, data_type)),
        Type::DataType(other_data_type, generics) => {
            let other_data_type = other_data_type.borrow();
            data_type.id == other_data_type.id
                || generics.iter().any(|typ| type_mentions_data_type(typ, data_type))
        }
        Type::Alias(_type_alias, generics) => {
            generics.iter().any(|typ| type_mentions_data_type(typ, data_type))
        }
        Type::TraitAsType(_, _, generics) => {
            generics.ordered.iter().any(|typ| type_mentions_data_type(typ, data_type))
                || generics
                    .named
                    .iter()
                    .any(|named_type| type_mentions_data_type(&named_type.typ, data_type))
        }
        Type::CheckedCast { from, to: _ } => type_mentions_data_type(from, data_type),
        Type::Function(args, ret, env, _) => {
            args.iter().any(|typ| type_mentions_data_type(typ, data_type))
                || type_mentions_data_type(ret, data_type)
                || type_mentions_data_type(env, data_type)
        }
        Type::Reference(typ, _) => type_mentions_data_type(typ, data_type),
        Type::Forall(_, typ) => type_mentions_data_type(typ, data_type),
        Type::InfixExpr(lhs, _, rhs, _) => {
            type_mentions_data_type(lhs, data_type) || type_mentions_data_type(rhs, data_type)
        }
        Type::Unit
        | Type::Bool
        | Type::Integer(..)
        | Type::FieldElement
        | Type::String(_)
        | Type::Quoted(_)
        | Type::Constant(..)
        | Type::TypeVariable(..)
        | Type::NamedGeneric(..)
        | Type::Error => true,
    }
}
