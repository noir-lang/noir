import { AztecAddress, Fr } from '@aztec/circuits.js';

import { allSameExcept, anyAvmContextInputs, initContext } from './fixtures/index.js';

describe('Avm Context', () => {
  it('New call should fork context correctly', () => {
    const context = initContext();
    context.machineState.pc = 20;

    const newAddress = AztecAddress.random();
    const newCalldata = [new Fr(1), new Fr(2)];
    const allocatedGas = { l1Gas: 1, l2Gas: 2, daGas: 3 }; // How much of the current call gas we pass to the nested call
    const newContext = context.createNestedContractCallContext(newAddress, newCalldata, allocatedGas, 'CALL');

    expect(newContext.environment).toEqual(
      allSameExcept(context.environment, {
        address: newAddress,
        storageAddress: newAddress,
        // Calldata also includes AvmContextInputs
        calldata: anyAvmContextInputs().concat(newCalldata),
        isStaticCall: false,
      }),
    );
    expect(newContext.machineState).toEqual(
      allSameExcept(context.machineState, {
        pc: 0,
        l1GasLeft: 1,
        l2GasLeft: 2,
        daGasLeft: 3,
      }),
    );

    // We stringify to remove circular references (parentJournal)
    expect(JSON.stringify(newContext.persistableState)).toEqual(JSON.stringify(context.persistableState.fork()));
  });

  it('New static call should fork context correctly', () => {
    const context = initContext();
    context.machineState.pc = 20;

    const newAddress = AztecAddress.random();
    const newCalldata = [new Fr(1), new Fr(2)];
    const allocatedGas = { l1Gas: 1, l2Gas: 2, daGas: 3 };
    const newContext = context.createNestedContractCallContext(newAddress, newCalldata, allocatedGas, 'STATICCALL');

    expect(newContext.environment).toEqual(
      allSameExcept(context.environment, {
        address: newAddress,
        storageAddress: newAddress,
        // Calldata also includes AvmContextInputs
        calldata: anyAvmContextInputs().concat(newCalldata),
        isStaticCall: true,
      }),
    );
    expect(newContext.machineState).toEqual(
      allSameExcept(context.machineState, {
        pc: 0,
        l1GasLeft: 1,
        l2GasLeft: 2,
        daGasLeft: 3,
      }),
    );

    // We stringify to remove circular references (parentJournal)
    expect(JSON.stringify(newContext.persistableState)).toEqual(JSON.stringify(context.persistableState.fork()));
  });
});
