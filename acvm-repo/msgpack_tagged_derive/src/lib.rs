//! Companion proc-macro crate for [`msgpack_tagged`].
//!
//! Currently provides a **stub** `MsgpackTagged` derive: it accepts any struct
//! or enum and emits a syntactically valid `MsgpackTagged` impl with empty
//! `TAGS` / `RESERVED` and a no-op `register_into`. Real expansion logic
//! (field/variant tagging, recursive registration, governance checks, etc.)
//! lands in subsequent steps; this scaffold exists so the rest of the design
//! has a working derive endpoint to point at.
//!
//! Design: [issue #12554](https://github.com/noir-lang/noir/issues/12554).

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(MsgpackTagged, attributes(tag, reserved, allow_unknown_tags, via))]
pub fn derive_msgpack_tagged(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Stub expansion: empty TAGS, empty RESERVED, no-op register_into.
    // Real expansion (read `#[tag(N)]` / `#[reserved(...)]` / `#[via(...)]`,
    // emit field-recursing `register_into`, propagate `T: MsgpackTagged`
    // bounds onto generic params) is added in the next step.
    let expanded = quote! {
        impl #impl_generics ::msgpack_tagged::MsgpackTagged for #name #ty_generics #where_clause {
            const TAGS: &'static [(::msgpack_tagged::Tag, &'static str)] = &[];
            const RESERVED: &'static [::msgpack_tagged::Tag] = &[];
            const ALLOW_UNKNOWN_TAGS: bool = false;
            fn register_into(_reg: &mut ::msgpack_tagged::TagRegistry) {}
        }
    };

    TokenStream::from(expanded)
}
