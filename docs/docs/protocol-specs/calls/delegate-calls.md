# Delegate calls

<!-- TODO: some kind of diagram showing the difference between an ordinary call and a delegatecall (there might be an existing diagram which explains this concept from ethereum-land, that we can use as inspiration). In particular showing who msg.sender is, what the storage contract address is, what the public keys are, and which class is executed (in a call vs a delegatecall)-->

Delegate calls are function calls against a contract class identifier instead of an instance. <!-- is this true? The storage space of the instance still needs to be known --> Any call -- synchronous or asynchronous -- can be made as a delegate call. The behavior of a delegate call is to execute the function code in the specified class identifier but in the context of the current instance. This opens the door to script-like executions and upgradeable contracts. Delegate calls are based on [EIP7](https://eips.ethereum.org/EIPS/eip-7).

At the protocol level, a delegate call is identified by a `is_delegate_call` flag in the `CircuitPublicInputs` of the `CallStackItem`. The `contract_address` field is reinterpreted as a contract class instead. When executing a delegate call, the kernel preserves the values of the `CallContext` `msgSender` and `storageContractAddress`.

<!--
"The `contract_address` field is reinterpreted as a contract class instead."

Isn't this a conflation of instances and classes? A contract address represents an instance; not a class. I'd favour introducing strong typing, so that a ClassId type cannot be used in the place of an AztecAddress type.

Perhaps the information that's needed for any kind of 'call' needs to be modified. At the moment it's: contract_address, function_selector. Perhaps it needs to become: contract_address, class_id, function_selector? Then, for an ordinary call, the class_id can be checked to match the one 'baked into ' the contract_address. For a delegatecall, the contract_address can be that of the calling contract (i.e. the storage contract address), and the class_id can be the target class?
-->

At the contract level, a caller can initiate a delegate call via a `delegateCallPrivateFunction` or `delegateCallPublicFunction` oracle call. The caller is responsible for asserting that the returned `CallStackItem` has the `is_delegate_call` flag correctly set.
