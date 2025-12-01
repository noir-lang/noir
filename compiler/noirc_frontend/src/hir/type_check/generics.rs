use std::cell::Ref;

use iter_extended::vecmap;

use crate::{
    DataType, Kind, ResolvedGeneric, Type,
    hir_def::traits::NamedType,
    node_interner::{FuncId, NodeInterner, TraitId, TypeAliasId},
};

/// Represents something that can be generic over type variables
/// such as a trait, struct type, or type alias.
///
/// Used primarily by `Elaborator::resolve_type_args` so that we can
/// have one function to do this for struct types, type aliases, traits, etc.
pub trait Generic {
    /// The name of this kind of item, for error messages. E.g. "trait", "struct type".
    fn item_kind(&self) -> &'static str;

    /// The name of this item, usually named by a user. E.g. "Foo" for "struct Foo {}"
    fn item_name(&self, interner: &NodeInterner) -> String;

    /// Each ordered generic kind on this type, excluding any named generics.
    fn generic_kinds(&self, interner: &NodeInterner) -> Vec<Kind>;

    /// True if this item kind can ever accept named type arguments.
    /// Currently, this is only true for traits. Structs & aliases can never have named args.
    fn accepts_named_type_args(&self) -> bool;

    fn named_generics(&self, interner: &NodeInterner) -> Vec<ResolvedGeneric>;
}

impl Generic for TraitId {
    fn item_kind(&self) -> &'static str {
        "trait"
    }

    fn item_name(&self, interner: &NodeInterner) -> String {
        interner.get_trait(*self).name.to_string()
    }

    fn generic_kinds(&self, interner: &NodeInterner) -> Vec<Kind> {
        interner.get_trait(*self).generics.iter().map(|generic| generic.kind()).collect()
    }

    fn accepts_named_type_args(&self) -> bool {
        true
    }

    fn named_generics(&self, interner: &NodeInterner) -> Vec<ResolvedGeneric> {
        interner.get_trait(*self).associated_types.clone()
    }
}

impl Generic for TypeAliasId {
    fn item_kind(&self) -> &'static str {
        "type alias"
    }

    fn item_name(&self, interner: &NodeInterner) -> String {
        interner.get_type_alias(*self).borrow().name.to_string()
    }

    fn generic_kinds(&self, interner: &NodeInterner) -> Vec<Kind> {
        interner
            .get_type_alias(*self)
            .borrow()
            .generics
            .iter()
            .map(|generic| generic.kind())
            .collect()
    }

    fn accepts_named_type_args(&self) -> bool {
        false
    }

    fn named_generics(&self, _interner: &NodeInterner) -> Vec<ResolvedGeneric> {
        Vec::new()
    }
}

impl Generic for Ref<'_, DataType> {
    fn item_kind(&self) -> &'static str {
        "struct"
    }

    fn item_name(&self, _interner: &NodeInterner) -> String {
        self.name.to_string()
    }

    fn generic_kinds(&self, _interner: &NodeInterner) -> Vec<Kind> {
        self.generics.iter().map(|generic| generic.kind()).collect()
    }

    fn accepts_named_type_args(&self) -> bool {
        false
    }

    fn named_generics(&self, _interner: &NodeInterner) -> Vec<ResolvedGeneric> {
        Vec::new()
    }
}

impl Generic for FuncId {
    fn item_kind(&self) -> &'static str {
        "function"
    }

    fn item_name(&self, interner: &NodeInterner) -> String {
        interner.function_name(self).to_string()
    }

    fn generic_kinds(&self, interner: &NodeInterner) -> Vec<Kind> {
        interner.function_meta(self).direct_generics.iter().map(|generic| generic.kind()).collect()
    }

    fn accepts_named_type_args(&self) -> bool {
        false
    }

    fn named_generics(&self, _interner: &NodeInterner) -> Vec<ResolvedGeneric> {
        Vec::new()
    }
}

pub struct StrPrimitiveType;

impl Generic for StrPrimitiveType {
    fn item_kind(&self) -> &'static str {
        "primitive type"
    }

    fn item_name(&self, _interner: &NodeInterner) -> String {
        "str".to_string()
    }

    fn generic_kinds(&self, _interner: &NodeInterner) -> Vec<Kind> {
        let length = Kind::u32();
        vec![length]
    }

    fn accepts_named_type_args(&self) -> bool {
        false
    }

    fn named_generics(&self, _interner: &NodeInterner) -> Vec<ResolvedGeneric> {
        Vec::new()
    }
}

pub struct FmtstrPrimitiveType;

impl Generic for FmtstrPrimitiveType {
    fn item_kind(&self) -> &'static str {
        "primitive type"
    }

    fn item_name(&self, _interner: &NodeInterner) -> String {
        "fmtstr".to_string()
    }

    fn generic_kinds(&self, _interner: &NodeInterner) -> Vec<Kind> {
        let length = Kind::u32();
        let element = Kind::Normal;
        vec![length, element]
    }

    fn accepts_named_type_args(&self) -> bool {
        false
    }

    fn named_generics(&self, _interner: &NodeInterner) -> Vec<ResolvedGeneric> {
        Vec::new()
    }
}

/// TraitGenerics are different from regular generics in that they can
/// also contain associated type arguments.
#[derive(Default, PartialEq, Eq, Clone, Hash, Ord, PartialOrd)]
pub struct TraitGenerics {
    pub ordered: Vec<Type>,
    pub named: Vec<NamedType>,
}

impl TraitGenerics {
    pub fn map(&self, mut f: impl FnMut(&Type) -> Type) -> TraitGenerics {
        let ordered = vecmap(&self.ordered, &mut f);
        let named =
            vecmap(&self.named, |named| NamedType { name: named.name.clone(), typ: f(&named.typ) });
        TraitGenerics { ordered, named }
    }

    pub fn is_empty(&self) -> bool {
        self.ordered.is_empty() && self.named.is_empty()
    }
}

impl std::fmt::Display for TraitGenerics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_trait_generics(self, f, false)
    }
}

impl std::fmt::Debug for TraitGenerics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_trait_generics(self, f, true)
    }
}

fn fmt_trait_generics(
    generics: &TraitGenerics,
    f: &mut std::fmt::Formatter<'_>,
    debug: bool,
) -> std::fmt::Result {
    if !generics.ordered.is_empty() || !generics.named.is_empty() {
        write!(f, "<")?;
        for (i, typ) in generics.ordered.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }

            if debug {
                write!(f, "{typ:?}")?;
            } else {
                write!(f, "{typ}")?;
            }
        }

        if !generics.ordered.is_empty() && !generics.named.is_empty() {
            write!(f, ", ")?;
        }

        for (i, named) in generics.named.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }

            if debug {
                write!(f, "{} = {:?}", named.name, named.typ)?;
            } else {
                write!(f, "{} = {}", named.name, named.typ)?;
            }
        }
        write!(f, ">")?;
    }
    Ok(())
}
