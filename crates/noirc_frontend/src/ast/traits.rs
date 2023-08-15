use std::fmt::Display;

use iter_extended::vecmap;
use noirc_errors::Span;

use crate::{
    BlockExpression, Expression, FunctionReturnType, Ident, NoirFunction, UnresolvedGenerics,
    UnresolvedType,
};

/// AST node for trait definitions:
/// `trait name<generics> { ... items ... }`
#[derive(Clone, Debug)]
pub struct NoirTrait {
    pub name: Ident,
    pub generics: Vec<Ident>,
    pub where_clause: Vec<TraitConstraint>,
    pub span: Span,
    pub items: Vec<TraitItem>,
}

/// Any declaration inside the body of a trait that a user is required to
/// specify when implementing the trait.
#[derive(Clone, Debug)]
pub enum TraitItem {
    Function {
        name: Ident,
        generics: Vec<Ident>,
        parameters: Vec<(Ident, UnresolvedType)>,
        return_type: FunctionReturnType,
        where_clause: Vec<TraitConstraint>,
        body: Option<BlockExpression>,
    },
    Constant {
        name: Ident,
        typ: UnresolvedType,
        default_value: Option<Expression>,
    },
    Type {
        name: Ident,
    },
}

/// Ast node for an impl of a concrete type
/// `impl object_type<generics> { ... methods ... }`
#[derive(Clone, Debug)]
pub struct TypeImpl {
    pub object_type: UnresolvedType,
    pub type_span: Span,
    pub generics: UnresolvedGenerics,
    pub methods: Vec<NoirFunction>,
}

/// Ast node for an implementation of a trait for a particular type
/// `impl trait_name<trait_generics> for object_type where where_clauses { ... items ... }`
#[derive(Clone, Debug)]
pub struct TraitImpl {
    pub impl_generics: UnresolvedGenerics,

    pub trait_name: Ident,
    pub trait_generics: Vec<UnresolvedType>,

    pub object_type: UnresolvedType,
    pub object_type_span: Span,

    pub where_clause: Vec<TraitConstraint>,

    pub items: Vec<TraitImplItem>,
}

/// Represents a trait constraint such as `where Foo: Display`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraitConstraint {
    pub typ: UnresolvedType,
    pub trait_name: Ident,
    pub trait_generics: Vec<UnresolvedType>,
}

#[derive(Clone, Debug)]
pub enum TraitImplItem {
    Function(NoirFunction),
    Constant(Ident, UnresolvedType, Expression),
    Type { name: Ident, alias: UnresolvedType },
}

impl Display for TypeImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generics = vecmap(&self.generics, |generic| generic.to_string());
        let generics = if generics.is_empty() { "".into() } else { generics.join(", ") };

        writeln!(f, "impl{} {} {{", generics, self.object_type)?;

        for method in self.methods.iter() {
            let method = method.to_string();
            for line in method.lines() {
                writeln!(f, "    {line}")?;
            }
        }

        write!(f, "}}")
    }
}

impl Display for NoirTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generics = vecmap(&self.generics, |generic| generic.to_string());
        let generics = if generics.is_empty() { "".into() } else { generics.join(", ") };

        writeln!(f, "trait {}{} {{", self.name, generics)?;

        for item in self.items.iter() {
            let item = item.to_string();
            for line in item.lines() {
                writeln!(f, "    {line}")?;
            }
        }

        write!(f, "}}")
    }
}

impl Display for TraitItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraitItem::Function { name, generics, parameters, return_type, where_clause, body } => {
                let generics = vecmap(generics, |generic| generic.to_string());
                let parameters = vecmap(parameters, |(name, typ)| format!("{name}: {typ}"));
                let where_clause = vecmap(where_clause, ToString::to_string);

                let generics = generics.join(", ");
                let parameters = parameters.join(", ");
                let where_clause = where_clause.join(", ");

                write!(
                    f,
                    "fn {name}<{}>({}) -> {} where {}",
                    generics, parameters, return_type, where_clause
                )?;

                if let Some(body) = body {
                    write!(f, "{}", body)
                } else {
                    write!(f, ";")
                }
            }
            TraitItem::Constant { name, typ, default_value } => {
                write!(f, "let {}: {}", name, typ)?;

                if let Some(default_value) = default_value {
                    write!(f, "{};", default_value)
                } else {
                    write!(f, ";")
                }
            }
            TraitItem::Type { name } => write!(f, "type {name};"),
        }
    }
}

impl Display for TraitConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generics = vecmap(&self.trait_generics, |generic| generic.to_string());
        write!(f, "{}: {}<{}>", self.typ, self.trait_name, generics.join(", "))
    }
}

impl Display for TraitImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generics = vecmap(&self.trait_generics, |generic| generic.to_string());
        let generics = generics.join(", ");

        writeln!(f, "impl {}<{}> for {} {{", self.trait_name, generics, self.object_type)?;

        for item in self.items.iter() {
            let item = item.to_string();
            for line in item.lines() {
                writeln!(f, "    {line}")?;
            }
        }

        write!(f, "}}")
    }
}

impl Display for TraitImplItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraitImplItem::Function(function) => function.fmt(f),
            TraitImplItem::Type { name, alias } => write!(f, "type {name} = {alias};"),
            TraitImplItem::Constant(name, typ, value) => {
                write!(f, "let {}: {} = {};", name, typ, value)
            }
        }
    }
}
