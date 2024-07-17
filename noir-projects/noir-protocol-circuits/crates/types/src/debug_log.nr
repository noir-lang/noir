// Utility function to console.log data in the acir simulator
// WARNING: sometimes when using debug logs the ACVM errors with: `thrown: "solver opcode resolution error: cannot solve opcode: expression has too many unknowns x155"`

#[oracle(debugLog)]
unconstrained fn debug_log_oracle<let M: u32>(_msg: str<M>, args: [Field]) {}

/// NOTE: call this with a str<N> msg of form
/// "some string with {0} and {1} ... {N}"
/// and an array of N field which will be formatted
/// into the string in the simulator.
/// Example:
/// debug_log_format("get_2(slot:{0}) =>\n\t0:{1}\n\t1:{2}", [storage_slot, note0_hash, note1_hash]);
/// debug_log_format("whole array: {}", [e1, e2, e3, e4]);
unconstrained pub fn debug_log_format<let M: u32, let N: u32>(msg: str<M>, args: [Field; N]) {
    debug_log_oracle(msg, args.as_slice());
}

/// NOTE: call this with a str<N> msg of length > 1
/// Example:
/// `debug_log("blah blah this is a debug string");`
unconstrained pub fn debug_log<let N: u32>(msg: str<N>) {
    debug_log_format(msg, []);
}
