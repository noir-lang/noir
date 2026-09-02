use acvm::acir::BlackBoxFunc;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumIter, EnumString};

/// A function the compiler implements, declared in the standard library with the
/// `#[builtin(name)]` or `#[foreign(name)]` attribute.
///
/// The two attribute kinds share a single flat namespace: by the time a call is
/// dispatched, builtin and foreign ("low level") functions are treated identically,
/// so this enum covers both. Oracle (`#[oracle]`) names are deliberately *not* here:
/// they are an open, user-extensible namespace and stay strings.
///
/// Attribute parsing keeps the name as a free-form string (so malformed or unknown
/// names still parse and error later, with a real diagnostic); the string is resolved
/// to this enum where the function is actually consumed: the comptime interpreter,
/// the monomorphizer, and (via `Definition::Builtin`/`Definition::LowLevel`) SSA
/// generation.
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    AsRefStr,
    EnumString,
    EnumIter
)]
#[strum(serialize_all = "snake_case")]
pub enum Builtin {
    // Runtime intrinsics: reach SSA generation and map to `Intrinsic` in noirc_evaluator.
    ApplyRangeConstraint,
    ArrayAsStrUnchecked,
    ArrayLen,
    ArrayRefcount,
    AsVector,
    AsWitness,
    AssertConstant,
    #[strum(serialize = "black_box")]
    BlackBoxHint,
    DerivePedersenGenerators,
    FieldLessThan,
    IsUnconstrained,
    StaticAssert,
    StrAsBytes,
    ToBeBits,
    ToBeRadix,
    ToLeBits,
    ToLeRadix,
    VectorInsert,
    VectorPopBack,
    VectorPopFront,
    VectorPushBack,
    VectorPushFront,
    VectorRefcount,
    VectorRemove,

    // Evaluated during monomorphization (see `HandledOpcode`); never reach SSA.
    CheckedTransmute,
    ModulusBeBits,
    ModulusBeBytes,
    ModulusLeBits,
    ModulusLeBytes,
    ModulusNumBits,
    Zeroed,

    // Comptime-only builtins, dispatched by the comptime interpreter.
    CtstringAppend,
    CtstringEq,
    CtstringHash,
    ExprAsArray,
    ExprAsAssert,
    ExprAsAssertEq,
    ExprAsAssign,
    ExprAsBinaryOp,
    ExprAsBlock,
    ExprAsBool,
    ExprAsCast,
    ExprAsComptime,
    ExprAsConstructor,
    ExprAsFor,
    ExprAsForRange,
    ExprAsFunctionCall,
    ExprAsIf,
    ExprAsIndex,
    ExprAsInteger,
    ExprAsLambda,
    ExprAsLet,
    ExprAsMemberAccess,
    ExprAsMethodCall,
    ExprAsRepeatedElementArray,
    ExprAsRepeatedElementVector,
    ExprAsTuple,
    ExprAsUnaryOp,
    ExprAsUnsafe,
    ExprAsVector,
    ExprHasSemicolon,
    ExprIsBreak,
    ExprIsContinue,
    ExprResolve,
    FmtstrAsCtstring,
    FmtstrQuotedContents,
    FreshTypeVariable,
    FunctionDefAsTypedExpr,
    FunctionDefBody,
    FunctionDefDisable,
    FunctionDefEq,
    FunctionDefHasBuiltinAttribute,
    FunctionDefHasNamedAttribute,
    FunctionDefHash,
    FunctionDefIsUnconstrained,
    FunctionDefLocation,
    FunctionDefModule,
    FunctionDefName,
    FunctionDefNamedAttributeArgs,
    FunctionDefParameters,
    FunctionDefReturnType,
    FunctionDefVisibility,
    IssueError,
    IssueWarning,
    LocationEq,
    LocationHash,
    ModuleChildModules,
    ModuleEq,
    ModuleFunctions,
    ModuleHasBuiltinAttribute,
    ModuleHasNamedAttribute,
    ModuleHash,
    ModuleIsContract,
    ModuleLocation,
    ModuleName,
    ModuleNamedAttributeArgs,
    ModuleParent,
    ModuleStructs,
    QuotedAsExpr,
    QuotedAsModule,
    QuotedAsTraitConstraint,
    QuotedAsType,
    QuotedEq,
    QuotedHash,
    QuotedLocation,
    QuotedTokens,
    StrAsCtstring,
    TraitConstraintEq,
    TraitConstraintHash,
    TraitDefAsTraitConstraint,
    TraitDefEq,
    TraitDefHash,
    TraitDefLocation,
    TraitImplMethods,
    TraitImplTraitGenericArgs,
    TypeAsArray,
    TypeAsConstant,
    TypeAsDataType,
    TypeAsInteger,
    TypeAsMutableReference,
    TypeAsStr,
    TypeAsTuple,
    TypeAsVector,
    TypeDefAddAbi,
    TypeDefAsType,
    TypeDefAsTypeWithGenerics,
    TypeDefEq,
    TypeDefFields,
    TypeDefFieldsAsWritten,
    TypeDefGenerics,
    TypeDefHasBuiltinAttribute,
    TypeDefHasNamedAttribute,
    TypeDefHash,
    TypeDefLocation,
    TypeDefModule,
    TypeDefName,
    TypeDefNamedAttributeArgs,
    TypeEq,
    TypeGetTraitImpl,
    TypeHash,
    TypeImplements,
    TypeIsBool,
    TypeIsField,
    TypeIsUnit,
    TypeOf,
    TypedExprAsFunctionDefinition,
    TypedExprGetType,
    TypedExprLocation,

    // Foreign (`#[foreign]`) functions that are not ACIR black box functions.
    Poseidon2ConfigStateSize,

    /// A foreign (`#[foreign]`) function implemented as an ACIR black box function
    /// (hashes, curve operations, signature verification). Excluded from the strum
    /// derives because of its payload; [`Self::lookup`] and [`Self::name`] handle it
    /// explicitly.
    #[strum(disabled)]
    BlackBox(BlackBoxFunc),
}

impl Builtin {
    /// Resolve a `#[builtin(name)]` / `#[foreign(name)]` attribute name.
    /// Returns `None` for names the compiler does not implement.
    pub fn lookup(name: &str) -> Option<Self> {
        name.parse().ok().or_else(|| {
            let func = BlackBoxFunc::lookup(name)?;
            // AND, XOR and RANGE are not callable functions: the compiler emits their
            // opcodes from binary operations, casts and range checks.
            let callable =
                !matches!(func, BlackBoxFunc::AND | BlackBoxFunc::XOR | BlackBoxFunc::RANGE);
            callable.then_some(Self::BlackBox(func))
        })
    }

    /// The name this builtin is declared with, i.e. the inverse of [`Self::lookup`].
    pub fn name(&self) -> &str {
        match self {
            Self::BlackBox(func) => func.name(),
            other => other.as_ref(),
        }
    }
}

impl std::fmt::Display for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::Builtin;
    use acvm::acir::BlackBoxFunc;

    #[test]
    fn name_round_trips_through_lookup() {
        // `Builtin::iter` skips the strum-disabled `BlackBox` variant, so chain the
        // callable black box functions explicitly.
        let black_boxes = BlackBoxFunc::iter()
            .filter(|func| Builtin::lookup(func.name()).is_some())
            .map(Builtin::BlackBox);
        for builtin in Builtin::iter().chain(black_boxes) {
            assert_eq!(
                Builtin::lookup(builtin.name()),
                Some(builtin),
                "`{builtin:?}` does not round-trip through its name `{}`",
                builtin.name()
            );
        }
    }

    #[test]
    fn black_box_funcs_resolve_to_builtins() {
        for func in BlackBoxFunc::iter() {
            // AND, XOR and RANGE are not callable functions: the compiler emits their
            // opcodes from binary operations, casts and range checks, so their names
            // do not resolve.
            let expected =
                if matches!(func, BlackBoxFunc::AND | BlackBoxFunc::XOR | BlackBoxFunc::RANGE) {
                    None
                } else {
                    Some(Builtin::BlackBox(func))
                };
            assert_eq!(Builtin::lookup(func.name()), expected);
        }
    }
}
