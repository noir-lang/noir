/// The visibility of an item in a Noir program.
///
/// It is returned for functions by [`FunctionDefinition::visibility`] and for struct fields by
/// [`StructField::visibility`](crate::meta::type_def::StructField::visibility).
///
/// Example:
///
/// ```noir
/// #[check]
/// struct Point {
///     pub x: u32,
///     pub(crate) y: u32,
///     z: u32,
/// }
///
/// comptime fn check(point: TypeDefinition) {
///     let fields = point.fields_as_written();
///     assert(fields[0].visibility().is_public());
///     assert(fields[1].visibility().is_public_crate());
///     assert(fields[2].visibility().is_private());
/// }
/// ```
pub comptime struct ItemVisibility {
    quoted: Quoted,
}

impl ItemVisibility {
    pub(crate) comptime fn from_quoted(quoted: Quoted) -> Self {
        Self { quoted }
    }

    /// Returns `true` if the item is private.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// struct Point {
    ///     pub x: u32,
    ///     pub(crate) y: u32,
    ///     z: u32,
    /// }
    ///
    /// comptime fn check(point: TypeDefinition) {
    ///     let fields = point.fields_as_written();
    ///     assert(fields[2].visibility().is_private());
    ///     assert(!fields[0].visibility().is_private());
    /// }
    /// ```
    // docs:start:is_private
    pub comptime fn is_private(self) -> bool {
        // docs:end:is_private
        self.quoted == quote {}
    }

    /// Returns `true` if the item is public outside its crate.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// struct Point {
    ///     pub x: u32,
    ///     pub(crate) y: u32,
    ///     z: u32,
    /// }
    ///
    /// comptime fn check(point: TypeDefinition) {
    ///     let fields = point.fields_as_written();
    ///     assert(fields[0].visibility().is_public());
    ///     // `pub(crate)` isn't public outside the crate.
    ///     assert(!fields[1].visibility().is_public());
    /// }
    /// ```
    // docs:start:is_public
    pub comptime fn is_public(self) -> bool {
        // docs:end:is_public
        self.quoted == quote { pub }
    }

    /// Returns `true` if the item is public only within its crate.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// struct Point {
    ///     pub x: u32,
    ///     pub(crate) y: u32,
    ///     z: u32,
    /// }
    ///
    /// comptime fn check(point: TypeDefinition) {
    ///     let fields = point.fields_as_written();
    ///     assert(fields[1].visibility().is_public_crate());
    ///     assert(!fields[0].visibility().is_public_crate());
    /// }
    /// ```
    // docs:start:is_public_crate
    pub comptime fn is_public_crate(self) -> bool {
        // docs:end:is_public_crate
        self.quoted == quote { pub(crate) }
    }

    /// Returns this visibility as quoted syntax for use in generated code.
    ///
    /// It is `quote {}` for private items, `quote { pub(crate) }` and `quote { pub }` otherwise.
    ///
    /// Example:
    ///
    /// ```noir
    /// #[check]
    /// struct Point {
    ///     pub x: u32,
    ///     pub(crate) y: u32,
    ///     z: u32,
    /// }
    ///
    /// comptime fn check(point: TypeDefinition) {
    ///     let fields = point.fields_as_written();
    ///     assert_eq(fields[0].visibility().as_quoted(), quote { pub });
    ///     assert_eq(fields[1].visibility().as_quoted(), quote { pub(crate) });
    ///     assert_eq(fields[2].visibility().as_quoted(), quote {});
    /// }
    /// ```
    // docs:start:as_quoted
    pub comptime fn as_quoted(self) -> Quoted {
        // docs:end:as_quoted
        self.quoted
    }
}
