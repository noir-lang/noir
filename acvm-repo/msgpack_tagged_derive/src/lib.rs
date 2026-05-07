//! Companion proc-macro crate for `msgpack_tagged`.
//!
//! Handles named-field structs, tuple structs / newtypes, and enums end-to-end:
//! parses `#[tag(N)]` annotations, builds a `Tagged::Product` (struct-shaped)
//! or `Tagged::Sum` (enum-shaped) wire description, and emits a `register_into`
//! that registers `Self` and recurses into each field/variant payload type.
//! Unit structs and unions still fall through to a stub expansion until
//! subsequent steps add their handling.
//!
//! Per-variant struct/tuple field tagging on enum variants is the next
//! incremental step — at this point every enum variant gets an *empty*
//! payload `Product`, and any `#[tag(...)]` on a variant's field is rejected.
//!
//! Design: [issue #12554](https://github.com/noir-lang/noir/issues/12554).

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Expr, ExprLit, Field, Fields, GenericParam,
    Ident, Lit, LitInt, Meta, Token, Type, Variant, WhereClause,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
};

#[proc_macro_derive(MsgpackTagged, attributes(tag, tagged))]
pub fn derive_msgpack_tagged(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(&input).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// `default_on_reserved` and `default_on_unknown` are sum-only decode
/// policies — a product has nothing to substitute (its fields are independent
/// and an unknown one is just skipped or errored, depending on
/// `allow_unknown_tags`). Reject them eagerly with a clear message rather
/// than silently dropping the flag.
fn reject_sum_only_decode_flags(input: &DeriveInput, type_attrs: &TypeAttrs) -> syn::Result<()> {
    if type_attrs.default_on_reserved {
        return Err(syn::Error::new_spanned(
            input,
            "`#[tagged(default_on_reserved)]` only applies to enums — products skip or \
             error on unknown field tags via `allow_unknown_tags` instead",
        ));
    }
    if type_attrs.default_on_unknown {
        return Err(syn::Error::new_spanned(
            input,
            "`#[tagged(default_on_unknown)]` only applies to enums — products skip or \
             error on unknown field tags via `allow_unknown_tags` instead",
        ));
    }
    Ok(())
}

/// Build a bare `Product { ... }` struct literal from parsed field entries
/// plus the reserved list and unknown-tag policy. Used both for top-level
/// struct shapes (wrapped in `Tagged::Product(...)`) and for the inner
/// payload of enum variants (used unwrapped).
fn product_struct_literal(
    entries: &[TaggedField<'_>],
    reserved: &[u8],
    allow_unknown_tags: bool,
) -> TokenStream2 {
    let field_entries = entries.iter().map(|e| {
        let tag = e.tag;
        let name = &e.name;
        quote! { (#tag, #name) }
    });
    let default_entries = entries.iter().filter(|e| e.has_default).map(|e| {
        let tag = e.tag;
        quote! { #tag }
    });
    let reserved_entries = reserved.iter().map(|tag| quote! { #tag });
    quote! {
        ::msgpack_tagged::Product {
            fields: &[#(#field_entries),*],
            reserved: &[#(#reserved_entries),*],
            defaults: &[#(#default_entries),*],
            allow_unknown_tags: #allow_unknown_tags,
        }
    }
}

/// Build a `Tagged::Product(Product { ... })` literal — top-level
/// struct/tuple-struct emission. Wraps [`product_struct_literal`].
fn product_literal(
    entries: &[TaggedField<'_>],
    reserved: &[u8],
    allow_unknown_tags: bool,
) -> TokenStream2 {
    let inner = product_struct_literal(entries, reserved, allow_unknown_tags);
    quote! { ::msgpack_tagged::Tagged::Product(#inner) }
}

/// Empty `Tagged::Product` literal — used by newtypes, `via`-delegating
/// types, the stub expansion, and any other shape that contributes no wire
/// metadata of its own.
fn empty_product_literal() -> TokenStream2 {
    quote! {
        ::msgpack_tagged::Tagged::Product(::msgpack_tagged::Product {
            fields: &[],
            reserved: &[],
            defaults: &[],
            allow_unknown_tags: false,
        })
    }
}

/// Build a `Tagged::Sum` literal from variant entries, the enum-level
/// reserved variant-tag list, and the runtime decode-policy flags. Each
/// variant's `payload` is rendered as a `Product` populated from the
/// variant's parsed tagged fields, plus its variant-level
/// `#[tagged(reserved(...))]` and `#[tagged(allow_unknown_tags)]` flags.
fn sum_literal(
    variants: &[TaggedVariant<'_>],
    reserved: &[u8],
    default_on_reserved: bool,
    default_on_unknown: bool,
) -> TokenStream2 {
    let variant_entries = variants.iter().map(|v| {
        let tag = v.tag;
        let name = &v.name;
        let payload =
            product_struct_literal(&v.payload, &v.payload_reserved, v.payload_allow_unknown_tags);
        quote! {
            ::msgpack_tagged::Variant {
                tag: #tag,
                name: #name,
                payload: #payload,
            }
        }
    });
    let reserved_entries = reserved.iter().map(|tag| quote! { #tag });
    quote! {
        ::msgpack_tagged::Tagged::Sum(::msgpack_tagged::Sum {
            variants: &[#(#variant_entries),*],
            reserved: &[#(#reserved_entries),*],
            default_on_reserved: #default_on_reserved,
            default_on_unknown: #default_on_unknown,
        })
    }
}

fn expand(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let type_attrs = parse_tagged_type_attrs(input)?;

    // `via(...)` short-circuits the rest of expansion regardless of shape:
    // struct, tuple struct, or enum — they all delegate to the wire DTO. The
    // public type's own fields/variants are wire-irrelevant in this case, so
    // we also reject any field-level `#[tag(...)]` annotations that would
    // suggest otherwise.
    if let Some(wire_type) = &type_attrs.via {
        validate_no_field_tag_attrs(input)?;
        return Ok(expand_via(input, wire_type));
    }

    match &input.data {
        Data::Struct(DataStruct { fields: Fields::Named(named), .. }) => {
            expand_named_struct(input, &named.named, &type_attrs)
        }
        Data::Struct(DataStruct { fields: Fields::Unnamed(unnamed), .. }) => {
            expand_unnamed_struct(input, &unnamed.unnamed, &type_attrs)
        }
        Data::Enum(data) => expand_enum(input, data, &type_attrs),
        // Unit structs and unions: stub for now. Real expansion lands in
        // subsequent steps.
        _ => Ok(stub(input)),
    }
}

/// Dispatch for tuple structs (`struct Foo(A, B)`). Single-field tuple
/// structs are *newtypes* and pass through to the inner type without a
/// registry entry of their own; multi-field tuple structs register
/// themselves with positional names ("0", "1", …).
fn expand_unnamed_struct(
    input: &DeriveInput,
    fields: &Punctuated<Field, Token![,]>,
    type_attrs: &TypeAttrs,
) -> syn::Result<TokenStream2> {
    debug_assert!(type_attrs.via.is_none()); // handled in `expand`
    if fields.len() == 1 {
        expand_newtype(input, fields.first().unwrap(), type_attrs)
    } else {
        expand_tuple_struct(input, fields, type_attrs)
    }
}

/// Newtype (single-element tuple struct): wire bytes are exactly the inner
/// type's bytes. The newtype itself doesn't get a registry entry — only its
/// inner type does (via the recursive `register_into`). Type-level
/// `reserved`/`allow_unknown_tags` are inert and rejected for clarity.
fn expand_newtype(
    input: &DeriveInput,
    inner_field: &Field,
    type_attrs: &TypeAttrs,
) -> syn::Result<TokenStream2> {
    reject_sum_only_decode_flags(input, type_attrs)?;
    if !type_attrs.reserved.is_empty() {
        return Err(syn::Error::new_spanned(
            input,
            "newtype structs (single-element tuple structs) pass through to the inner type \
             and have no wire shape of their own — `#[tagged(reserved(...))]` doesn't apply",
        ));
    }
    if type_attrs.allow_unknown_tags {
        return Err(syn::Error::new_spanned(
            input,
            "newtype structs (single-element tuple structs) pass through to the inner type \
             and have no wire shape of their own — `#[tagged(allow_unknown_tags)]` doesn't apply",
        ));
    }
    for attr in &inner_field.attrs {
        if attr.path().is_ident("tag") {
            return Err(syn::Error::new_spanned(
                attr,
                "newtype structs pass through to the inner type — \
                 `#[tag(...)]` on the inner field is not allowed",
            ));
        }
    }

    let name = &input.ident;
    let inner_type = &inner_field.ty;
    let where_clause = build_passthrough_where_clause(input, inner_type);
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
    let tagged = empty_product_literal();

    Ok(quote! {
        impl #impl_generics ::msgpack_tagged::MsgpackTagged for #name #ty_generics #where_clause {
            const TAGGED: ::msgpack_tagged::Tagged = #tagged;

            fn register_into(_reg: &mut ::msgpack_tagged::TagRegistry) {
                <#inner_type as ::msgpack_tagged::MsgpackTagged>::register_into(_reg);
            }
        }
    })
}

/// Multi-element tuple struct (`struct Pair(A, B, ...)`). Tagging style must
/// be uniform: either every field carries `#[tag(N)]` (explicit, allows
/// reordering / `default`) or none do (implicit positional 0, 1, 2, …).
/// Mixing is rejected.
///
/// To be clear, even with positional tagging, the tags becomes keys in a map,
/// not indexes in an array, they just don't have to be spelled out. As such,
/// they can be reserved, if one field replaces another in a newer version.
///
/// Field names in `TAGS` are positional strings ("0", "1", …) — the wrapper
/// Serializer addresses tuple-struct fields positionally, not by name, so
/// the names are placeholders.
fn expand_tuple_struct(
    input: &DeriveInput,
    fields: &Punctuated<Field, Token![,]>,
    type_attrs: &TypeAttrs,
) -> syn::Result<TokenStream2> {
    reject_sum_only_decode_flags(input, type_attrs)?;
    let name = &input.ident;
    let name_str = parse_serde_rename(input)?.unwrap_or_else(|| name.to_string());
    let reserved = &type_attrs.reserved;
    let allow_unknown_tags = type_attrs.allow_unknown_tags;

    let entries = parse_tuple_fields(input, fields, reserved)?;

    let recursion_calls = entries.iter().map(|e| {
        let ty = e.ty;
        quote! { <#ty as ::msgpack_tagged::MsgpackTagged>::register_into(_reg); }
    });

    let tagged = product_literal(&entries, reserved, allow_unknown_tags);
    let where_clause = build_where_clause(input, &entries);
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::msgpack_tagged::MsgpackTagged for #name #ty_generics #where_clause {
            const TAGGED: ::msgpack_tagged::Tagged = #tagged;

            fn register_into(_reg: &mut ::msgpack_tagged::TagRegistry) {
                if _reg.try_insert::<Self>(#name_str) {
                    #(#recursion_calls)*
                }
            }
        }
    })
}

/// Per-tagged-variant info collected during enum macro expansion. `name` is
/// the variant's wire-name (its Rust ident, as a string). `payload` holds
/// the parsed payload-field entries — empty for unit variants, populated by
/// [`parse_named_fields`] for struct-shaped variants and [`parse_tuple_fields`]
/// for tuple-shaped variants. The entries drive both the variant's emitted
/// payload `Product` and the per-field bounds (`MsgpackTagged`, `Default`)
/// in the impl's where clause.
///
/// `payload_reserved` and `payload_allow_unknown_tags` are the variant-level
/// `#[tagged(reserved(...))]` and `#[tagged(allow_unknown_tags)]` flags,
/// scoped to the variant's *payload field* tag space (not to the variant
/// tag itself — that's governed by the enclosing type's `#[tagged(...)]`).
struct TaggedVariant<'a> {
    tag: u8,
    name: String,
    payload: Vec<TaggedField<'a>>,
    payload_reserved: Vec<u8>,
    payload_allow_unknown_tags: bool,
}

/// Enum (`enum E { A, B(...), C { ... } }`). Each variant carries an
/// explicit `#[tag(N)]`; the variant tag is what goes on the wire as the
/// discriminator. The expansion emits a `Tagged::Sum` listing every variant
/// in tag-ascending order, and a `register_into` that registers `Self` and
/// recurses into every tagged variant-payload field type so nested
/// `MsgpackTagged` types are reached.
///
/// Variant payloads carry their own `#[tag(N)]` annotations: named-variant
/// fields use the same "every field needs an explicit tag (or auto-skip)"
/// rule as top-level named structs, and tuple-variant fields use the same
/// all-or-nothing implicit/explicit positional rule as top-level tuple
/// structs. `#[tagged(reserved(...))]` at the enum level applies only to
/// the variant tags, not the field tags inside any variant's payload —
/// each variant's payload starts with an empty reserved list.
///
/// `#[tagged(allow_unknown_tags)]` is rejected on enums: an unknown variant
/// tag has no skip semantics — there's no fragment to skip, since the
/// value's discriminator itself is unknown — so the flag would have nowhere
/// to land in the wire shape. Use `#[tagged(default_on_unknown)]` instead
/// for sums where `T::default()` is a sound stand-in.
fn expand_enum(
    input: &DeriveInput,
    data: &DataEnum,
    type_attrs: &TypeAttrs,
) -> syn::Result<TokenStream2> {
    debug_assert!(type_attrs.via.is_none()); // handled in `expand`
    if type_attrs.allow_unknown_tags {
        return Err(syn::Error::new_spanned(
            input,
            "`#[tagged(allow_unknown_tags)]` doesn't apply to enums — there's no \
             meaningful skip semantics for an unknown variant tag (the value's \
             discriminator itself becomes unrepresentable). Use it on a struct \
             field whose unknown tags can be silently dropped instead",
        ));
    }
    let name = &input.ident;
    let name_str = parse_serde_rename(input)?.unwrap_or_else(|| name.to_string());
    let reserved = &type_attrs.reserved;

    let mut variants: Vec<TaggedVariant<'_>> = Vec::with_capacity(data.variants.len());
    let mut seen_tags = std::collections::HashSet::new();
    for variant in &data.variants {
        let tag = parse_variant_tag(variant, reserved)?;
        if !seen_tags.insert(tag) {
            return Err(syn::Error::new_spanned(
                variant,
                format!("variant tag {tag} is used more than once"),
            ));
        }
        // Variant-level `#[tagged(...)]` configures the variant's payload —
        // its `reserved` list governs payload field tags (not the variant
        // tag itself), and `allow_unknown_tags` governs unknown payload
        // field tags on decode.
        let variant_attrs = parse_tagged_variant_attrs(variant)?;
        let payload = match &variant.fields {
            Fields::Unit => Vec::new(),
            Fields::Named(named) => parse_named_fields(&named.named, &variant_attrs.reserved)?,
            Fields::Unnamed(unnamed) => {
                parse_tuple_fields(variant, &unnamed.unnamed, &variant_attrs.reserved)?
            }
        };
        variants.push(TaggedVariant {
            tag,
            name: variant.ident.to_string(),
            payload,
            payload_reserved: variant_attrs.reserved,
            payload_allow_unknown_tags: variant_attrs.allow_unknown_tags,
        });
    }
    variants.sort_by_key(|v| v.tag);

    let recursion_calls = variants.iter().flat_map(|v| {
        v.payload.iter().map(|entry| {
            let ty = entry.ty;
            quote! { <#ty as ::msgpack_tagged::MsgpackTagged>::register_into(_reg); }
        })
    });

    let default_on_reserved = type_attrs.default_on_reserved;
    let default_on_unknown = type_attrs.default_on_unknown;
    let needs_default_bound = default_on_reserved || default_on_unknown;
    let tagged = sum_literal(&variants, reserved, default_on_reserved, default_on_unknown);
    let where_clause = build_enum_where_clause(input, &variants, needs_default_bound);
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::msgpack_tagged::MsgpackTagged for #name #ty_generics #where_clause {
            const TAGGED: ::msgpack_tagged::Tagged = #tagged;

            fn register_into(_reg: &mut ::msgpack_tagged::TagRegistry) {
                if _reg.try_insert::<Self>(#name_str) {
                    #(#recursion_calls)*
                }
            }
        }
    })
}

/// Parse the (required) `#[tag(N)]` attribute on an enum variant. Rejects the
/// `skip` form and the `default` modifier — neither has clear semantics for a
/// variant — and rejects tags that collide with the type's reserved list.
fn parse_variant_tag(variant: &Variant, reserved: &[u8]) -> syn::Result<u8> {
    let mut found: Option<(&Attribute, TagArgs)> = None;
    for attr in &variant.attrs {
        if !attr.path().is_ident("tag") {
            continue;
        }
        if found.is_some() {
            return Err(syn::Error::new_spanned(attr, "duplicate `#[tag(...)]` attribute"));
        }
        found = Some((attr, attr.parse_args()?));
    }
    let Some((attr, args)) = found else {
        return Err(syn::Error::new_spanned(
            variant,
            "missing `#[tag(N)]` attribute on enum variant — every variant needs an explicit tag",
        ));
    };
    match args {
        TagArgs::Tag { tag, default } => {
            if default {
                return Err(syn::Error::new_spanned(
                    attr,
                    "`default` modifier is not allowed on enum variants",
                ));
            }
            if reserved.contains(&tag) {
                return Err(syn::Error::new_spanned(
                    attr,
                    format!(
                        "tag {tag} is in the type's `#[tagged(reserved(...))]` list — pick a different tag, or remove it from the reserved list"
                    ),
                ));
            }
            Ok(tag)
        }
        TagArgs::Skip => {
            Err(syn::Error::new_spanned(attr, "`#[tag(skip)]` is not allowed on enum variants"))
        }
    }
}

/// Where clause for an enum impl. Same shape as `build_where_clause` for
/// structs — `T: 'static` per type parameter, plus a deduped
/// `<PayloadFieldType>: MsgpackTagged` bound for every tagged field type
/// appearing in any variant's payload, plus `<PayloadFieldType>: Default`
/// for any field marked `#[tag(N, default)]`. When `needs_default_bound`
/// is set (because the type opts into `default_on_reserved` or
/// `default_on_unknown`), we additionally add `Self: Default` so a missing
/// `derive(Default)` surfaces as a clear "Self: Default is not satisfied"
/// error at the impl site.
fn build_enum_where_clause(
    input: &DeriveInput,
    variants: &[TaggedVariant<'_>],
    needs_default_bound: bool,
) -> Option<WhereClause> {
    let has_type_params = input.generics.params.iter().any(|p| matches!(p, GenericParam::Type(_)));
    let any_payload = variants.iter().any(|v| !v.payload.is_empty());
    if !any_payload && !has_type_params && !needs_default_bound {
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

    let mut seen_msgpack = std::collections::HashSet::new();
    let mut seen_default = std::collections::HashSet::new();
    for v in variants {
        for entry in &v.payload {
            let ty = entry.ty;
            let key = quote!(#ty).to_string();
            if seen_msgpack.insert(key.clone()) {
                where_clause.predicates.push(parse_quote!(#ty: ::msgpack_tagged::MsgpackTagged));
            }
            if entry.has_default && seen_default.insert(key) {
                where_clause.predicates.push(parse_quote!(#ty: ::std::default::Default));
            }
        }
    }

    if needs_default_bound {
        where_clause.predicates.push(parse_quote!(Self: ::std::default::Default));
    }

    Some(where_clause)
}

/// Where clause for newtype structs: every type param needs `'static` (from
/// the supertrait), and the inner type must be `MsgpackTagged` so the
/// `register_into` call type-checks. No field-type bounds beyond that — a
/// newtype contributes no fields of its own to the wire.
fn build_passthrough_where_clause(input: &DeriveInput, inner_type: &Type) -> Option<WhereClause> {
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
    where_clause.predicates.push(parse_quote!(#inner_type: ::msgpack_tagged::MsgpackTagged));
    Some(where_clause)
}

/// Reject any field-level `#[tag(...)]` attribute on the input. Used when
/// `#[tagged(via(...))]` is set: the public type's fields are wire-irrelevant,
/// so a `#[tag(...)]` annotation would either be a leftover from before the
/// migration to `via` or a misunderstanding of where tags belong (on the
/// wire DTO). Either way, loud rejection is better than silent ignore.
fn validate_no_field_tag_attrs(input: &DeriveInput) -> syn::Result<()> {
    let check = |fields: &Fields| -> syn::Result<()> {
        for field in fields {
            for attr in &field.attrs {
                if attr.path().is_ident("tag") {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "field-level `#[tag(...)]` is not allowed on a type with `#[tagged(via(...))]` — \
                         fields of a `via`-delegating type are wire-irrelevant; \
                         tag the wire DTO's fields instead",
                    ));
                }
            }
        }
        Ok(())
    };
    match &input.data {
        Data::Struct(s) => check(&s.fields)?,
        Data::Enum(e) => {
            for variant in &e.variants {
                for attr in &variant.attrs {
                    if attr.path().is_ident("tag") {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "variant-level `#[tag(...)]` is not allowed on a type with `#[tagged(via(...))]` — \
                             variants of a `via`-delegating enum are wire-irrelevant; \
                             tag the wire DTO's variants instead",
                        ));
                    }
                    if attr.path().is_ident("tagged") {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "variant-level `#[tagged(...)]` is not allowed on a type with `#[tagged(via(...))]` — \
                             variants of a `via`-delegating enum are wire-irrelevant; \
                             configure the wire DTO instead",
                        ));
                    }
                }
                check(&variant.fields)?;
            }
        }
        Data::Union(u) => {
            for field in &u.fields.named {
                for attr in &field.attrs {
                    if attr.path().is_ident("tag") {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "field-level `#[tag(...)]` is not allowed on a type with `#[tagged(via(...))]` — \
                             fields of a `via`-delegating type are wire-irrelevant; \
                             tag the wire DTO's fields instead",
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Stub expansion: empty `Tagged::Product`, no-op `register_into`. Used for
/// shapes the macro hasn't learned to handle yet.
fn stub(input: &DeriveInput) -> TokenStream2 {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let tagged = empty_product_literal();
    quote! {
        impl #impl_generics ::msgpack_tagged::MsgpackTagged for #name #ty_generics #where_clause {
            const TAGGED: ::msgpack_tagged::Tagged = #tagged;
            fn register_into(_reg: &mut ::msgpack_tagged::TagRegistry) {}
        }
    }
}

/// Per-tagged-field info collected during macro expansion. `name` is the
/// field's wire-name as a string — for named structs that's the field
/// identifier; for tuple structs it's the source-position-as-string ("0",
/// "1", …). Either way, the name lands in the `Product`'s `fields` slice
/// as a `&'static str` literal.
struct TaggedField<'a> {
    tag: u8,
    name: String,
    ty: &'a Type,
    has_default: bool,
}

/// Parse a list of named fields (struct fields or named-variant payload
/// fields) into the per-tagged-field entries that drive `Product` emission.
/// Every field needs an explicit `#[tag(N)]` or auto-skips via `#[tag(skip)]`
/// / `PhantomData<_>`; missing both is a compile error. The returned vec is
/// already in tag-ascending order, the canonical wire order.
fn parse_named_fields<'a>(
    fields: &'a Punctuated<Field, Token![,]>,
    reserved: &[u8],
) -> syn::Result<Vec<TaggedField<'a>>> {
    let mut entries = Vec::with_capacity(fields.len());
    for field in fields {
        let ident = field.ident.as_ref().expect("named field has an ident");
        match classify_field(field, reserved)? {
            FieldKind::Tagged { tag, has_default } => {
                // Field-level `#[serde(rename = "X")]` overrides the wire
                // name. This is what makes the shadow-DTO pattern work when
                // the wire DTO uses a different field name than the public
                // type — `serialize_field("X", ...)` matches our `tag_for("X")`.
                let wire_name =
                    parse_serde_field_rename(field)?.unwrap_or_else(|| ident.to_string());
                entries.push(TaggedField { tag, name: wire_name, ty: &field.ty, has_default });
            }
            FieldKind::Skipped => {}
        }
    }
    entries.sort_by_key(|e| e.tag);
    Ok(entries)
}

/// Parse a list of unnamed (positional) fields — top-level tuple-struct
/// fields or tuple-variant payload fields. Tagging style must be uniform:
/// either every field carries `#[tag(N)]` (explicit, allows reordering /
/// `default`) or none do (implicit positional 0, 1, 2, …). Mixing is
/// rejected. The returned vec is in tag-ascending order with names being
/// the position-as-string ("0", "1", …).
///
/// `mixing_error_span` controls where the "mixing implicit and explicit
/// is rejected" error is anchored — typically the surrounding `DeriveInput`
/// for top-level tuple structs or the variant for variant payloads.
fn parse_tuple_fields<'a>(
    mixing_error_span: &dyn ToTokens,
    fields: &'a Punctuated<Field, Token![,]>,
    reserved: &[u8],
) -> syn::Result<Vec<TaggedField<'a>>> {
    let explicit_count =
        fields.iter().filter(|f| f.attrs.iter().any(|a| a.path().is_ident("tag"))).count();
    if explicit_count != 0 && explicit_count != fields.len() {
        return Err(syn::Error::new_spanned(
            mixing_error_span,
            "tuple-style fields must either all carry `#[tag(N)]` or none — \
             mixing implicit positional tags with explicit tags is rejected",
        ));
    }
    let all_explicit = explicit_count == fields.len();

    let mut entries = Vec::with_capacity(fields.len());
    for (position, field) in fields.iter().enumerate() {
        let position_u8: u8 = position.try_into().map_err(|_| {
            syn::Error::new_spanned(
                field,
                format!("tuple position {position} is out of range for u8 tags"),
            )
        })?;
        let (tag, has_default) = if all_explicit {
            match classify_field(field, reserved)? {
                FieldKind::Tagged { tag, has_default } => (tag, has_default),
                FieldKind::Skipped => {
                    return Err(syn::Error::new_spanned(
                        field,
                        "`#[tag(skip)]` on tuple-style fields is not supported",
                    ));
                }
            }
        } else {
            // Implicit positional: `#[serde(skip)]` would shift positional
            // indices, same brittleness rationale as `#[tag(skip)]` in the
            // all-explicit branch. Reject it instead of silently honoring it.
            if has_serde_skip(field)? {
                return Err(syn::Error::new_spanned(
                    field,
                    "`#[serde(skip)]` on tuple-style fields is not supported",
                ));
            }
            if reserved.contains(&position_u8) {
                return Err(syn::Error::new_spanned(
                    field,
                    format!(
                        "implicit positional tag {position_u8} collides with the type's \
                         `#[tagged(reserved(...))]` list — assign explicit `#[tag(N)]`s, \
                         or remove the reserved entry"
                    ),
                ));
            }
            (position_u8, false)
        };
        entries.push(TaggedField { tag, name: position.to_string(), ty: &field.ty, has_default });
    }
    entries.sort_by_key(|e| e.tag);
    Ok(entries)
}

fn expand_named_struct(
    input: &DeriveInput,
    fields: &Punctuated<Field, Token![,]>,
    type_attrs: &TypeAttrs,
) -> syn::Result<TokenStream2> {
    reject_sum_only_decode_flags(input, type_attrs)?;
    let name = &input.ident;
    // The registry key is the *serde* name — it must match what
    // `serialize_struct(name, ...)` will pass at runtime. So we honor
    // `#[serde(rename = "...")]` if present, fall back to the Rust ident
    // otherwise. This is what makes the shadow-DTO pattern work: the wire
    // DTO `MemOpWire` with `#[serde(rename = "MemOp")]` registers under
    // `"MemOp"`, and the wrapper's lookup at `serialize_struct("MemOp", ...)`
    // hits correctly.
    let name_str = parse_serde_rename(input)?.unwrap_or_else(|| name.to_string());

    // `via` is handled in `expand` before dispatch — by the time we reach
    // this function, it must be `None`. Reservation list and unknown-tag
    // policy come from the already-parsed type attrs.
    debug_assert!(type_attrs.via.is_none());
    let reserved = &type_attrs.reserved;
    let allow_unknown_tags = type_attrs.allow_unknown_tags;

    // Parse each field. Tagged fields contribute to TAGS, the recursion list,
    // and the where clause. Skipped fields (`#[tag(skip)]` or `PhantomData<_>`)
    // are silently dropped — they don't go on the wire and don't constrain
    // their type.
    let entries = parse_named_fields(fields, reserved)?;

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
    let tagged = product_literal(&entries, reserved, allow_unknown_tags);
    let where_clause = build_where_clause(input, &entries);
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::msgpack_tagged::MsgpackTagged for #name #ty_generics #where_clause {
            const TAGGED: ::msgpack_tagged::Tagged = #tagged;

            fn register_into(_reg: &mut ::msgpack_tagged::TagRegistry) {
                if _reg.try_insert::<Self>(#name_str) {
                    #(#recursion_calls)*
                }
            }
        }
    })
}

/// Expand the `#[tagged(via(WireType))]` form: the public type delegates
/// `register_into` entirely to the wire DTO and contributes no entry of its
/// own. The emitted `TAGGED` is an empty `Tagged::Product` purely to
/// satisfy the trait — it's never consulted, because the public type itself
/// never appears in the registry.
fn expand_via(input: &DeriveInput, wire_type: &Type) -> TokenStream2 {
    let name = &input.ident;
    let where_clause = build_via_where_clause(input, wire_type);
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();
    let tagged = empty_product_literal();

    quote! {
        impl #impl_generics ::msgpack_tagged::MsgpackTagged for #name #ty_generics #where_clause {
            const TAGGED: ::msgpack_tagged::Tagged = #tagged;

            fn register_into(_reg: &mut ::msgpack_tagged::TagRegistry) {
                <#wire_type as ::msgpack_tagged::MsgpackTagged>::register_into(_reg);
            }
        }
    }
}

/// Build the where clause for a `via`-delegating impl. The public type
/// contributes no field-type bounds (it has no field types on the wire), but
/// it does need:
/// 1. `T: 'static` on every type parameter (the supertrait propagates `Self: 'static`).
/// 2. `<WireType>: MsgpackTagged` so the recursive call type-checks.
fn build_via_where_clause(input: &DeriveInput, wire_type: &Type) -> Option<WhereClause> {
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
    where_clause.predicates.push(parse_quote!(#wire_type: ::msgpack_tagged::MsgpackTagged));
    Some(where_clause)
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
/// strict-by-default discipline. Also enforces that an active `#[tag(N)]`
/// doesn't collide with the surrounding `#[tagged(reserved(...))]` list,
/// and that `#[tag(N)]` and `#[serde(skip)]` aren't both set on the same
/// field (those are contradictory — one says "on the wire", the other
/// "not on the wire").
fn classify_field(field: &Field, reserved: &[u8]) -> syn::Result<FieldKind> {
    let serde_skip = has_serde_skip(field)?;
    let mut found: Option<(&Attribute, TagArgs)> = None;
    for attr in &field.attrs {
        if !attr.path().is_ident("tag") {
            continue;
        }
        if found.is_some() {
            return Err(syn::Error::new_spanned(attr, "duplicate `#[tag(...)]` attribute"));
        }
        found = Some((attr, attr.parse_args()?));
    }

    // Explicit annotation wins over auto-skip — if the user explicitly tags a
    // PhantomData field with `#[tag(N)]`, we honor that (unusual but valid).
    if let Some((attr, args)) = found {
        return match args {
            TagArgs::Tag { tag, default } => {
                if serde_skip {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "field has both `#[tag(N)]` and `#[serde(skip)]` — these are \
                         contradictory; pick one (use `#[tag(skip)]` for our skip semantics, \
                         or `#[tag(N)]` to put the field on the wire under tag N)",
                    ));
                }
                if reserved.contains(&tag) {
                    return Err(syn::Error::new_spanned(
                        attr,
                        format!(
                            "tag {tag} is in the surrounding `#[tagged(reserved(...))]` list — pick a different tag, or remove it from the reserved list"
                        ),
                    ));
                }
                Ok(FieldKind::Tagged { tag, has_default: default })
            }
            TagArgs::Skip => Ok(FieldKind::Skipped),
        };
    }

    // No `#[tag(...)]` at all — `#[serde(skip)]` is recognized as an
    // alias for `#[tag(skip)]`; otherwise auto-skip for `PhantomData<_>`
    // (the conventional zero-sized "use a type parameter without storing
    // anything" pattern). Any other untagged field is an error.
    if serde_skip {
        return Ok(FieldKind::Skipped);
    }
    if is_phantom_data(&field.ty) {
        return Ok(FieldKind::Skipped);
    }

    Err(syn::Error::new_spanned(
        field,
        "missing `#[tag(N)]` attribute — every field needs an explicit tag, \
         `#[tag(skip)]`, `#[serde(skip)]`, or be `PhantomData<_>`",
    ))
}

/// Read `#[serde(rename = "X")]` off a list of attributes, if present, and
/// return `"X"`. Used both at the type level (the returned name becomes the
/// registry key) and at the field level (the returned name becomes the
/// `Product.fields` wire-name for that field).
///
/// Only the simple symmetric form `rename = "X"` is recognized. Other serde
/// items (`default`, `skip`, `rename_all`, asymmetric `rename(serialize = ...,
/// deserialize = ...)`, etc.) are ignored. If the user has multiple
/// `#[serde(rename = "X")]` attributes that disagree, the last one wins
/// (matches serde's own behavior).
fn parse_serde_rename_in_attrs(attrs: &[Attribute]) -> syn::Result<Option<String>> {
    let mut found: Option<String> = None;
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        let items: Punctuated<Meta, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;
        for item in items {
            if let Meta::NameValue(nv) = &item
                && nv.path.is_ident("rename")
                && let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = &nv.value
            {
                found = Some(s.value());
            }
        }
    }
    Ok(found)
}

/// Type-level `#[serde(rename = "X")]` — used as the registry key for a
/// type, so it matches what `serialize_struct(name, ...)` passes at runtime
/// through the auto-derived `Serialize` impl.
fn parse_serde_rename(input: &DeriveInput) -> syn::Result<Option<String>> {
    parse_serde_rename_in_attrs(&input.attrs)
}

/// Field-level `#[serde(rename = "X")]` — used as the wire-name in
/// `Product.fields` for that field, matching what `serialize_field("X", ...)`
/// passes at runtime through the auto-derived `Serialize` impl. The
/// load-bearing piece for the shadow-DTO pattern when the wire DTO renames
/// individual fields (e.g., `index` → `i`).
fn parse_serde_field_rename(field: &Field) -> syn::Result<Option<String>> {
    parse_serde_rename_in_attrs(&field.attrs)
}

/// Whether a field carries `#[serde(skip)]` — recognized by the macro as an
/// alias for `#[tag(skip)]`. Only the bare-ident form is honored;
/// asymmetric `skip_serializing` / `skip_deserializing` and conditional
/// `skip_serializing_if = "..."` are deliberately ignored, since they don't
/// have a clean encode-and-decode-symmetric mapping in this format.
fn has_serde_skip(field: &Field) -> syn::Result<bool> {
    for attr in &field.attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        let items: Punctuated<Meta, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;
        for item in items {
            if let Meta::Path(path) = &item
                && path.is_ident("skip")
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Variant-level configuration parsed from one or more `#[tagged(...)]`
/// attributes on an enum variant. The grammar is a strict subset of the
/// type-level grammar — only `reserved(...)` and `allow_unknown_tags`
/// apply to a variant payload (which is shape-equivalent to a struct).
/// `default_on_reserved`, `default_on_unknown`, and `via(...)` are
/// sum-level decisions that have no place on individual variants.
#[derive(Default)]
struct VariantAttrs {
    reserved: Vec<u8>,
    allow_unknown_tags: bool,
}

/// Parse the variant-level `#[tagged(...)]` attributes (if any) into a
/// `VariantAttrs`. Multiple `#[tagged(...)]` attributes on the same variant
/// are allowed and merged, but each named modifier may appear at most once
/// across them.
fn parse_tagged_variant_attrs(variant: &Variant) -> syn::Result<VariantAttrs> {
    let mut out = VariantAttrs::default();
    let mut seen_reserved = false;
    let mut seen_allow_unknown = false;

    for attr in &variant.attrs {
        if !attr.path().is_ident("tagged") {
            continue;
        }
        let items: Punctuated<Meta, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;
        for item in items {
            if let Meta::List(list) = &item
                && list.path.is_ident("reserved")
            {
                if seen_reserved {
                    return Err(syn::Error::new_spanned(
                        list,
                        "duplicate `reserved(...)` modifier in `#[tagged(...)]`",
                    ));
                }
                seen_reserved = true;
                let lits: Punctuated<LitInt, Token![,]> =
                    list.parse_args_with(Punctuated::parse_terminated)?;
                let mut seen_dup = std::collections::HashSet::new();
                for lit in &lits {
                    let n: u8 = lit.base10_parse()?;
                    if !seen_dup.insert(n) {
                        return Err(syn::Error::new_spanned(
                            lit,
                            format!("tag {n} listed more than once in `reserved(...)`"),
                        ));
                    }
                    out.reserved.push(n);
                }
                continue;
            }
            if let Meta::Path(path) = &item
                && path.is_ident("allow_unknown_tags")
            {
                if seen_allow_unknown {
                    return Err(syn::Error::new_spanned(
                        path,
                        "duplicate `allow_unknown_tags` modifier in `#[tagged(...)]`",
                    ));
                }
                seen_allow_unknown = true;
                out.allow_unknown_tags = true;
                continue;
            }
            return Err(syn::Error::new_spanned(
                &item,
                "expected `reserved(...)` or `allow_unknown_tags` inside `#[tagged(...)]` \
                 on an enum variant — `default_on_reserved`, `default_on_unknown`, and \
                 `via(...)` are type-level modifiers, not variant-level",
            ));
        }
    }
    Ok(out)
}

/// Type-level configuration parsed from one or more `#[tagged(...)]`
/// attributes on the struct/enum. Holds every modifier the macro understands
/// at the type level.
#[derive(Default)]
struct TypeAttrs {
    /// Tags listed in `#[tagged(reserved(N, M, ...))]`. Empty if absent.
    reserved: Vec<u8>,
    /// `true` iff `#[tagged(allow_unknown_tags)]` appears anywhere. Applies
    /// to product (struct) shapes only — sums reject it (no skip semantics
    /// for an unknown variant tag).
    allow_unknown_tags: bool,
    /// `true` iff `#[tagged(default_on_reserved)]` appears anywhere. Sum-only
    /// runtime decode policy: substitute `T::default()` when the encoded
    /// variant tag is in `reserved`.
    default_on_reserved: bool,
    /// `true` iff `#[tagged(default_on_unknown)]` appears anywhere. Sum-only
    /// runtime decode policy: substitute `T::default()` when the encoded
    /// variant tag is in neither `variants` nor `reserved`.
    default_on_unknown: bool,
    /// The wire DTO from `#[tagged(via(WireType))]`, if present. When set,
    /// the public type delegates `register_into` to this type and contributes
    /// no entry of its own to the registry. Mutually exclusive with every
    /// other type-level modifier — those are wire-format properties and
    /// belong on the wire DTO.
    via: Option<Type>,
}

/// Parse the type-level `#[tagged(...)]` attributes (if any) into a single
/// `TypeAttrs`. Multiple `#[tagged(...)]` attributes are allowed and merged,
/// but each named modifier may appear at most once across them.
///
/// Inner grammar — comma-separated items, each one of:
/// * `reserved(N, M, ...)` — list-form, integer literals, no duplicates.
/// * `allow_unknown_tags` — bare ident, presence-only. Product-shapes only.
/// * `default_on_reserved` — bare ident, presence-only. Sum-shapes only.
/// * `default_on_unknown` — bare ident, presence-only. Sum-shapes only.
/// * `via(WireType)` — list-form, single Rust type (the wire DTO to delegate
///   `register_into` to). Mutually exclusive with every other modifier —
///   those properties belong on the wire DTO.
fn parse_tagged_type_attrs(input: &DeriveInput) -> syn::Result<TypeAttrs> {
    let mut out = TypeAttrs::default();
    let mut seen_reserved = false;
    let mut seen_allow_unknown = false;
    let mut seen_default_on_reserved = false;
    let mut seen_default_on_unknown = false;
    let mut seen_via = false;

    for attr in &input.attrs {
        if !attr.path().is_ident("tagged") {
            continue;
        }
        let items: Punctuated<Meta, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;
        for item in items {
            if let Meta::List(list) = &item
                && list.path.is_ident("reserved")
            {
                if seen_reserved {
                    return Err(syn::Error::new_spanned(
                        list,
                        "duplicate `reserved(...)` modifier in `#[tagged(...)]`",
                    ));
                }
                seen_reserved = true;
                let lits: Punctuated<LitInt, Token![,]> =
                    list.parse_args_with(Punctuated::parse_terminated)?;
                let mut seen_dup = std::collections::HashSet::new();
                for lit in &lits {
                    let n: u8 = lit.base10_parse()?;
                    if !seen_dup.insert(n) {
                        return Err(syn::Error::new_spanned(
                            lit,
                            format!("tag {n} listed more than once in `reserved(...)`"),
                        ));
                    }
                    out.reserved.push(n);
                }
                continue;
            }
            if let Meta::Path(path) = &item
                && path.is_ident("allow_unknown_tags")
            {
                if seen_allow_unknown {
                    return Err(syn::Error::new_spanned(
                        path,
                        "duplicate `allow_unknown_tags` modifier in `#[tagged(...)]`",
                    ));
                }
                seen_allow_unknown = true;
                out.allow_unknown_tags = true;
                continue;
            }
            if let Meta::Path(path) = &item
                && path.is_ident("default_on_reserved")
            {
                if seen_default_on_reserved {
                    return Err(syn::Error::new_spanned(
                        path,
                        "duplicate `default_on_reserved` modifier in `#[tagged(...)]`",
                    ));
                }
                seen_default_on_reserved = true;
                out.default_on_reserved = true;
                continue;
            }
            if let Meta::Path(path) = &item
                && path.is_ident("default_on_unknown")
            {
                if seen_default_on_unknown {
                    return Err(syn::Error::new_spanned(
                        path,
                        "duplicate `default_on_unknown` modifier in `#[tagged(...)]`",
                    ));
                }
                seen_default_on_unknown = true;
                out.default_on_unknown = true;
                continue;
            }
            if let Meta::List(list) = &item
                && list.path.is_ident("via")
            {
                if seen_via {
                    return Err(syn::Error::new_spanned(
                        list,
                        "duplicate `via(...)` modifier in `#[tagged(...)]`",
                    ));
                }
                seen_via = true;
                out.via = Some(list.parse_args::<Type>()?);
                continue;
            }
            return Err(syn::Error::new_spanned(
                &item,
                "expected `reserved(...)`, `allow_unknown_tags`, `default_on_reserved`, \
                 `default_on_unknown`, or `via(...)` inside `#[tagged(...)]` on a type",
            ));
        }
    }

    // `via` is wire-format-agnostic delegation — the wire DTO carries every
    // wire-format property, the public type carries none.
    if out.via.is_some() {
        if seen_reserved {
            return Err(syn::Error::new_spanned(
                input,
                "`#[tagged(via(...))]` is incompatible with `reserved(...)` — \
                 the reserved-tag list belongs on the wire DTO, not on the public type",
            ));
        }
        if seen_allow_unknown {
            return Err(syn::Error::new_spanned(
                input,
                "`#[tagged(via(...))]` is incompatible with `allow_unknown_tags` — \
                 that flag belongs on the wire DTO, not on the public type",
            ));
        }
        if seen_default_on_reserved {
            return Err(syn::Error::new_spanned(
                input,
                "`#[tagged(via(...))]` is incompatible with `default_on_reserved` — \
                 decode-policy flags belong on the wire DTO, not on the public type",
            ));
        }
        if seen_default_on_unknown {
            return Err(syn::Error::new_spanned(
                input,
                "`#[tagged(via(...))]` is incompatible with `default_on_unknown` — \
                 decode-policy flags belong on the wire DTO, not on the public type",
            ));
        }
    }

    Ok(out)
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
