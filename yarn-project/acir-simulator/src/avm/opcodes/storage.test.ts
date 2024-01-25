import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import { MockProxy, mock } from 'jest-mock-extended';

import { AvmMachineState } from '../avm_machine_state.js';
import { AvmStateManager } from '../avm_state_manager.js';
import { initExecutionEnvironment } from '../fixtures/index.js';
import { SLoad, SStore } from './storage.js';

describe('Storage Instructions', () => {
  let stateManager: MockProxy<AvmStateManager>;
  let machineState: AvmMachineState;
  const address = AztecAddress.random();

  beforeEach(() => {
    stateManager = mock<AvmStateManager>();

    const executionEnvironment = initExecutionEnvironment({ address, storageAddress: address });
    machineState = new AvmMachineState(executionEnvironment);
  });

  it('Sstore should Write into storage', () => {
    const a = new Fr(1n);
    const b = new Fr(2n);

    machineState.memory.set(0, a);
    machineState.memory.set(1, b);

    new SStore(0, 1).execute(machineState, stateManager);

    expect(stateManager.store).toBeCalledWith(address, a, b);
  });

  it('Sload should Read into storage', async () => {
    // Mock response
    const expectedResult = new Fr(1n);
    stateManager.read.mockReturnValueOnce(Promise.resolve(expectedResult));

    const a = new Fr(1n);
    const b = new Fr(2n);

    machineState.memory.set(0, a);
    machineState.memory.set(1, b);

    await new SLoad(0, 1).execute(machineState, stateManager);

    expect(stateManager.read).toBeCalledWith(address, a);

    const actual = machineState.memory.get(1);
    expect(actual).toEqual(expectedResult);
  });
});
