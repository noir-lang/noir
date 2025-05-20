/// Utility function to console.log data in the acir simulator.
/// Example:
///   debug_log("blah blah this is a debug string");
pub fn debug_log<let N: u32>(msg: str<N>) {
    debug_log_format(msg, []);
}

/// Utility function to console.log data in the acir simulator. This variant receives a format string in which the
/// `${k}` tokens will be replaced with the k-eth value in the `args` array.
/// Examples:
///   debug_log_format("get_2(slot:{0}) =>\n\t0:{1}\n\t1:{2}", [storage_slot, note0_hash, note1_hash]);
///   debug_log_format("whole array: {}", [e1, e2, e3, e4]);
pub fn debug_log_format<let M: u32, let N: u32>(msg: str<M>, args: [Field; N]) {
    // Safety: This oracle call returns nothing: we only call it for its side effects. It is therefore always safe
    // to call.
    unsafe { debug_log_oracle_wrapper(msg, args) };
}

pub unconstrained fn debug_log_oracle_wrapper<let M: u32, let N: u32>(
    msg: str<M>,
    args: [Field; N],
) {
    debug_log_oracle(msg, args.as_slice());
}

// WARNING: sometimes when using debug logs the ACVM errors with: `thrown: "solver opcode resolution error: cannot solve opcode: expression has too many unknowns x155"`
#[oracle(debugLog)]
unconstrained fn debug_log_oracle<let M: u32>(_msg: str<M>, args: [Field]) {}
