use std::collections::BTreeSet;

use crate::ssa::ir::value::ValueId;

/// A set of possible aliases. Each ValueId in this set represents one possible value the reference
/// holding this AliasSet may be aliased to. This struct wrapper is provided so that when we take
/// the union of multiple alias sets, the result should be empty if any individual set is empty.
///
/// Note that we distinguish between "definitely has no aliases" - `Some(BTreeSet::new())`, and
/// "unknown which aliases this may refer to" - `None`.
#[derive(Debug, Default, Clone)]
pub(super) enum AliasSet {
    #[default]
    Unknown,
    Empty,
    Known(ValueId),
    KnownMultiple(BTreeSet<ValueId>),
}

impl AliasSet {
    pub(super) fn unknown() -> AliasSet {
        AliasSet::Unknown
    }

    pub(super) fn known(value: ValueId) -> AliasSet {
        AliasSet::Known(value)
    }

    pub(super) fn known_multiple(values: BTreeSet<ValueId>) -> AliasSet {
        if values.is_empty() {
            AliasSet::Empty
        } else if values.len() == 1 {
            AliasSet::Known(*values.first().unwrap())
        } else {
            AliasSet::KnownMultiple(values)
        }
    }

    /// In rare cases, such as when creating an empty array of references, the set of aliases for a
    /// particular value will be known to be zero, which is distinct from being unknown and
    /// possibly referring to any alias.
    pub(super) fn known_empty() -> AliasSet {
        AliasSet::Empty
    }

    pub(super) fn is_unknown(&self) -> bool {
        matches!(self, AliasSet::Unknown)
    }

    /// Return the single known alias if there is exactly one.
    /// Otherwise, return None.
    pub(super) fn single_alias(&self) -> Option<ValueId> {
        if let AliasSet::Known(alias) = self {
            Some(*alias)
        } else {
            None
        }
    }

    /// Unify this alias set with another. The result of this set is unknown if either set is unknown.
    /// Otherwise, it is the union of both alias sets.
    pub(super) fn unify(&mut self, other: &Self) {
        match self {
            AliasSet::Unknown => (),
            AliasSet::Empty => *self = other.clone(),
            AliasSet::Known(id) => match other {
                AliasSet::Unknown => *self = AliasSet::Unknown,
                AliasSet::Empty => (),
                AliasSet::Known(other_id) => {
                    if id != other_id {
                        *self = AliasSet::KnownMultiple([*id, *other_id].iter().copied().collect());
                    }
                }
                AliasSet::KnownMultiple(other_values) => {
                    let mut values = other_values.clone();
                    values.insert(*id);
                    *self = AliasSet::KnownMultiple(values);
                }
            },
            AliasSet::KnownMultiple(values) => match other {
                AliasSet::Unknown => *self = AliasSet::Unknown,
                AliasSet::Empty => (),
                AliasSet::Known(other_id) => {
                    values.insert(*other_id);
                }
                AliasSet::KnownMultiple(other_values) => {
                    values.extend(other_values);
                }
            },
        }
    }

    /// Inserts a new alias into this set if it is not unknown
    pub(super) fn insert(&mut self, new_alias: ValueId) {
        match self {
            AliasSet::Unknown => (),
            AliasSet::Empty => *self = AliasSet::Known(new_alias),
            AliasSet::Known(id) => {
                if *id != new_alias {
                    *self = AliasSet::KnownMultiple([*id, new_alias].iter().copied().collect());
                }
            }
            AliasSet::KnownMultiple(values) => {
                values.insert(new_alias);
            }
        }
    }

    /// Returns `Some(true)` if `f` returns true for any known alias in this set.
    /// If this alias set is unknown, None is returned.
    pub(super) fn any(&self, mut f: impl FnMut(ValueId) -> bool) -> Option<bool> {
        match self {
            AliasSet::Unknown => None,
            AliasSet::Empty => Some(false),
            AliasSet::Known(id) => Some(f(*id)),
            AliasSet::KnownMultiple(values) => Some(values.iter().copied().any(f)),
        }
    }

    pub(super) fn for_each(&self, mut f: impl FnMut(ValueId)) {
        match self {
            AliasSet::Unknown | AliasSet::Empty => (),
            AliasSet::Known(id) => f(*id),
            AliasSet::KnownMultiple(values) => {
                for value in values {
                    f(*value);
                }
            }
        }
    }

    /// Return the first ValueId in the alias set as long as there is at least one.
    /// The ordering is arbitrary (by lowest ValueId) so this method should only be
    /// used when you need an arbitrary ValueId from the alias set.
    pub(super) fn first(&self) -> Option<ValueId> {
        match self {
            AliasSet::Unknown | AliasSet::Empty => None,
            AliasSet::Known(id) => Some(*id),
            AliasSet::KnownMultiple(values) => values.first().copied(),
        }
    }
}
