use noirc_frontend::Type;

/// When finding items in a module, whether to show only direct children or all visible items.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ModuleCompletionKind {
    // Only show a module's direct children. This is used when completing a use statement
    // or a path after the first segment.
    DirectChildren,
    // Show all of a module's visible items. This is used when completing a path outside
    // of a use statement (in regular code) when the path is just a single segment:
    // we want to find items exposed in the current module.
    AllVisibleItems,
}

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
    // Only suggest types.
    OnlyTypes,
}
