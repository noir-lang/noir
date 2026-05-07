//! Blanket [`MsgpackTagged`] impls for stdlib container types.
//!
//! Containers don't have struct fields or enum variants of their own, so they
//! never go through `serialize_struct` / `deserialize_struct` and don't appear
//! as registry entries. But unlike primitives, their `register_into` is *not*
//! a no-op: it propagates the recursive walk into the inner type(s) so any
//! nested `MsgpackTagged` types contained within reach the registry.
//!
//! Deliberately omitted: `HashMap` and `HashSet`. Their iteration order is
//! non-deterministic, which would produce different wire bytes for equal values
//! across runs. Wire types should use `BTreeMap` / `BTreeSet` instead. If
//! someone reaches for `HashMap` on the wire, the missing impl is a compile
//! error — and the `#[diagnostic::on_unimplemented]` note on the trait
//! definition points them at the `BTreeMap` alternative.

use std::collections::{BTreeMap, BTreeSet};

use crate::{MsgpackTagged, Product, TagRegistry, Tagged};

const LEAF: Tagged = Tagged::Product(Product {
    fields: &[],
    reserved: &[],
    defaults: &[],
    allow_unknown_tags: false,
});

impl<T: MsgpackTagged> MsgpackTagged for Vec<T> {
    const TAGGED: Tagged = LEAF;
    fn register_into(reg: &mut TagRegistry) {
        T::register_into(reg);
    }
}

impl<T: MsgpackTagged, const N: usize> MsgpackTagged for [T; N] {
    const TAGGED: Tagged = LEAF;
    fn register_into(reg: &mut TagRegistry) {
        T::register_into(reg);
    }
}

impl<T: MsgpackTagged> MsgpackTagged for Option<T> {
    const TAGGED: Tagged = LEAF;
    fn register_into(reg: &mut TagRegistry) {
        T::register_into(reg);
    }
}

impl<T: MsgpackTagged> MsgpackTagged for Box<T> {
    const TAGGED: Tagged = LEAF;
    fn register_into(reg: &mut TagRegistry) {
        T::register_into(reg);
    }
}

impl<K: MsgpackTagged, V: MsgpackTagged> MsgpackTagged for BTreeMap<K, V> {
    const TAGGED: Tagged = LEAF;
    fn register_into(reg: &mut TagRegistry) {
        K::register_into(reg);
        V::register_into(reg);
    }
}

impl<T: MsgpackTagged> MsgpackTagged for BTreeSet<T> {
    const TAGGED: Tagged = LEAF;
    fn register_into(reg: &mut TagRegistry) {
        T::register_into(reg);
    }
}

macro_rules! impl_msgpack_tagged_for_tuple {
    ($($t:ident),+ $(,)?) => {
        impl<$($t: MsgpackTagged),+> MsgpackTagged for ($($t,)+) {
            const TAGGED: Tagged = LEAF;
            fn register_into(reg: &mut TagRegistry) {
                $($t::register_into(reg);)+
            }
        }
    };
}

impl_msgpack_tagged_for_tuple!(T0);
impl_msgpack_tagged_for_tuple!(T0, T1);
impl_msgpack_tagged_for_tuple!(T0, T1, T2);
impl_msgpack_tagged_for_tuple!(T0, T1, T2, T3);
impl_msgpack_tagged_for_tuple!(T0, T1, T2, T3, T4);
impl_msgpack_tagged_for_tuple!(T0, T1, T2, T3, T4, T5);
impl_msgpack_tagged_for_tuple!(T0, T1, T2, T3, T4, T5, T6);
impl_msgpack_tagged_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_msgpack_tagged_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_msgpack_tagged_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_msgpack_tagged_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_msgpack_tagged_for_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);

#[cfg(test)]
mod tests {
    use super::*;

    /// A `MsgpackTagged` type that *does* register itself, used to prove
    /// containers correctly propagate the recursion.
    struct Foo;
    impl MsgpackTagged for Foo {
        const TAGGED: Tagged = Tagged::Product(Product {
            fields: &[(0, "x")],
            reserved: &[],
            defaults: &[],
            allow_unknown_tags: false,
        });
        fn register_into(reg: &mut TagRegistry) {
            reg.try_insert::<Foo>("Foo");
        }
    }

    /// Another self-registering type, for two-recursion tests (maps, tuples).
    struct Bar;
    impl MsgpackTagged for Bar {
        const TAGGED: Tagged = Tagged::Product(Product {
            fields: &[(0, "y")],
            reserved: &[],
            defaults: &[],
            allow_unknown_tags: false,
        });
        fn register_into(reg: &mut TagRegistry) {
            reg.try_insert::<Bar>("Bar");
        }
    }

    /// Compile-time check that each container has an impl. If any impl above
    /// is removed, this fails to build.
    #[test]
    fn each_container_satisfies_the_trait_bound() {
        fn assert_impl<T: MsgpackTagged>() {}
        assert_impl::<Vec<u32>>();
        assert_impl::<[u32; 4]>();
        assert_impl::<Option<u32>>();
        assert_impl::<Box<u32>>();
        assert_impl::<BTreeMap<String, u32>>();
        assert_impl::<BTreeSet<u32>>();
        assert_impl::<(u32,)>();
        assert_impl::<(u32, u32)>();
        assert_impl::<(u32, u32, u32)>();
        assert_impl::<(u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32)>();
    }

    #[test]
    fn vec_recurses_into_inner() {
        let mut reg = TagRegistry::new();
        <Vec<Foo>>::register_into(&mut reg);
        assert_eq!(reg.len(), 1);
        assert!(reg.get("Foo").is_some());
    }

    #[test]
    fn option_recurses_into_inner() {
        let mut reg = TagRegistry::new();
        <Option<Foo>>::register_into(&mut reg);
        assert!(reg.get("Foo").is_some());
    }

    #[test]
    fn array_recurses_into_inner() {
        let mut reg = TagRegistry::new();
        <[Foo; 7]>::register_into(&mut reg);
        assert!(reg.get("Foo").is_some());
    }

    #[test]
    fn box_recurses_into_inner() {
        let mut reg = TagRegistry::new();
        <Box<Foo>>::register_into(&mut reg);
        assert!(reg.get("Foo").is_some());
    }

    #[test]
    fn btreemap_recurses_into_key_and_value() {
        let mut reg = TagRegistry::new();
        <BTreeMap<Foo, Bar>>::register_into(&mut reg);
        assert_eq!(reg.len(), 2);
        assert!(reg.get("Foo").is_some());
        assert!(reg.get("Bar").is_some());
    }

    #[test]
    fn btreeset_recurses_into_inner() {
        let mut reg = TagRegistry::new();
        <BTreeSet<Foo>>::register_into(&mut reg);
        assert!(reg.get("Foo").is_some());
    }

    #[test]
    fn tuple_recurses_into_each_element() {
        let mut reg = TagRegistry::new();
        <(Foo, Bar)>::register_into(&mut reg);
        assert!(reg.get("Foo").is_some());
        assert!(reg.get("Bar").is_some());
    }

    /// Recursion is transitive: `Vec<Option<Foo>>` reaches `Foo` through two
    /// container layers, and `Foo` is registered exactly once.
    #[test]
    fn nested_containers_recurse_through() {
        let mut reg = TagRegistry::new();
        <Vec<Option<Foo>>>::register_into(&mut reg);
        assert_eq!(reg.len(), 1);
        assert!(reg.get("Foo").is_some());
    }

    /// Containers wrapping primitives leave the registry empty — primitives
    /// don't register themselves, and the container has no entry of its own.
    #[test]
    fn container_of_primitive_doesnt_populate_registry() {
        let mut reg = TagRegistry::new();
        <Vec<u32>>::register_into(&mut reg);
        <Option<bool>>::register_into(&mut reg);
        <BTreeMap<String, u64>>::register_into(&mut reg);
        <(u32, u64, bool)>::register_into(&mut reg);
        assert!(reg.is_empty());
    }
}
