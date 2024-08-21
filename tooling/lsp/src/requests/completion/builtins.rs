use lsp_types::CompletionItemKind;
use noirc_frontend::token::Keyword;
use strum::IntoEnumIterator;

use super::{
    completion_items::{simple_completion_item, snippet_completion_item},
    name_matches, NodeFinder,
};

impl<'a> NodeFinder<'a> {
    pub(super) fn builtin_functions_completion(&mut self, prefix: &str) {
        for keyword in Keyword::iter() {
            if let Some(func) = keyword_builtin_function(&keyword) {
                if name_matches(func.name, prefix) {
                    self.completion_items.push(snippet_completion_item(
                        format!("{}(…)", func.name),
                        CompletionItemKind::FUNCTION,
                        format!("{}({})", func.name, func.parameters),
                        Some(func.description.to_string()),
                    ));
                }
            }
        }
    }

    pub(super) fn builtin_values_completion(&mut self, prefix: &str) {
        for keyword in ["false", "true"] {
            if name_matches(keyword, prefix) {
                self.completion_items.push(simple_completion_item(
                    keyword,
                    CompletionItemKind::KEYWORD,
                    Some("bool".to_string()),
                ));
            }
        }
    }

    pub(super) fn builtin_types_completion(&mut self, prefix: &str) {
        for keyword in Keyword::iter() {
            if let Some(typ) = keyword_builtin_type(&keyword) {
                if name_matches(typ, prefix) {
                    self.completion_items.push(simple_completion_item(
                        typ,
                        CompletionItemKind::STRUCT,
                        Some(typ.to_string()),
                    ));
                }
            }
        }

        for typ in builtin_integer_types() {
            if name_matches(typ, prefix) {
                self.completion_items.push(simple_completion_item(
                    typ,
                    CompletionItemKind::STRUCT,
                    Some(typ.to_string()),
                ));
            }
        }
    }
}

pub(super) fn builtin_integer_types() -> [&'static str; 8] {
    ["i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64"]
}

/// If a keyword corresponds to a built-in type, returns that type's name.
pub(super) fn keyword_builtin_type(keyword: &Keyword) -> Option<&'static str> {
    match keyword {
        Keyword::Bool => Some("bool"),
        Keyword::Expr => Some("Expr"),
        Keyword::Field => Some("Field"),
        Keyword::FunctionDefinition => Some("FunctionDefinition"),
        Keyword::StructDefinition => Some("StructDefinition"),
        Keyword::TraitConstraint => Some("TraitConstraint"),
        Keyword::TraitDefinition => Some("TraitDefinition"),
        Keyword::TraitImpl => Some("TraitImpl"),
        Keyword::TypeType => Some("Type"),

        Keyword::As
        | Keyword::Assert
        | Keyword::AssertEq
        | Keyword::Break
        | Keyword::CallData
        | Keyword::Char
        | Keyword::Comptime
        | Keyword::Constrain
        | Keyword::Continue
        | Keyword::Contract
        | Keyword::Crate
        | Keyword::Dep
        | Keyword::Else
        | Keyword::Fn
        | Keyword::For
        | Keyword::FormatString
        | Keyword::Global
        | Keyword::If
        | Keyword::Impl
        | Keyword::In
        | Keyword::Let
        | Keyword::Mod
        | Keyword::Module
        | Keyword::Mut
        | Keyword::Pub
        | Keyword::Quoted
        | Keyword::Return
        | Keyword::ReturnData
        | Keyword::String
        | Keyword::Struct
        | Keyword::Super
        | Keyword::TopLevelItem
        | Keyword::Trait
        | Keyword::Type
        | Keyword::Unchecked
        | Keyword::Unconstrained
        | Keyword::Unsafe
        | Keyword::Use
        | Keyword::Where
        | Keyword::While => None,
    }
}

pub(super) struct BuiltInFunction {
    pub(super) name: &'static str,
    pub(super) parameters: &'static str,
    pub(super) description: &'static str,
}

/// If a keyword corresponds to a built-in function, returns info about it
pub(super) fn keyword_builtin_function(keyword: &Keyword) -> Option<BuiltInFunction> {
    match keyword {
        Keyword::Assert => Some(BuiltInFunction {
            name: "assert",
            parameters: "${1:predicate}",
            description: "fn(T)",
        }),
        Keyword::AssertEq => Some(BuiltInFunction {
            name: "assert_eq",
            parameters: "${1:lhs}, ${2:rhs}",
            description: "fn(T, T)",
        }),

        Keyword::As
        | Keyword::Bool
        | Keyword::Break
        | Keyword::CallData
        | Keyword::Char
        | Keyword::Comptime
        | Keyword::Constrain
        | Keyword::Continue
        | Keyword::Contract
        | Keyword::Crate
        | Keyword::Dep
        | Keyword::Else
        | Keyword::Expr
        | Keyword::Field
        | Keyword::Fn
        | Keyword::For
        | Keyword::FormatString
        | Keyword::FunctionDefinition
        | Keyword::Global
        | Keyword::If
        | Keyword::Impl
        | Keyword::In
        | Keyword::Let
        | Keyword::Mod
        | Keyword::Module
        | Keyword::Mut
        | Keyword::Pub
        | Keyword::Quoted
        | Keyword::Return
        | Keyword::ReturnData
        | Keyword::String
        | Keyword::Struct
        | Keyword::StructDefinition
        | Keyword::Super
        | Keyword::TopLevelItem
        | Keyword::Trait
        | Keyword::TraitConstraint
        | Keyword::TraitDefinition
        | Keyword::TraitImpl
        | Keyword::Type
        | Keyword::TypeType
        | Keyword::Unchecked
        | Keyword::Unconstrained
        | Keyword::Unsafe
        | Keyword::Use
        | Keyword::Where
        | Keyword::While => None,
    }
}
