use std::cell::Ref;

use crate::{
    macros_api::NodeInterner,
    node_interner::{TraitId, TypeAliasId},
    ResolvedGeneric, StructType,
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

    /// Each ordered generic on this type, excluding any named generics.
    fn generics(&self, interner: &NodeInterner) -> Vec<ResolvedGeneric>;

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

    fn generics(&self, interner: &NodeInterner) -> Vec<ResolvedGeneric> {
        interner.get_trait(*self).generics.clone()
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

    fn generics(&self, interner: &NodeInterner) -> Vec<ResolvedGeneric> {
        interner.get_type_alias(*self).borrow().generics.clone()
    }

    fn accepts_named_type_args(&self) -> bool {
        false
    }

    fn named_generics(&self, _interner: &NodeInterner) -> Vec<ResolvedGeneric> {
        Vec::new()
    }
}

impl Generic for Ref<'_, StructType> {
    fn item_kind(&self) -> &'static str {
        "struct"
    }

    fn item_name(&self, _interner: &NodeInterner) -> String {
        self.name.to_string()
    }

    fn generics(&self, _interner: &NodeInterner) -> Vec<ResolvedGeneric> {
        self.generics.clone()
    }

    fn accepts_named_type_args(&self) -> bool {
        false
    }

    fn named_generics(&self, _interner: &NodeInterner) -> Vec<ResolvedGeneric> {
        Vec::new()
    }
}
