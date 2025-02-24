use noirc_frontend::{ast::AttributeTarget, Type};

/// When suggest a function as a result of completion, whether to autocomplete its name or its name and parameters.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum FunctionCompletionKind {
    // Only complete a function's name. This is used in use statement.
    Name,
    // Complete a function's name and parameters (as a snippet). This is used in regular code.
    NameAndParameters,
}

/// Is there a requirement for suggesting functions?
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum FunctionKind<'a> {
    /// No requirement: any function is okay to suggest.
    Any,
    /// Only show functions that have the given self type.
    SelfType(&'a Type),
}

/// When requesting completions, whether to list all items or just types.
/// For example, when writing `let x: S` we only want to suggest types at this
/// point (modules too, because they might include types too).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum RequestedItems {
    // Suggest any items (types, functions, etc.).
    AnyItems,
    // Only suggest types (and modules, because they can contain types).
    OnlyTypes,
    // Only suggest traits (and modules, because they can contain traits).
    OnlyTraits,
    // Only attribute functions
    OnlyAttributeFunctions(AttributeTarget),
}
