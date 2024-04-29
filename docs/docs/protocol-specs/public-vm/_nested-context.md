The nested call's execution context is derived from the caller's context and the call instruction's arguments.

The following shorthand syntax is used to refer to nested context derivation in the ["Instruction Set"](./instruction-set) and other sections:

```jsx
// instr.args are { gasOffset, addrOffset, argsOffset, retOffset, retSize }

isStaticCall = instr.opcode == STATICCALL
isDelegateCall = instr.opcode == DELEGATECALL

nestedContext = deriveContext(context, instr.args, isStaticCall, isDelegateCall)
```

Nested context derivation is defined as follows:
```jsx
nestedExecutionEnvironment = ExecutionEnvironment {
    sender: isDelegateCall ? context.sender : context.address,
    address: M[addrOffset],
    storageAddress: isDelegateCall ? context.storageAddress : M[addrOffset],
    feePerL2Gas: context.environment.feePerL2Gas,
    feePerDaGas: context.environment.feePerDaGas,
    contractCallDepth: context.contractCallDepth + 1,
    contractCallPointer: context.worldStateAccessTrace.contractCalls.length + 1,
    globals: context.globals,
    isStaticCall: isStaticCall,
    isDelegateCall: isDelegateCall,
    calldata: context.memory[M[argsOffset]:M[argsOffset]+argsSize],
}

nestedMachineState = MachineState {
    l2GasLeft: context.machineState.memory[M[gasOffset]],
    daGasLeft: context.machineState.memory[M[gasOffset+1]],
    pc = 0,
    internalCallStack = [], // initialized as empty
    memory = [0, ..., 0],   // all 2^32 entries are initialized to zero
}
```


```jsx
nestedContext = AvmContext {
    environment: nestedExecutionEnvironment,
    machineState: nestedMachineState,
    worldState: context.worldState,
    worldStateAccessTrace: context.worldStateAccessTrace,
    accruedSubstate: { [], ... [], }, // all empty
    results: {reverted: false, output: []},
}
```

> `M[offset]` notation is shorthand for `context.machineState.memory[offset]`