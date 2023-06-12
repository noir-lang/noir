// Utility function to console.log data in the acir simulator
// WARNING: sometimes when using debug logs the ACVM errors with: `thrown: "solver opcode resolution error: cannot solve opcode: expression has too many unknowns x155"`

#[oracle(debugLog)]
fn debug_log_oracle<T, N>(_msg: T, _num_args: Field) -> Field {}
#[oracle(debugLog)]
fn debug_log_format_oracle<T, N>(_msg: T, _args: [Field; N], _num_args: Field) -> Field {}
#[oracle(debugLog)]
fn debug_log_field_oracle(_field: Field) -> Field {}

/// NOTE: call this with a str<N> msg of length > 1
/// Example:
/// `debug_log("blah blah this is a debug string");`
unconstrained fn debug_log<T>(msg: T) {
    constrain debug_log_oracle(msg, 0) == 0;
}

/// NOTE: call this with a str<N> msg of form
/// "some string with {0} and {1} ... {N}"
/// and an array of N field which will be formatted
/// into the string in the simulator.
/// Example:
/// debug_log_format("get_2(slot:{0}) =>\n\t0:{1}\n\t1:{2}", [storage_slot, note0_hash, note1_hash]);
unconstrained fn debug_log_format<T, N>(msg: T, args: [Field; N]) {
    constrain debug_log_format_oracle(msg, args, args.len()) == 0;
}

/// Example:
/// `debug_log_field(my_field);`
unconstrained fn debug_log_field(field: Field) {
    constrain debug_log_field_oracle(field) == 0;
}
