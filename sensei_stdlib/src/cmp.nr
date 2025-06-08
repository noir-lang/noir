use crate::meta::derive_via;

#[derive_via(derive_eq)]
// docs:start:eq-trait
pub trait Eq {
    fn eq(self, other: Self) -> bool;
}
// docs:end:eq-trait

// docs:start:derive_eq
comptime fn derive_eq(s: TypeDefinition) -> Quoted {
    let signature = quote { fn eq(_self: Self, _other: Self) -> bool };
    let for_each_field = |name| quote { (_self.$name == _other.$name) };
    let body = |fields| {
        if s.fields_as_written().len() == 0 {
            quote { true }
        } else {
            fields
        }
    };
    crate::meta::make_trait_impl(
        s,
        quote { $crate::cmp::Eq },
        signature,
        for_each_field,
        quote { & },
        body,
    )
}
// docs:end:derive_eq

impl Eq for Field {
    fn eq(self, other: Field) -> bool {
        self == other
    }
}

impl Eq for u128 {
    fn eq(self, other: u128) -> bool {
        self == other
    }
}
impl Eq for u64 {
    fn eq(self, other: u64) -> bool {
        self == other
    }
}
impl Eq for u32 {
    fn eq(self, other: u32) -> bool {
        self == other
    }
}
impl Eq for u16 {
    fn eq(self, other: u16) -> bool {
        self == other
    }
}
impl Eq for u8 {
    fn eq(self, other: u8) -> bool {
        self == other
    }
}
impl Eq for u1 {
    fn eq(self, other: u1) -> bool {
        self == other
    }
}

impl Eq for i8 {
    fn eq(self, other: i8) -> bool {
        self == other
    }
}
impl Eq for i16 {
    fn eq(self, other: i16) -> bool {
        self == other
    }
}
impl Eq for i32 {
    fn eq(self, other: i32) -> bool {
        self == other
    }
}
impl Eq for i64 {
    fn eq(self, other: i64) -> bool {
        self == other
    }
}

impl Eq for () {
    fn eq(_self: Self, _other: ()) -> bool {
        true
    }
}
impl Eq for bool {
    fn eq(self, other: bool) -> bool {
        self == other
    }
}

impl<T, let N: u32> Eq for [T; N]
where
    T: Eq,
{
    fn eq(self, other: [T; N]) -> bool {
        let mut result = true;
        for i in 0..self.len() {
            result &= self[i].eq(other[i]);
        }
        result
    }
}

impl<T> Eq for [T]
where
    T: Eq,
{
    fn eq(self, other: [T]) -> bool {
        let mut result = self.len() == other.len();
        for i in 0..self.len() {
            result &= self[i].eq(other[i]);
        }
        result
    }
}

impl<let N: u32> Eq for str<N> {
    fn eq(self, other: str<N>) -> bool {
        let self_bytes = self.as_bytes();
        let other_bytes = other.as_bytes();
        self_bytes == other_bytes
    }
}

impl<A, B> Eq for (A, B)
where
    A: Eq,
    B: Eq,
{
    fn eq(self, other: (A, B)) -> bool {
        self.0.eq(other.0) & self.1.eq(other.1)
    }
}

impl<A, B, C> Eq for (A, B, C)
where
    A: Eq,
    B: Eq,
    C: Eq,
{
    fn eq(self, other: (A, B, C)) -> bool {
        self.0.eq(other.0) & self.1.eq(other.1) & self.2.eq(other.2)
    }
}

impl<A, B, C, D> Eq for (A, B, C, D)
where
    A: Eq,
    B: Eq,
    C: Eq,
    D: Eq,
{
    fn eq(self, other: (A, B, C, D)) -> bool {
        self.0.eq(other.0) & self.1.eq(other.1) & self.2.eq(other.2) & self.3.eq(other.3)
    }
}

impl<A, B, C, D, E> Eq for (A, B, C, D, E)
where
    A: Eq,
    B: Eq,
    C: Eq,
    D: Eq,
    E: Eq,
{
    fn eq(self, other: (A, B, C, D, E)) -> bool {
        self.0.eq(other.0)
            & self.1.eq(other.1)
            & self.2.eq(other.2)
            & self.3.eq(other.3)
            & self.4.eq(other.4)
    }
}

pub enum Ordering {
    Less,
    Equal,
    Greater
}

impl Eq for Ordering {
    fn eq(self, other: Ordering) -> bool {
        match (self, other) {
            (Self::Less, Self::Less) => true,
            (Self::Greater, Self::Greater) => true,
            (Self::Equal, Self::Equal) => true,
            _ => false,
        }
    }
}

#[derive_via(derive_ord)]
// docs:start:ord-trait
pub trait Ord {
    fn cmp(self, other: Self) -> Ordering;
}
// docs:end:ord-trait

// docs:start:derive_ord
comptime fn derive_ord(s: TypeDefinition) -> Quoted {
    let name = quote { $crate::cmp::Ord };
    let signature = quote { fn cmp(_self: Self, _other: Self) -> $crate::cmp::Ordering };
    let for_each_field = |name| quote {
        if result == $crate::cmp::Ordering::Equal {
            result = _self.$name.cmp(_other.$name);
        }
    };
    let body = |fields| quote {
        let mut result = $crate::cmp::Ordering::Equal;
        $fields
        result
    };
    crate::meta::make_trait_impl(s, name, signature, for_each_field, quote {}, body)
}
// docs:end:derive_ord

// Note: Field deliberately does not implement Ord

impl Ord for u128 {
    fn cmp(self, other: u128) -> Ordering {
        if self < other {
            Ordering::Less
        } else if self > other {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}
impl Ord for u64 {
    fn cmp(self, other: u64) -> Ordering {
        if self < other {
            Ordering::Less
        } else if self > other {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl Ord for u32 {
    fn cmp(self, other: u32) -> Ordering {
        if self < other {
            Ordering::Less
        } else if self > other {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl Ord for u16 {
    fn cmp(self, other: u16) -> Ordering {
        if self < other {
            Ordering::Less
        } else if self > other {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl Ord for u8 {
    fn cmp(self, other: u8) -> Ordering {
        if self < other {
            Ordering::Less
        } else if self > other {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl Ord for i8 {
    fn cmp(self, other: i8) -> Ordering {
        if self < other {
            Ordering::Less
        } else if self > other {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl Ord for i16 {
    fn cmp(self, other: i16) -> Ordering {
        if self < other {
            Ordering::Less
        } else if self > other {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl Ord for i32 {
    fn cmp(self, other: i32) -> Ordering {
        if self < other {
            Ordering::Less
        } else if self > other {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl Ord for i64 {
    fn cmp(self, other: i64) -> Ordering {
        if self < other {
            Ordering::Less
        } else if self > other {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl Ord for () {
    fn cmp(_self: Self, _other: ()) -> Ordering {
        Ordering::Equal
    }
}

impl Ord for bool {
    fn cmp(self, other: bool) -> Ordering {
        if self {
            if other {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        } else if other {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

impl<T, let N: u32> Ord for [T; N]
where
    T: Ord,
{
    // The first non-equal element of both arrays determines
    // the ordering for the whole array.
    fn cmp(self, other: [T; N]) -> Ordering {
        let mut result = Ordering::Equal;
        for i in 0..self.len() {
            if result == Ordering::Equal {
                result = self[i].cmp(other[i]);
            }
        }
        result
    }
}

impl<T> Ord for [T]
where
    T: Ord,
{
    // The first non-equal element of both arrays determines
    // the ordering for the whole array.
    fn cmp(self, other: [T]) -> Ordering {
        let mut result = self.len().cmp(other.len());
        for i in 0..self.len() {
            if result == Ordering::Equal {
                result = self[i].cmp(other[i]);
            }
        }
        result
    }
}

impl<A, B> Ord for (A, B)
where
    A: Ord,
    B: Ord,
{
    fn cmp(self, other: (A, B)) -> Ordering {
        let result = self.0.cmp(other.0);

        if result != Ordering::Equal {
            result
        } else {
            self.1.cmp(other.1)
        }
    }
}

impl<A, B, C> Ord for (A, B, C)
where
    A: Ord,
    B: Ord,
    C: Ord,
{
    fn cmp(self, other: (A, B, C)) -> Ordering {
        let mut result = self.0.cmp(other.0);

        if result == Ordering::Equal {
            result = self.1.cmp(other.1);
        }

        if result == Ordering::Equal {
            result = self.2.cmp(other.2);
        }

        result
    }
}

impl<A, B, C, D> Ord for (A, B, C, D)
where
    A: Ord,
    B: Ord,
    C: Ord,
    D: Ord,
{
    fn cmp(self, other: (A, B, C, D)) -> Ordering {
        let mut result = self.0.cmp(other.0);

        if result == Ordering::Equal {
            result = self.1.cmp(other.1);
        }

        if result == Ordering::Equal {
            result = self.2.cmp(other.2);
        }

        if result == Ordering::Equal {
            result = self.3.cmp(other.3);
        }

        result
    }
}

impl<A, B, C, D, E> Ord for (A, B, C, D, E)
where
    A: Ord,
    B: Ord,
    C: Ord,
    D: Ord,
    E: Ord,
{
    fn cmp(self, other: (A, B, C, D, E)) -> Ordering {
        let mut result = self.0.cmp(other.0);

        if result == Ordering::Equal {
            result = self.1.cmp(other.1);
        }

        if result == Ordering::Equal {
            result = self.2.cmp(other.2);
        }

        if result == Ordering::Equal {
            result = self.3.cmp(other.3);
        }

        if result == Ordering::Equal {
            result = self.4.cmp(other.4);
        }

        result
    }
}

// Compares and returns the maximum of two values.
//
// Returns the second argument if the comparison determines them to be equal.
//
// # Examples
//
// ```
// use std::cmp;
//
// assert_eq(cmp::max(1, 2), 2);
// assert_eq(cmp::max(2, 2), 2);
// ```
pub fn max<T>(v1: T, v2: T) -> T
where
    T: Ord,
{
    if v1 > v2 {
        v1
    } else {
        v2
    }
}

// Compares and returns the minimum of two values.
//
// Returns the first argument if the comparison determines them to be equal.
//
// # Examples
//
// ```
// use std::cmp;
//
// assert_eq(cmp::min(1, 2), 1);
// assert_eq(cmp::min(2, 2), 2);
// ```
pub fn min<T>(v1: T, v2: T) -> T
where
    T: Ord,
{
    if v1 > v2 {
        v2
    } else {
        v1
    }
}

mod cmp_tests {
    use crate::cmp::{max, min, Ordering};

    #[test]
    fn sanity_check_min() {
        assert_eq(min(0 as u64, 1 as u64), 0);
        assert_eq(min(0 as u64, 0 as u64), 0);
        assert_eq(min(1 as u64, 1 as u64), 1);
        assert_eq(min(255 as u8, 0 as u8), 0);
    }

    #[test]
    fn sanity_check_max() {
        assert_eq(max(0 as u64, 1 as u64), 1);
        assert_eq(max(0 as u64, 0 as u64), 0);
        assert_eq(max(1 as u64, 1 as u64), 1);
        assert_eq(max(255 as u8, 0 as u8), 255);
    }

    #[test]
    fn ordering_eq() {
        assert(Ordering::Equal == Ordering::Equal);
        assert(Ordering::Less == Ordering::Less);
        assert(Ordering::Greater == Ordering::Greater);
        assert(Ordering::Equal != Ordering::Greater);
        assert(Ordering::Equal != Ordering::Less);
        assert(Ordering::Less != Ordering::Greater);
        assert(Ordering::Less != Ordering::Equal);
        assert(Ordering::Greater != Ordering::Equal);
        assert(Ordering::Greater != Ordering::Less);
    }
}
