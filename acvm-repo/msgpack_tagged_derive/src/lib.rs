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
    Data, DataStruct, DeriveInput, Field, Fields, GenericParam, Ident, LitInt, Token, Type,
    WhereClause,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
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

/// Per-tagged-field info collected during macro expansion.
struct TaggedField<'a> {
    tag: u8,
    ident: &'a Ident,
    ty: &'a Type,
    has_default: bool,
}

fn expand_named_struct(
    input: &DeriveInput,
    fields: &Punctuated<Field, Token![,]>,
) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let name_str = name.to_string();

    // Parse each field. Tagged fields contribute to TAGS, the recursion list,
    // and the where clause. Skipped fields (`#[tag(skip)]` or `PhantomData<_>`)
    // are silently dropped — they don't go on the wire and don't constrain
    // their type.
    let mut entries: Vec<TaggedField<'_>> = Vec::with_capacity(fields.len());
    for field in fields {
        let ident = field.ident.as_ref().expect("named field has an ident");
        match classify_field(field)? {
            FieldKind::Tagged { tag, has_default } => {
                entries.push(TaggedField { tag, ident, ty: &field.ty, has_default });
            }
            FieldKind::Skipped => {}
        }
    }
    // Canonical order on the wire is tag-ascending, not source-declaration order.
    entries.sort_by_key(|e| e.tag);

    let tag_entries = entries.iter().map(|e| {
        let tag = e.tag;
        let name = e.ident.to_string();
        quote! { (#tag, #name) }
    });

    let default_entries = entries.iter().filter(|e| e.has_default).map(|e| {
        let tag = e.tag;
        quote! { #tag }
    });

    let recursion_calls = entries.iter().map(|e| {
        let ty = e.ty;
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
            const DEFAULTS: &'static [::msgpack_tagged::Tag] = &[
                #(#default_entries),*
            ];
            const ALLOW_UNKNOWN_TAGS: bool = false;

            fn register_into(_reg: &mut ::msgpack_tagged::TagRegistry) {
                if _reg.try_insert::<Self>(#name_str) {
                    #(#recursion_calls)*
                }
            }
        }
    })
}

/// What the macro should do with a given field on the wire.
enum FieldKind {
    /// Field appears on the wire under integer tag `tag`. `has_default = true`
    /// when the user wrote `#[tag(N, default)]`: encoder always emits, decoder
    /// fills `T::default()` if the tag is missing.
    Tagged { tag: u8, has_default: bool },
    /// Field is omitted from the wire (via explicit `#[tag(skip)]` or because
    /// its type is `PhantomData<_>`). Skipped fields contribute no entry to
    /// `TAGS`, no recursion into `register_into`, and no where-clause bound.
    Skipped,
}

/// Inner-args grammar for `#[tag(...)]`:
/// * `#[tag(skip)]` — the bare ident `skip`.
/// * `#[tag(N)]` — an integer tag literal.
/// * `#[tag(N, default)]` — integer tag plus the wire-tolerance modifier.
///
/// More modifiers can be added later by extending the comma-separated list
/// after the integer tag.
enum TagArgs {
    Tag { tag: u8, default: bool },
    Skip,
}

impl Parse for TagArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitInt) {
            let lit: LitInt = input.parse()?;
            let tag: u8 = lit.base10_parse()?;
            let mut default = false;
            while input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
                let modifier: Ident = input.parse()?;
                if modifier == "default" {
                    if default {
                        return Err(syn::Error::new(
                            modifier.span(),
                            "duplicate `default` modifier",
                        ));
                    }
                    default = true;
                } else {
                    return Err(syn::Error::new(
                        modifier.span(),
                        format!("unknown modifier `{modifier}` (expected `default`)"),
                    ));
                }
            }
            Ok(TagArgs::Tag { tag, default })
        } else if lookahead.peek(Ident) {
            let ident: Ident = input.parse()?;
            if ident == "skip" {
                Ok(TagArgs::Skip)
            } else {
                Err(syn::Error::new(ident.span(), "expected an integer tag or the keyword `skip`"))
            }
        } else {
            Err(lookahead.error())
        }
    }
}

/// Decide whether a field is wire-visible or skipped. Errors loudly when a
/// field has no annotation and isn't a recognized auto-skip type — the
/// strict-by-default discipline.
fn classify_field(field: &Field) -> syn::Result<FieldKind> {
    let mut found: Option<TagArgs> = None;
    for attr in &field.attrs {
        if !attr.path().is_ident("tag") {
            continue;
        }
        if found.is_some() {
            return Err(syn::Error::new_spanned(attr, "duplicate `#[tag(...)]` attribute"));
        }
        found = Some(attr.parse_args()?);
    }

    // Explicit annotation wins over auto-skip — if the user explicitly tags a
    // PhantomData field with `#[tag(N)]`, we honor that (unusual but valid).
    if let Some(args) = found {
        return Ok(match args {
            TagArgs::Tag { tag, default } => FieldKind::Tagged { tag, has_default: default },
            TagArgs::Skip => FieldKind::Skipped,
        });
    }

    // No `#[tag(...)]` at all — fall back to auto-skip for `PhantomData<_>`
    // (the conventional zero-sized "use a type parameter without storing
    // anything" pattern), otherwise error.
    if is_phantom_data(&field.ty) {
        return Ok(FieldKind::Skipped);
    }

    Err(syn::Error::new_spanned(
        field,
        "missing `#[tag(N)]` attribute — every field needs an explicit tag, `#[tag(skip)]`, or be `PhantomData<_>`",
    ))
}

/// Syntactically detect `PhantomData<_>` by checking the last path segment.
/// Matches the conventional forms (`PhantomData`, `marker::PhantomData`,
/// `std::marker::PhantomData`, `core::marker::PhantomData`). Won't recognize
/// a `PhantomData` re-imported under a different alias — that's the standard
/// trade-off for syntactic detection (serde-derive's auto-skip works the same way).
fn is_phantom_data(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(last) = type_path.path.segments.last()
    {
        return last.ident == "PhantomData";
    }
    false
}

/// Build a `where` clause for the generated impl. Adds three kinds of bounds:
///
/// 1. **`T: 'static` for every type parameter on the input.** The
///    `MsgpackTagged: 'static` supertrait propagates `Self: 'static` onto the
///    impl, which requires every generic param to be `'static` regardless of
///    whether it appears in a tagged field. (Skipped fields like
///    `_phantom: PhantomData<T>` still reference T at the type level, so
///    `Self: 'static` requires `T: 'static` even though we don't tag the
///    PhantomData field.)
/// 2. **`<TaggedFieldType>: MsgpackTagged` for each tagged field's type.**
///    Per-field-type bounds compose with hand-written impls that have unusual
///    bounds: if `MyType<A, B>: MsgpackTagged` requires `A: SomeOtherTrait`,
///    our `where MyType<A, B>: MsgpackTagged` propagates that requirement to
///    the caller transparently. Field types appearing more than once are only
///    emitted as a bound once.
/// 3. **`<TaggedFieldType>: Default` for each `#[tag(N, default)]` field.**
///    The `default` modifier promises the decoder can fill `T::default()` if
///    the tag is missing on the wire — that's only sound when `T: Default`.
///    Enforcing it via a where bound surfaces a clear "X: Default is not
///    satisfied" error at the impl site if a user marks a field `default`
///    whose type isn't `Default`.
///
/// Returns `None` only if the input has no generic params, no tagged fields,
/// and no pre-existing where clause — that lets the caller avoid emitting a
/// stray `where` token.
fn build_where_clause(input: &DeriveInput, entries: &[TaggedField<'_>]) -> Option<WhereClause> {
    let has_type_params = input.generics.params.iter().any(|p| matches!(p, GenericParam::Type(_)));
    if entries.is_empty() && !has_type_params {
        return input.generics.where_clause.clone();
    }

    let mut where_clause = input.generics.where_clause.clone().unwrap_or_else(|| WhereClause {
        where_token: <Token![where]>::default(),
        predicates: Punctuated::new(),
    });

    for param in &input.generics.params {
        if let GenericParam::Type(type_param) = param {
            let ident = &type_param.ident;
            where_clause.predicates.push(parse_quote!(#ident: 'static));
        }
    }

    let mut seen_tagged = std::collections::HashSet::new();
    let mut seen_default = std::collections::HashSet::new();
    for entry in entries {
        let ty = entry.ty;
        // Dedup by stringified token-stream of the type. Not semantic equality
        // (`Vec<u32>` vs `std::vec::Vec<u32>` would be treated as distinct),
        // but it dedupes the common case where the same path is written the
        // same way in multiple field declarations.
        let key = quote!(#ty).to_string();
        if seen_tagged.insert(key.clone()) {
            where_clause.predicates.push(parse_quote!(#ty: ::msgpack_tagged::MsgpackTagged));
        }
        if entry.has_default && seen_default.insert(key) {
            where_clause.predicates.push(parse_quote!(#ty: ::std::default::Default));
        }
    }
    Some(where_clause)
}
