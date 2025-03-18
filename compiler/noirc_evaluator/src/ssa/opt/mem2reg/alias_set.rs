use vec_collections::{AbstractVecSet, VecSet};

use crate::ssa::ir::value::ValueId;

/// A set of possible aliases. Each ValueId in this set represents one possible value the reference
/// holding this AliasSet may be aliased to. This struct wrapper is provided so that when we take
/// the union of multiple alias sets, the result should be empty if any individual set is empty.
///
/// Note that we distinguish between "definitely has no aliases" - `Some(BTreeSet::new())`, and
/// "unknown which aliases this may refer to" - `None`.
#[derive(Debug, Default, Clone)]
pub(super) struct AliasSet {
    aliases: Option<VecSet<[ValueId; 1]>>,
}

impl AliasSet {
    pub(super) fn unknown() -> AliasSet {
        Self { aliases: None }
    }

    pub(super) fn known(value: ValueId) -> AliasSet {
        Self { aliases: Some(VecSet::single(value)) }
    }

    pub(super) fn known_multiple(values: VecSet<[ValueId; 1]>) -> AliasSet {
        Self { aliases: Some(values) }
    }

    /// In rare cases, such as when creating an empty array of references, the set of aliases for a
    /// particular value will be known to be zero, which is distinct from being unknown and
    /// possibly referring to any alias.
    pub(super) fn known_empty() -> AliasSet {
        Self { aliases: Some(VecSet::empty()) }
    }

    pub(super) fn is_unknown(&self) -> bool {
        self.aliases.is_none()
    }

    /// Return the single known alias if there is exactly one.
    /// Otherwise, return None.
    pub(super) fn single_alias(&self) -> Option<ValueId> {
        self.aliases
            .as_ref()
            .and_then(|aliases| (aliases.len() == 1).then(|| *aliases.iter().next().unwrap()))
    }

    /// Unify this alias set with another. The result of this set is empty if either set is empty.
    /// Otherwise, it is the union of both alias sets.
    pub(super) fn unify(&mut self, other: &Self) {
        if let (Some(self_aliases), Some(other_aliases)) = (&mut self.aliases, &other.aliases) {
            self_aliases.extend(other_aliases.iter().cloned());
        } else {
            self.aliases = None;
        }
    }

    /// Returns true if calling `unify` would change something in this alias set.
    ///
    /// This is an optimization to avoid having to look up an entry ready to be modified in the [Block](crate::ssa::opt::mem2reg::block::Block),
    /// because doing so would involve calling `Arc::make_mut` which clones the entry, ready for modification.
    pub(super) fn should_unify(&self, other: &Self) -> bool {
        if let (Some(self_aliases), Some(other_aliases)) = (&self.aliases, &other.aliases) {
            // `unify` would extend `self_aliases` with `other_aliases`, so if `other_aliases` is a subset, then nothing would happen.
            !other_aliases.is_subset(self_aliases)
        } else {
            // `unify` would set `aliases` to `None`, so if it's not `Some`, then nothing would happen.
            self.aliases.is_some()
        }
    }

    /// Inserts a new alias into this set if it is not unknown
    pub(super) fn insert(&mut self, new_alias: ValueId) {
        if let Some(aliases) = &mut self.aliases {
            aliases.insert(new_alias);
        }
    }

    /// Returns `Some(true)` if `f` returns true for any known alias in this set.
    /// If this alias set is unknown, None is returned.
    pub(super) fn any(&self, f: impl FnMut(ValueId) -> bool) -> Option<bool> {
        self.aliases.as_ref().map(|aliases| aliases.iter().copied().any(f))
    }

    pub(super) fn for_each(&self, mut f: impl FnMut(ValueId)) {
        if let Some(aliases) = &self.aliases {
            for alias in aliases {
                f(*alias);
            }
        }
    }

    /// Return the first ValueId in the alias set as long as there is at least one.
    /// The ordering is arbitrary (by lowest ValueId) so this method should only be
    /// used when you need an arbitrary ValueId from the alias set.
    pub(super) fn first(&self) -> Option<ValueId> {
        self.aliases.as_ref().and_then(|aliases| aliases.iter().next().copied())
    }
}
