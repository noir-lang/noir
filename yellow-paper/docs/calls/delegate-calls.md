---
sidebar_position: 6
---

# Delegate calls

Delegate calls are function calls against a contract class identifier instead of an instance. Any call, synchronous or asynchronous, can be made as a delegate call. The behavior of a delegate call is to execute the function code in the specified class identifier but on the context of the current instance. This opens the door to script-like executions and upgradeable contracts. Delegate calls are based on [EIP7](https://eips.ethereum.org/EIPS/eip-7).

At the protocol level, a delegate call is identified by a `is_delegate_call` flag in the `CircuitPublicInputs` of the `CallStackItem`. The `contract_address` field is reinterpreted as a contract class instead. When executing a delegate call, the kernel preserves the values of the `CallContext` `msgSender` and `storageContractAddress`.

At the contract level, a caller can initiate a delegate call via a `delegateCallPrivateFunction` or `delegateCallPublicFunction` oracle call. The caller is responsible for asserting that the returned `CallStackItem` has the `is_delegate_call` flag correctly set.
