use std::fmt::Display;

use iter_extended::vecmap;
use noirc_errors::Location;

use crate::ast::{
    BlockExpression, Expression, FunctionReturnType, Ident, NoirFunction, Path, UnresolvedGenerics,
    UnresolvedType,
};
use crate::node_interner::TraitId;
use crate::token::SecondaryAttribute;

use super::{Documented, GenericTypeArgs, ItemVisibility, UnresolvedGeneric, UnresolvedTypeData};

/// AST node for trait definitions:
/// `trait name<generics> { ... items ... }`
#[derive(Clone, Debug)]
pub struct NoirTrait {
    pub name: Ident,
    pub generics: UnresolvedGenerics,
    pub bounds: Vec<TraitBound>,
    pub where_clause: Vec<UnresolvedTraitConstraint>,
    pub location: Location,
    pub items: Vec<Documented<TraitItem>>,
    pub attributes: Vec<SecondaryAttribute>,
    pub visibility: ItemVisibility,
    pub is_alias: bool,
}

/// Any declaration inside the body of a trait that a user is required to
/// specify when implementing the trait.
#[derive(Clone, Debug)]
pub enum TraitItem {
    Function {
        is_unconstrained: bool,
        visibility: ItemVisibility,
        is_comptime: bool,
        name: Ident,
        generics: UnresolvedGenerics,
        parameters: Vec<(Ident, UnresolvedType)>,
        return_type: FunctionReturnType,
        where_clause: Vec<UnresolvedTraitConstraint>,
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
    pub type_location: Location,
    pub generics: UnresolvedGenerics,
    pub where_clause: Vec<UnresolvedTraitConstraint>,
    pub methods: Vec<(Documented<NoirFunction>, Location)>,
}

/// Ast node for an implementation of a trait for a particular type
/// `impl trait_name<trait_generics> for object_type where where_clauses { ... items ... }`
#[derive(Clone, Debug)]
pub struct NoirTraitImpl {
    pub impl_generics: UnresolvedGenerics,

    pub r#trait: UnresolvedType,

    pub object_type: UnresolvedType,

    pub where_clause: Vec<UnresolvedTraitConstraint>,

    pub items: Vec<Documented<TraitImplItem>>,

    /// true if generated at compile-time, e.g. from a trait alias
    pub is_synthetic: bool,
}

/// Represents a simple trait constraint such as `where Foo: TraitY<U, V>`
/// Complex trait constraints such as `where Foo: Display + TraitX + TraitY<U, V>` are converted
/// in the parser to a series of simple constraints:
///   `Foo: Display`
///   `Foo: TraitX`
///   `Foo: TraitY<U, V>`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnresolvedTraitConstraint {
    pub typ: UnresolvedType,
    pub trait_bound: TraitBound,
}

/// Represents a single trait bound, such as `TraitX` or `TraitY<U, V>`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitBound {
    pub trait_path: Path,
    pub trait_id: Option<TraitId>, // initially None, gets assigned during DC
    pub trait_generics: GenericTypeArgs,
}

#[derive(Clone, Debug)]
pub struct TraitImplItem {
    pub kind: TraitImplItemKind,
    pub location: Location,
}

#[derive(Clone, Debug)]
pub enum TraitImplItemKind {
    Function(NoirFunction),
    Constant(Ident, UnresolvedType, Expression),
    Type { name: Ident, alias: UnresolvedType },
}

impl Display for TypeImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generics = vecmap(&self.generics, |generic| generic.to_string());
        let generics =
            if generics.is_empty() { "".into() } else { format!("<{}>", generics.join(", ")) };

        writeln!(f, "impl{} {} {{", generics, self.object_type)?;

        for (method, _) in self.methods.iter() {
            let method = method.to_string();
            for line in method.lines() {
                writeln!(f, "    {line}")?;
            }
        }

        write!(f, "}}")
    }
}

// TODO: display where clauses (follow-up issue)
impl Display for NoirTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generics = vecmap(&self.generics, |generic| generic.to_string());
        let generics = if generics.is_empty() { "".into() } else { generics.join(", ") };

        write!(f, "trait {}{}", self.name, generics)?;

        if self.is_alias {
            let bounds = vecmap(&self.bounds, |bound| bound.to_string()).join(" + ");
            return write!(f, " = {};", bounds);
        }

        if !self.bounds.is_empty() {
            let bounds = vecmap(&self.bounds, |bound| bound.to_string()).join(" + ");
            write!(f, ": {}", bounds)?;
        }
        writeln!(f, " {{")?;

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
            TraitItem::Function {
                name,
                generics,
                parameters,
                return_type,
                where_clause,
                body,
                is_unconstrained,
                visibility,
                is_comptime,
            } => {
                let generics = vecmap(generics, |generic| generic.to_string());
                let parameters = vecmap(parameters, |(name, typ)| format!("{name}: {typ}"));
                let where_clause = vecmap(where_clause, ToString::to_string);

                let generics = generics.join(", ");
                let parameters = parameters.join(", ");
                let where_clause = where_clause.join(", ");

                let unconstrained = if *is_unconstrained { "unconstrained " } else { "" };
                let visibility = if *visibility == ItemVisibility::Private {
                    "".to_string()
                } else {
                    visibility.to_string()
                };
                let is_comptime = if *is_comptime { "comptime " } else { "" };

                write!(
                    f,
                    "{unconstrained}{visibility}{is_comptime}fn {name}<{generics}>({parameters}) -> {return_type} where {where_clause}"
                )?;

                if let Some(body) = body { write!(f, "{body}") } else { write!(f, ";") }
            }
            TraitItem::Constant { name, typ, default_value } => {
                write!(f, "let {name}: {typ}")?;

                if let Some(default_value) = default_value {
                    write!(f, "{default_value};")
                } else {
                    write!(f, ";")
                }
            }
            TraitItem::Type { name } => write!(f, "type {name};"),
        }
    }
}

impl Display for UnresolvedTraitConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.typ, self.trait_bound)
    }
}

impl Display for TraitBound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.trait_path, self.trait_generics)
    }
}

impl Display for NoirTraitImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Synthetic NoirTraitImpl's don't get printed
        if self.is_synthetic {
            return Ok(());
        }

        write!(f, "impl")?;
        if !self.impl_generics.is_empty() {
            write!(
                f,
                "<{}>",
                self.impl_generics.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")
            )?;
        }

        write!(f, " {} for {}", self.r#trait, self.object_type)?;
        if !self.where_clause.is_empty() {
            write!(
                f,
                " where {}",
                self.where_clause.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")
            )?;
        }
        writeln!(f, "{{")?;

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
        self.kind.fmt(f)
    }
}

impl Display for TraitImplItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraitImplItemKind::Function(function) => function.fmt(f),
            TraitImplItemKind::Type { name, alias } => write!(f, "type {name} = {alias};"),
            TraitImplItemKind::Constant(name, typ, value) => {
                write!(f, "let {name}: {typ} = {value};")
            }
        }
    }
}

/// Moves trait bounds from generics into where clauses. For example:
///
/// ```noir
/// fn foo<T: Trait>(x: T) -> T {}
/// ```
///
/// becomes:
///
/// ```noir
/// fn foo<T>(x: T) -> T where T: Trait {}
/// ```
pub(crate) fn desugar_generic_trait_bounds(
    generics: &mut Vec<UnresolvedGeneric>,
    where_clause: &mut Vec<UnresolvedTraitConstraint>,
) {
    for generic in generics {
        let UnresolvedGeneric::Variable(ident, trait_bounds) = generic else {
            continue;
        };

        if trait_bounds.is_empty() {
            continue;
        }

        for trait_bound in std::mem::take(trait_bounds) {
            let path = Path::from_ident(ident.clone());
            let typ = UnresolvedTypeData::Named(path, GenericTypeArgs::default(), true);
            let typ = UnresolvedType { typ, location: ident.location() };
            let trait_constraint = UnresolvedTraitConstraint { typ, trait_bound };
            where_clause.push(trait_constraint);
        }
    }
}
