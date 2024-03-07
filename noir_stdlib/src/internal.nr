// This file contains functions which should only be used in calls injected by the Noir compiler.
// These functions should not be called manually in user code.
//
// Changes to this file will not be considered breaking.

#[oracle(assert_message)]
unconstrained fn assert_message_oracle<T>(_input: T) {}
unconstrained pub fn resolve_assert_message<T>(input: T, condition: bool) {
    if !condition {
        assert_message_oracle(input);
    }
}
