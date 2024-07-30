mod trait_constraint;
mod trait_def;
mod type_def;
mod quoted;

/// Calling unquote as a macro (via `unquote!(arg)`) will unquote
/// its argument. Since this is the effect `!` already does, `unquote`
/// itself does not need to do anything besides return its argument.
pub comptime fn unquote(code: Quoted) -> Quoted {
    code
}
