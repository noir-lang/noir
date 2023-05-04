//! `LateAllocSet` is an alternative to `BTreeSet` optimized for small sets that can be located
//! entirely in stack memory. Once the set size goes beyond 3, performance is less than that of a
//! `BTreeMap`.
//!
//! Approximately 20-50 times faster than `BTreeSet` it is beyond three elements - at which point
//! it switches to using a `BTreeSet` internally. This container makes sense for short lived sets
//! that very rarely go beyond 3 elements, and for which the elements can be represented entirely
//! in stack  memory.
//!
//! This set's size is at least 3 times the size of it's element's, so it is not suitable to be
//! held as an item type in larger parent collections.
//!
//! Below - time taken to insert Nth element one millions times for differing types, sampled by
//! running `inserts_different_types` in tests below on a 2.3 GHz MacBook Pro (2019).
//!
//! | Nth insert | &str        | u32         | Token       | String      |
//! |------------|-------------|-------------|-------------|-------------|
//! | 0 -> 1     | 29.425ms    | 27.088ms    | 47.936ms    | 150.282ms   |
//! | 1 -> 2     | 33.252ms    | 29.752ms    | 60.845ms    | 301.634ms   |
//! | 2 -> 3     | 35.657ms    | 31.898ms    | 79.367ms    | 487.948ms   |
//! | 3 -> 4     | 1,324.44ms  | 1,079.197ms | 1,846.823ms | 2,225.094ms |
//! | 4 -> 5     | 1,482.358ms | 1,231.839ms | 1,918.353ms | 2,541.392ms |

use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
enum LateAllocSetData<T> {
    None,
    One(T),
    Two(T, T),
    Three(T, T, T),
    Set(BTreeSet<T>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LateAllocSet<T> {
    data: LateAllocSetData<T>,
}

impl<T> LateAllocSet<T>
where
    T: std::cmp::Ord,
{
    pub(super) fn new() -> Self {
        LateAllocSet { data: LateAllocSetData::None }
    }

    pub(super) fn insert(&mut self, x: T) {
        let old_data = std::mem::replace(&mut self.data, LateAllocSetData::None);
        self.data = match old_data {
            LateAllocSetData::None => LateAllocSetData::One(x),
            LateAllocSetData::One(x0) => {
                if x0 == x {
                    LateAllocSetData::One(x0)
                } else {
                    LateAllocSetData::Two(x0, x)
                }
            }
            LateAllocSetData::Two(x0, x1) => {
                if x0 == x || x1 == x {
                    LateAllocSetData::Two(x0, x1)
                } else {
                    LateAllocSetData::Three(x0, x1, x)
                }
            }
            LateAllocSetData::Three(x0, x1, x2) => {
                if x0 == x || x1 == x || x2 == x {
                    LateAllocSetData::Three(x0, x1, x2)
                } else {
                    LateAllocSetData::Set(BTreeSet::from([x0, x1, x2, x]))
                }
            }
            LateAllocSetData::Set(mut xs) => {
                xs.insert(x);
                LateAllocSetData::Set(xs)
            }
        };
    }

    pub(super) fn as_vec(&self) -> Vec<&T> {
        match &self.data {
            LateAllocSetData::None => vec![],
            LateAllocSetData::One(x0) => vec![x0],
            LateAllocSetData::Two(x0, x1) => vec![x0, x1],
            LateAllocSetData::Three(x0, x1, x2) => vec![x0, x1, x2],
            LateAllocSetData::Set(xs) => xs.iter().collect::<Vec<_>>(),
        }
    }

    pub(super) fn append(&mut self, other: LateAllocSet<T>) {
        match other.data {
            LateAllocSetData::None => {
                // No work
            }
            LateAllocSetData::One(x0) => self.insert(x0),
            LateAllocSetData::Two(x0, x1) => {
                self.insert(x0);
                self.insert(x1);
            }
            LateAllocSetData::Three(x0, x1, x2) => {
                self.insert(x0);
                self.insert(x1);
                self.insert(x2);
            }
            LateAllocSetData::Set(xs) => {
                for x in xs {
                    self.insert(x);
                }
            }
        }
    }

    pub(super) fn clear(&mut self) {
        self.data = LateAllocSetData::None;
    }
}

impl<T> FromIterator<T> for LateAllocSetData<T>
where
    T: std::cmp::Ord,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let first = iter.next();
        if first.is_none() {
            return LateAllocSetData::None;
        }
        let second = iter.next();
        if second.is_none() {
            return LateAllocSetData::One(first.unwrap());
        }
        let third = iter.next();
        if third.is_none() {
            return LateAllocSetData::Two(first.unwrap(), second.unwrap());
        }
        let fourth = iter.next();
        if fourth.is_none() {
            return LateAllocSetData::Three(first.unwrap(), second.unwrap(), third.unwrap());
        }
        let btree_set: BTreeSet<T> =
            [first.unwrap(), second.unwrap(), third.unwrap(), fourth.unwrap()]
                .into_iter()
                .chain(iter)
                .collect();
        LateAllocSetData::Set(btree_set)
    }
}

impl<T> FromIterator<T> for LateAllocSet<T>
where
    T: std::cmp::Ord,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let data: LateAllocSetData<T> = iter.into_iter().collect();
        LateAllocSet { data }
    }
}
#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, time::SystemTime};

    use super::{LateAllocSet, LateAllocSetData};
    use crate::token::Token;

    fn time_1m<F>(f: F)
    where
        F: Fn(),
    {
        let start = SystemTime::now();
        for _ in 0..1000000 {
            f();
        }
        println!("{:?}", start.elapsed().unwrap());
    }

    fn time_1m_inserts_1_to_5<T, F0, F1, F2, F3, F4>(x0: F0, x1: F1, x2: F2, x3: F3, x4: F4)
    where
        T: std::cmp::Ord + Clone,
        F0: Fn() -> T,
        F1: Fn() -> T,
        F2: Fn() -> T,
        F3: Fn() -> T,
        F4: Fn() -> T,
    {
        print!("0 -> 1: ");
        time_1m(|| {
            LateAllocSet { data: LateAllocSetData::None }.insert(x0());
        });

        print!("1 -> 2: ");
        time_1m(|| {
            LateAllocSet { data: LateAllocSetData::One(x0()) }.insert(x1());
        });
        print!("2 -> 3: ");
        time_1m(|| {
            LateAllocSet { data: LateAllocSetData::Two(x0(), x1()) }.insert(x2());
        });
        print!("3 -> 4: ");
        time_1m(|| {
            LateAllocSet { data: LateAllocSetData::Three(x0(), x1(), x2()) }.insert(x3());
        });
        print!("4 -> 5: ");
        time_1m(|| {
            LateAllocSet { data: LateAllocSetData::Set(BTreeSet::from([x0(), x1(), x2(), x3()])) }
                .insert(x4());
        });
    }

    #[test]
    #[ignore]
    fn inserts_different_types() {
        println!("\nelement type: &str");
        time_1m_inserts_1_to_5(|| "a", || "b", || "c", || "d", || "e");

        println!("\nelement type: u32");
        time_1m_inserts_1_to_5(|| 0, || 1, || 2, || 3, || 4);

        println!("\nelement type: Token");
        time_1m_inserts_1_to_5(
            || Token::Ampersand,
            || Token::Arrow,
            || Token::Assign,
            || Token::Bang,
            || Token::Caret,
        );

        println!("\nelement type: String");
        time_1m_inserts_1_to_5(
            || String::from("a"),
            || String::from("b"),
            || String::from("c"),
            || String::from("d"),
            || String::from("e"),
        );
    }
}
