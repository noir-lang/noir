/// The visibility of an item in a Noir program.
pub comptime struct ItemVisibility {
    quoted: Quoted,
}

impl ItemVisibility {
    pub(crate) comptime fn from_quoted(quoted: Quoted) -> Self {
        Self { quoted }
    }

    /// Returns `true` if the item is private.
    // docs:start:is_private
    pub comptime fn is_private(self) -> bool {
        // docs:end:is_private
        self.quoted == quote {}
    }

    /// Returns `true` if the item is public outside its crate.
    // docs:start:is_public
    pub comptime fn is_public(self) -> bool {
        // docs:end:is_public
        self.quoted == quote { pub }
    }

    /// Returns `true` if the item is public only within its crate.
    // docs:start:is_public_crate
    pub comptime fn is_public_crate(self) -> bool {
        // docs:end:is_public_crate
        self.quoted == quote { pub(crate) }
    }

    /// Returns this visibility as quoted syntax for use in generated code.
    // docs:start:as_quoted
    pub comptime fn as_quoted(self) -> Quoted {
        // docs:end:as_quoted
        self.quoted
    }
}
