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
//! ======================================================================
//! | 0 -> 1     | 29.425ms    | 27.088ms    | 47.936ms    | 150.282ms   |
//! | 1 -> 2     | 33.252ms    | 29.752ms    | 60.845ms    | 301.634ms   |
//! | 2 -> 3     | 35.657ms    | 31.898ms    | 79.367ms    | 487.948ms   |
//! | 3 -> 4     | 1,324.44ms  | 1,079.197ms | 1,846.823ms | 2,225.094ms |
//! | 4 -> 5     | 1,482.358ms | 1,231.839ms | 1,918.353ms | 2,541.392ms |

use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum LateAllocSet<T> {
    None,
    One(T),
    Two(T, T),
    Three(T, T, T),
    Set(BTreeSet<T>),
}

impl<T> LateAllocSet<T>
where
    T: std::cmp::Ord,
{
    pub(super) fn insert(self, x: T) -> Self {
        match self {
            LateAllocSet::None => LateAllocSet::One(x),
            LateAllocSet::One(x0) => {
                if x0 == x {
                    LateAllocSet::One(x0)
                } else {
                    LateAllocSet::Two(x0, x)
                }
            }
            LateAllocSet::Two(x0, x1) => {
                if x0 == x || x1 == x {
                    LateAllocSet::Two(x0, x1)
                } else {
                    LateAllocSet::Three(x0, x1, x)
                }
            }
            LateAllocSet::Three(x0, x1, x2) => {
                if x0 == x || x1 == x || x2 == x {
                    LateAllocSet::Three(x0, x1, x2)
                } else {
                    LateAllocSet::Set(BTreeSet::from([x0, x1, x2, x]))
                }
            }
            LateAllocSet::Set(mut xs) => {
                xs.insert(x);
                LateAllocSet::Set(xs)
            }
        }
    }

    pub(super) fn as_vec(&self) -> Vec<&T> {
        match self {
            LateAllocSet::None => vec![],
            LateAllocSet::One(x0) => vec![x0],
            LateAllocSet::Two(x0, x1) => vec![x0, x1],
            LateAllocSet::Three(x0, x1, x2) => vec![x0, x1, x2],
            LateAllocSet::Set(xs) => xs.iter().collect::<Vec<_>>(),
        }
    }

    pub(super) fn append(self, xs: LateAllocSet<T>) -> Self {
        let mut out = self;
        match xs {
            LateAllocSet::None => {
                // No work
            }
            LateAllocSet::One(x0) => out = out.insert(x0),
            LateAllocSet::Two(x0, x1) => {
                out = out.insert(x0);
                out = out.insert(x1);
            }
            LateAllocSet::Three(x0, x1, x2) => {
                out = out.insert(x0);
                out = out.insert(x1);
                out = out.insert(x2);
            }
            LateAllocSet::Set(xs) => {
                for x in xs {
                    out = out.insert(x);
                }
            }
        }
        out
    }
}

impl<T> FromIterator<T> for LateAllocSet<T>
where
    T: std::cmp::Ord,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let first = iter.next();
        if first.is_none() {
            return LateAllocSet::None;
        }
        let second = iter.next();
        if second.is_none() {
            return LateAllocSet::One(first.unwrap());
        }
        let third = iter.next();
        if third.is_none() {
            return LateAllocSet::Two(first.unwrap(), second.unwrap());
        }
        let fourth = iter.next();
        if fourth.is_none() {
            return LateAllocSet::Three(first.unwrap(), second.unwrap(), third.unwrap());
        }
        let btree_set: BTreeSet<T> =
            [first.unwrap(), second.unwrap(), third.unwrap(), fourth.unwrap()]
                .into_iter()
                .chain(iter)
                .collect();
        LateAllocSet::Set(btree_set)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, time::SystemTime};

    use super::LateAllocSet;
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
            LateAllocSet::None.insert(x0());
        });

        print!("1 -> 2: ");
        time_1m(|| {
            LateAllocSet::One(x0()).insert(x1());
        });
        print!("2 -> 3: ");
        time_1m(|| {
            LateAllocSet::Two(x0(), x1()).insert(x2());
        });
        print!("3 -> 4: ");
        time_1m(|| {
            LateAllocSet::Three(x0(), x1(), x2()).insert(x3());
        });
        print!("4 -> 5: ");
        time_1m(|| {
            LateAllocSet::Set(BTreeSet::from([x0(), x1(), x2(), x3()])).insert(x4());
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
