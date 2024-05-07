// Utility function to console.log data in the acir simulator
// WARNING: sometimes when using debug logs the ACVM errors with: `thrown: "solver opcode resolution error: cannot solve opcode: expression has too many unknowns x155"`

#[oracle(debugLog)]
fn debug_log_oracle<T, N>(_msg: T, _num_args: Field) {}
#[oracle(debugLog)]
fn debug_log_format_oracle<T, N>(_msg: T, _args: [Field; N], _num_args: Field) {}
#[oracle(debugLog)]
fn debug_log_field_oracle(_field: Field) {}
#[oracle(debugLog)]
fn debug_log_array_oracle<T, N>(_arbitrary_array: [T; N]) {}
#[oracle(debugLogWithPrefix)]
fn debug_log_array_with_prefix_oracle<S, T, N>(_prefix: S, _arbitrary_array: [T; N]) {}

/// NOTE: call this with a str<N> msg of length > 1
/// Example:
/// `debug_log("blah blah this is a debug string");`
unconstrained pub fn debug_log<T>(msg: T) {
    debug_log_oracle(msg, 0);
}

/// NOTE: call this with a str<N> msg of form
/// "some string with {0} and {1} ... {N}"
/// and an array of N field which will be formatted
/// into the string in the simulator.
/// Example:
/// debug_log_format("get_2(slot:{0}) =>\n\t0:{1}\n\t1:{2}", [storage_slot, note0_hash, note1_hash]);
unconstrained pub fn debug_log_format<T, N>(msg: T, args: [Field; N]) {
    debug_log_format_oracle(msg, args, args.len() as Field);
}

/// Example:
/// `debug_log_field(my_field);`
unconstrained pub fn debug_log_field(field: Field) {
    debug_log_field_oracle(field);
}

/// Example:
/// `debug_log_array(my_array);`
unconstrained fn debug_log_array<T, N>(arbitrary_array: [T; N]) {
    debug_log_array_oracle(arbitrary_array);
}

/// Example:
/// `debug_log_array_with_prefix("Prefix", my_array);`
unconstrained pub fn debug_log_array_with_prefix<S, T, N>(prefix: S, arbitrary_array: [T; N]) {
    debug_log_array_with_prefix_oracle(prefix, arbitrary_array);
}
