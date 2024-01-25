import { Fr } from '@aztec/foundation/fields';

import { initExecutionEnvironment } from './fixtures/index.js';

describe('Execution Environment', () => {
  const newAddress = new Fr(123456n);
  const calldata = [new Fr(1n), new Fr(2n), new Fr(3n)];

  it('New call should fork execution environment correctly', () => {
    const executionEnvironment = initExecutionEnvironment();
    const newExecutionEnvironment = executionEnvironment.newCall(newAddress, calldata);

    allTheSameExcept(executionEnvironment, newExecutionEnvironment, {
      address: newAddress,
      storageAddress: newAddress,
      calldata,
    });
  });

  it('New delegate call should fork execution environment correctly', () => {
    const executionEnvironment = initExecutionEnvironment();
    const newExecutionEnvironment = executionEnvironment.newDelegateCall(newAddress, calldata);

    allTheSameExcept(executionEnvironment, newExecutionEnvironment, {
      address: newAddress,
      isDelegateCall: true,
      calldata,
    });
  });

  it('New static call call should fork execution environment correctly', () => {
    const executionEnvironment = initExecutionEnvironment();
    const newExecutionEnvironment = executionEnvironment.newStaticCall(newAddress, calldata);

    allTheSameExcept(executionEnvironment, newExecutionEnvironment, {
      address: newAddress,
      storageAddress: newAddress,
      isStaticCall: true,
      calldata,
    });
  });
});

/**
 * Check all properties of one object are the same, except for the specified differentProperties
 */
function allTheSameExcept(referenceObject: any, comparingObject: any, differentProperties: Record<string, any>): void {
  for (const key in referenceObject) {
    if (Object.keys(differentProperties).includes(key)) {
      expect(comparingObject[key]).toEqual(differentProperties[key]);
    } else {
      expect(comparingObject[key]).toEqual(referenceObject[key]);
    }
  }
}
