//! Companion proc-macro crate for [`msgpack_tagged`].
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
use quote::quote;
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

/// Build a `Tagged::Product { ... }` literal from parsed field entries plus
/// the type-level reserved list and unknown-tag policy. Used for named
/// structs, tuple structs, and (eventually) enum variant payloads.
fn product_literal(
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
        ::msgpack_tagged::Tagged::Product(::msgpack_tagged::Product {
            fields: &[#(#field_entries),*],
            reserved: &[#(#reserved_entries),*],
            defaults: &[#(#default_entries),*],
            allow_unknown_tags: #allow_unknown_tags,
        })
    }
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

/// Build a `Tagged::Sum` literal from variant entries and the
/// enum-level reserved variant-tag list. Each variant carries an *empty*
/// payload [`Product`] in this iteration — per-variant field tags are the
/// next incremental step and will populate the payload's `fields`.
fn sum_literal(variants: &[TaggedVariant<'_>], reserved: &[u8]) -> TokenStream2 {
    let variant_entries = variants.iter().map(|v| {
        let tag = v.tag;
        let name = &v.name;
        quote! {
            ::msgpack_tagged::Variant {
                tag: #tag,
                name: #name,
                payload: ::msgpack_tagged::Product {
                    fields: &[],
                    reserved: &[],
                    defaults: &[],
                    allow_unknown_tags: false,
                },
            }
        }
    });
    let reserved_entries = reserved.iter().map(|tag| quote! { #tag });
    quote! {
        ::msgpack_tagged::Tagged::Sum(::msgpack_tagged::Sum {
            variants: &[#(#variant_entries),*],
            reserved: &[#(#reserved_entries),*],
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
    let name = &input.ident;
    let name_str = parse_serde_rename(input)?.unwrap_or_else(|| name.to_string());
    let reserved = &type_attrs.reserved;
    let allow_unknown_tags = type_attrs.allow_unknown_tags;

    // First pass: count how many fields have an explicit `#[tag(...)]`.
    // Mixing implicit and explicit is rejected.
    let explicit_count =
        fields.iter().filter(|f| f.attrs.iter().any(|a| a.path().is_ident("tag"))).count();
    if explicit_count != 0 && explicit_count != fields.len() {
        return Err(syn::Error::new_spanned(
            input,
            "tuple-struct fields must either all carry `#[tag(N)]` or none — \
             mixing implicit positional tags with explicit tags is rejected",
        ));
    }
    let all_explicit = explicit_count == fields.len();

    let mut entries: Vec<TaggedField<'_>> = Vec::with_capacity(fields.len());
    for (position, field) in fields.iter().enumerate() {
        let position_u8: u8 = position.try_into().map_err(|_| {
            syn::Error::new_spanned(
                field,
                format!("tuple-struct position {position} is out of range for u8 tags"),
            )
        })?;
        let (tag, has_default) = if all_explicit {
            match classify_field(field, reserved)? {
                FieldKind::Tagged { tag, has_default } => (tag, has_default),
                FieldKind::Skipped => {
                    return Err(syn::Error::new_spanned(
                        field,
                        "`#[tag(skip)]` on tuple-struct fields is not supported",
                    ));
                }
            }
        } else {
            // Implicit positional: tag = position, no default.
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
/// the variant's wire-name (its Rust ident, as a string). `payload_types`
/// holds the types appearing inside the variant's fields (named or unnamed)
/// — the `register_into` recursion targets, and the source of per-payload
/// `MsgpackTagged` bounds in the where clause.
struct TaggedVariant<'a> {
    tag: u8,
    name: String,
    payload_types: Vec<&'a Type>,
}

/// Enum (`enum E { A, B(...), C { ... } }`). Each variant carries an
/// explicit `#[tag(N)]`; the variant tag is what goes on the wire as the
/// discriminator. The expansion emits a `Tagged::Sum` listing every variant
/// in tag-ascending order, and a `register_into` that registers `Self` and
/// recurses into every variant's payload type so nested `MsgpackTagged`
/// types are reached.
///
/// Per-variant struct/tuple field tagging is the next step — for now the
/// macro emits an *empty* `Product` payload for every variant and rejects
/// `#[tag(...)]` annotations *inside* a variant's payload. `#[tag(skip)]`
/// and the `default` modifier are likewise rejected on variants (no clear
/// semantics).
///
/// `#[tagged(allow_unknown_tags)]` is also rejected on enums: an unknown
/// variant tag has no skip semantics — there's no fragment to skip, since
/// the value's discriminator itself is unknown — so the flag would have
/// nowhere to land in the wire shape.
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
        for field in &variant.fields {
            for attr in &field.attrs {
                if attr.path().is_ident("tag") {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "`#[tag(...)]` on enum variant fields is not yet supported — \
                         only the variant itself can be tagged",
                    ));
                }
            }
        }
        let tag = parse_variant_tag(variant, reserved)?;
        if !seen_tags.insert(tag) {
            return Err(syn::Error::new_spanned(
                variant,
                format!("variant tag {tag} is used more than once"),
            ));
        }
        let payload_types: Vec<&Type> = match &variant.fields {
            Fields::Unit => Vec::new(),
            Fields::Named(named) => named.named.iter().map(|f| &f.ty).collect(),
            Fields::Unnamed(unnamed) => unnamed.unnamed.iter().map(|f| &f.ty).collect(),
        };
        variants.push(TaggedVariant { tag, name: variant.ident.to_string(), payload_types });
    }
    variants.sort_by_key(|v| v.tag);

    let recursion_calls = variants.iter().flat_map(|v| {
        v.payload_types.iter().map(|ty| {
            quote! { <#ty as ::msgpack_tagged::MsgpackTagged>::register_into(_reg); }
        })
    });

    let tagged = sum_literal(&variants, reserved);
    let where_clause = build_enum_where_clause(input, &variants);
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
/// `<PayloadType>: MsgpackTagged` bound for every type appearing in any
/// variant's payload — but enums have no `default`-modifier semantics so
/// we don't emit `Default` bounds.
fn build_enum_where_clause(
    input: &DeriveInput,
    variants: &[TaggedVariant<'_>],
) -> Option<WhereClause> {
    let has_type_params = input.generics.params.iter().any(|p| matches!(p, GenericParam::Type(_)));
    let any_payload = variants.iter().any(|v| !v.payload_types.is_empty());
    if !any_payload && !has_type_params {
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

    let mut seen = std::collections::HashSet::new();
    for v in variants {
        for ty in &v.payload_types {
            let key = quote!(#ty).to_string();
            if seen.insert(key) {
                where_clause.predicates.push(parse_quote!(#ty: ::msgpack_tagged::MsgpackTagged));
            }
        }
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
/// "1", …). Either way, the name lands in `TAGS` as a `&'static str` literal.
struct TaggedField<'a> {
    tag: u8,
    name: String,
    ty: &'a Type,
    has_default: bool,
}

fn expand_named_struct(
    input: &DeriveInput,
    fields: &Punctuated<Field, Token![,]>,
    type_attrs: &TypeAttrs,
) -> syn::Result<TokenStream2> {
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
    let mut entries: Vec<TaggedField<'_>> = Vec::with_capacity(fields.len());
    for field in fields {
        let ident = field.ident.as_ref().expect("named field has an ident");
        match classify_field(field, reserved)? {
            FieldKind::Tagged { tag, has_default } => {
                entries.push(TaggedField {
                    tag,
                    name: ident.to_string(),
                    ty: &field.ty,
                    has_default,
                });
            }
            FieldKind::Skipped => {}
        }
    }
    // Canonical order on the wire is tag-ascending, not source-declaration order.
    entries.sort_by_key(|e| e.tag);

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
/// doesn't collide with the type-level `#[tagged(reserved(...))]` list.
fn classify_field(field: &Field, reserved: &[u8]) -> syn::Result<FieldKind> {
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
                if reserved.contains(&tag) {
                    return Err(syn::Error::new_spanned(
                        attr,
                        format!(
                            "tag {tag} is in the type's `#[tagged(reserved(...))]` list — pick a different tag, or remove it from the reserved list"
                        ),
                    ));
                }
                Ok(FieldKind::Tagged { tag, has_default: default })
            }
            TagArgs::Skip => Ok(FieldKind::Skipped),
        };
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

/// Read `#[serde(rename = "X")]` off the type, if present, and return `"X"`.
/// The returned name becomes the registry key for the type — matching what
/// `serialize_struct(name, ...)` will pass at runtime, so the wrapper's
/// lookup hits correctly.
///
/// Only the simple symmetric form `rename = "X"` is recognized. Other serde
/// items (`default`, `skip`, `rename_all`, asymmetric `rename(serialize = ...,
/// deserialize = ...)`, etc.) are ignored — they don't affect the registry
/// key. If the user has multiple `#[serde(rename = "X")]` attributes that
/// disagree, the last one wins (matches serde's own behavior).
fn parse_serde_rename(input: &DeriveInput) -> syn::Result<Option<String>> {
    let mut found: Option<String> = None;
    for attr in &input.attrs {
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

/// Type-level configuration parsed from one or more `#[tagged(...)]`
/// attributes on the struct/enum. Holds every modifier the macro understands
/// at the type level.
#[derive(Default)]
struct TypeAttrs {
    /// Tags listed in `#[tagged(reserved(N, M, ...))]`. Empty if absent.
    reserved: Vec<u8>,
    /// `true` iff `#[tagged(allow_unknown_tags)]` appears anywhere.
    allow_unknown_tags: bool,
    /// The wire DTO from `#[tagged(via(WireType))]`, if present. When set,
    /// the public type delegates `register_into` to this type and contributes
    /// no entry of its own to the registry. Mutually exclusive with `reserved`
    /// and `allow_unknown_tags` (those are wire-format properties and belong
    /// on the wire DTO).
    via: Option<Type>,
}

/// Parse the type-level `#[tagged(...)]` attributes (if any) into a single
/// `TypeAttrs`. Multiple `#[tagged(...)]` attributes are allowed and merged,
/// but each named modifier may appear at most once across them.
///
/// Inner grammar — comma-separated items, each one of:
/// * `reserved(N, M, ...)` — list-form, integer literals, no duplicates.
/// * `allow_unknown_tags` — bare ident, presence-only.
/// * `via(WireType)` — list-form, single Rust type (the wire DTO to delegate
///   `register_into` to). Mutually exclusive with `reserved` and
///   `allow_unknown_tags` — those properties belong on the wire DTO.
fn parse_tagged_type_attrs(input: &DeriveInput) -> syn::Result<TypeAttrs> {
    let mut out = TypeAttrs::default();
    let mut seen_reserved = false;
    let mut seen_allow_unknown = false;
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
                "expected `reserved(...)`, `allow_unknown_tags`, or `via(...)` inside `#[tagged(...)]` on a type",
            ));
        }
    }

    // `via` is wire-format-agnostic delegation — the wire DTO carries
    // `reserved` / `allow_unknown_tags` if anyone does.
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
