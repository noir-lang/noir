use crate::cmp::Eq;
use crate::hash::{Hash, Hasher};

impl TraitDefinition {
    /// Returns this trait as a trait constraint, such as `Eq` or `Into<T>`.
    ///
    /// Generics on the trait are kept as they are, without being instantiated.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub trait Shape {
    ///     fn area(self) -> u32;
    /// }
    ///
    /// pub struct Square {
    ///     side: u32,
    /// }
    ///
    /// impl Shape for Square {
    ///     fn area(self) -> u32 {
    ///         self.side * self.side
    ///     }
    /// }
    ///
    /// comptime fn check(shape: TraitDefinition) {
    ///     let constraint = shape.as_trait_constraint();
    ///     assert(quote { Square }.as_type().implements(constraint));
    ///     assert(!quote { u32 }.as_type().implements(constraint));
    /// }
    /// ```
    #[builtin(trait_def_as_trait_constraint)]
    // docs:start:as_trait_constraint
    pub comptime fn as_trait_constraint(_self: Self) -> TraitConstraint {}
    // docs:end:as_trait_constraint

    /// Returns the source location of this trait's name in its definition.
    ///
    /// Pass it to [`std::meta::error`](crate::meta::error) or [`std::meta::warn`](crate::meta::warn)
    /// to report a diagnostic at the trait.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// pub trait Shape {
    ///     fn area(self) -> u32;
    /// }
    ///
    /// comptime fn check(shape: TraitDefinition) {
    ///     // Warns "this trait has no implementations yet", pointing at `Shape`.
    ///     std::meta::warn(
    ///         f"this trait has no implementations yet",
    ///         Option::<fmtstr<0, ()>>::none(),
    ///         shape.location(),
    ///     );
    /// }
    /// ```
    #[builtin(trait_def_location)]
    // docs:start:location
    pub comptime fn location(self) -> Location {}
    // docs:end:location
}

impl Eq for TraitDefinition {
    #[builtin(trait_def_eq)]
    comptime fn eq(self, other: Self) -> bool {}
}

impl Hash for TraitDefinition {
    comptime fn hash<H>(self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(trait_def_hash(self));
    }
}

#[builtin(trait_def_hash)]
comptime fn trait_def_hash(_def: TraitDefinition) -> Field {}
