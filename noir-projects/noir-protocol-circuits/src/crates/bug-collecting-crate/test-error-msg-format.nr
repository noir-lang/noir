Should be able to use special characters (,.-) in the string in a macro.

error: Malformed function attribute
    ┌─ /mnt/user-data/leila/aztec-packages/yarn-project/noir-protocol-circuits/src/crates/private-kernel-lib/src/private_kernel_init.nr:436:5
    │
436 │     #[test(should_fail_with="tx_request.function_data must match call_stack_item.function_data")]
    │     ---------------------------------------------------------------------------------------------  test(should_fail_with="tx_request.function_data must match call_stack_item.function_data") is not a valid attribute
