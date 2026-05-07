//! Companion proc-macro crate for [`msgpack_tagged`].
//!
//! Currently handles named-field structs end-to-end (parses `#[tag(N)]` per
//! field, emits `TAGS`, and emits a `register_into` that registers `Self` and
//! recurses into each field's type). Tuple structs and enums fall through to
//! a stub expansion (empty `TAGS`, no-op `register_into`) until subsequent
//! steps add their handling.
//!
//! Design: [issue #12554](https://github.com/noir-lang/noir/issues/12554).

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Data, DataStruct, DeriveInput, Field, Fields, LitInt, Token, WhereClause, parse_macro_input,
    parse_quote, punctuated::Punctuated,
};

#[proc_macro_derive(MsgpackTagged, attributes(tag, reserved, allow_unknown_tags, via))]
pub fn derive_msgpack_tagged(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(&input).unwrap_or_else(syn::Error::into_compile_error).into()
}

fn expand(input: &DeriveInput) -> syn::Result<TokenStream2> {
    match &input.data {
        Data::Struct(DataStruct { fields: Fields::Named(named), .. }) => {
            expand_named_struct(input, &named.named)
        }
        // Tuple structs, unit structs, enums, unions: stub for now. Real
        // expansion lands in subsequent steps.
        _ => Ok(stub(input)),
    }
}

/// Stub expansion: empty `TAGS`/`RESERVED`, no-op `register_into`. Used for
/// shapes the macro hasn't learned to handle yet.
fn stub(input: &DeriveInput) -> TokenStream2 {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    quote! {
        impl #impl_generics ::msgpack_tagged::MsgpackTagged for #name #ty_generics #where_clause {
            const TAGS: &'static [(::msgpack_tagged::Tag, &'static str)] = &[];
            const RESERVED: &'static [::msgpack_tagged::Tag] = &[];
            const ALLOW_UNKNOWN_TAGS: bool = false;
            fn register_into(_reg: &mut ::msgpack_tagged::TagRegistry) {}
        }
    }
}

fn expand_named_struct(
    input: &DeriveInput,
    fields: &Punctuated<Field, Token![,]>,
) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let name_str = name.to_string();

    // Parse `#[tag(N)]` per field. Every named field must have one (eventual
    // `#[tag(skip)]` and `PhantomData<_>` auto-skip will land in a later step).
    let mut entries: Vec<(u8, &syn::Ident, &syn::Type)> = Vec::with_capacity(fields.len());
    for field in fields {
        let ident = field.ident.as_ref().expect("named field has an ident");
        let tag = parse_tag_attribute(field)?;
        entries.push((tag, ident, &field.ty));
    }
    // Canonical order on the wire is tag-ascending, not source-declaration order.
    entries.sort_by_key(|(tag, _, _)| *tag);

    let tag_entries = entries.iter().map(|(tag, ident, _)| {
        let name = ident.to_string();
        quote! { (#tag, #name) }
    });

    let recursion_calls = entries.iter().map(|(_, _, ty)| {
        quote! { <#ty as ::msgpack_tagged::MsgpackTagged>::register_into(_reg); }
    });

    // Bound *each tagged field's type* (rather than each generic param) on
    // `MsgpackTagged`. This composes correctly with hand-written impls that
    // have unusual bounds: e.g. if `MyType<A, B>: MsgpackTagged` requires
    // `A: SomeOtherTrait`, our `where MyType<A, B>: MsgpackTagged` propagates
    // that requirement through to the caller without us having to know about
    // it. Naive per-type-param bounds (`A: MsgpackTagged, B: MsgpackTagged`)
    // would be both too restrictive and insufficient in that case.
    let where_clause = build_where_clause(input, &entries);
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::msgpack_tagged::MsgpackTagged for #name #ty_generics #where_clause {
            const TAGS: &'static [(::msgpack_tagged::Tag, &'static str)] = &[
                #(#tag_entries),*
            ];
            const RESERVED: &'static [::msgpack_tagged::Tag] = &[];
            const ALLOW_UNKNOWN_TAGS: bool = false;

            fn register_into(_reg: &mut ::msgpack_tagged::TagRegistry) {
                if _reg.try_insert::<Self>(#name_str) {
                    #(#recursion_calls)*
                }
            }
        }
    })
}

/// Read the `#[tag(N)]` attribute off a field. Errors loudly if missing or
/// malformed — the strict-by-default discipline is the point.
fn parse_tag_attribute(field: &Field) -> syn::Result<u8> {
    let mut found: Option<u8> = None;
    for attr in &field.attrs {
        if !attr.path().is_ident("tag") {
            continue;
        }
        if found.is_some() {
            return Err(syn::Error::new_spanned(attr, "duplicate `#[tag(...)]` attribute"));
        }
        let lit: LitInt = attr.parse_args()?;
        let tag: u8 = lit.base10_parse()?;
        found = Some(tag);
    }
    found.ok_or_else(|| {
        syn::Error::new_spanned(
            field,
            "missing `#[tag(N)]` attribute — every field of a `MsgpackTagged` struct needs an explicit tag",
        )
    })
}

/// Build a `where` clause that requires every tagged field's type to implement
/// `MsgpackTagged`, on top of any existing where clause on the input.
///
/// Field types appearing more than once (e.g. two fields of `Vec<u8>`) are
/// only emitted as a bound once — Rust accepts duplicate bounds silently, but
/// a single bound is cleaner in cargo-expand output and avoids tickling any
/// future lint that flags duplicate trait bounds.
///
/// Returns `None` only if there are no tagged fields *and* no pre-existing
/// where clause — that lets the caller avoid emitting a stray `where` token.
fn build_where_clause(
    input: &DeriveInput,
    entries: &[(u8, &syn::Ident, &syn::Type)],
) -> Option<WhereClause> {
    if entries.is_empty() {
        return input.generics.where_clause.clone();
    }
    let mut where_clause = input.generics.where_clause.clone().unwrap_or_else(|| WhereClause {
        where_token: <Token![where]>::default(),
        predicates: Punctuated::new(),
    });
    let mut seen = std::collections::HashSet::new();
    for (_, _, ty) in entries {
        // Dedup by stringified token-stream of the type. Not semantic equality
        // (`Vec<u32>` vs `std::vec::Vec<u32>` would be treated as distinct),
        // but it dedupes the common case where the same path is written the
        // same way in multiple field declarations.
        if seen.insert(quote!(#ty).to_string()) {
            where_clause.predicates.push(parse_quote!(#ty: ::msgpack_tagged::MsgpackTagged));
        }
    }
    Some(where_clause)
}
