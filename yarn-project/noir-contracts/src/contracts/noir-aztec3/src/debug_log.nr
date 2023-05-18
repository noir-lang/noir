
// Utility function to console.log data in the acir simulator

#[oracle(debugLog)]
fn debug_log_oracle(_msg: Field) -> Field {}

unconstrained fn debug_log(msg: Field) -> Field {
    debug_log_oracle(msg)
}