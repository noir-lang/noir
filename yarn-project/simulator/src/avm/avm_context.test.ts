import { AztecAddress, Fr } from '@aztec/circuits.js';

import { allSameExcept, anyAvmContextInputs, initContext } from './fixtures/index.js';

describe('Avm Context', () => {
  it('New call should fork context correctly', () => {
    const context = initContext();
    context.machineState.pc = 20;

    const newAddress = AztecAddress.random();
    const newCalldata = [new Fr(1), new Fr(2)];
    const newContext = context.createNestedContractCallContext(newAddress, newCalldata);

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
    const newContext = context.createNestedContractStaticCallContext(newAddress, newCalldata);

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
      }),
    );

    // We stringify to remove circular references (parentJournal)
    expect(JSON.stringify(newContext.persistableState)).toEqual(JSON.stringify(context.persistableState.fork()));
  });
});
