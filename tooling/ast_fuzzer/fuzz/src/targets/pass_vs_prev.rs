//! Compare the execution of an SSA pass to the one preceding it
//! using the SSA interpreter.
//!
//! By using the SSA interpreter we can execute any pass in the pipeline,
//! as opposed to the Brillig runtime, which requires a minimum number
//! of passes to be carried out to work.
