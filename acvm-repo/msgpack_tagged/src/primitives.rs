//! Blanket [`MsgpackTagged`] impls for scalar primitives.
//!
//! Primitives don't have struct fields or enum variants, so they have no tags
//! and never go through `serialize_struct` / `deserialize_struct`. Their
//! `register_into` is a no-op — they exist only to satisfy the `T: MsgpackTagged`
//! bound that the macro propagates onto every tagged field's type.

use crate::{MsgpackTagged, Tag, TagRegistry};

macro_rules! impl_msgpack_tagged_for_primitive {
    ($($t:ty),* $(,)?) => {
        $(
            impl MsgpackTagged for $t {
                const TAGS: &'static [(Tag, &'static str)] = &[];
                fn register_into(_reg: &mut TagRegistry) {}
            }
        )*
    };
}

impl_msgpack_tagged_for_primitive!(
    (),
    bool,
    char,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    String,
);

#[cfg(test)]
mod tests {
    use super::*;

    /// The mere fact that these calls type-check proves each primitive has a
    /// `MsgpackTagged` impl. If any primitive impl gets removed (or an entry
    /// gets dropped from the macro invocation above), this fails to build.
    #[test]
    fn each_primitive_satisfies_the_trait_bound() {
        fn assert_impl<T: MsgpackTagged>() {}
        assert_impl::<()>();
        assert_impl::<bool>();
        assert_impl::<char>();
        assert_impl::<u8>();
        assert_impl::<u16>();
        assert_impl::<u32>();
        assert_impl::<u64>();
        assert_impl::<u128>();
        assert_impl::<usize>();
        assert_impl::<i8>();
        assert_impl::<i16>();
        assert_impl::<i32>();
        assert_impl::<i64>();
        assert_impl::<i128>();
        assert_impl::<isize>();
        assert_impl::<f32>();
        assert_impl::<f64>();
        assert_impl::<String>();
    }

    /// Primitives must not put themselves in the registry — only struct/enum
    /// types that need name → tag lookups do. If a primitive's `register_into`
    /// ever called `try_insert`, the registry would carry pointless entries
    /// and the wrapper's strict-on-unknown panic could fire on innocuous types.
    #[test]
    fn primitives_dont_register_themselves() {
        let mut reg = TagRegistry::new();
        <()>::register_into(&mut reg);
        <bool>::register_into(&mut reg);
        <u32>::register_into(&mut reg);
        <i64>::register_into(&mut reg);
        <f64>::register_into(&mut reg);
        <String>::register_into(&mut reg);
        assert!(reg.is_empty());
    }
}
