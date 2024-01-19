import { Fr } from '@aztec/foundation/fields';

import { mock } from 'jest-mock-extended';

import { AvmMachineState } from '../avm_machine_state.js';
import { AvmStateManager } from '../avm_state_manager.js';
import { Add, Div, Mul, Sub } from './arithmetic.js';

describe('Arithmetic Instructions', () => {
  let machineState: AvmMachineState;
  let stateManager = mock<AvmStateManager>();

  beforeEach(() => {
    machineState = new AvmMachineState([]);
    stateManager = mock<AvmStateManager>();
  });

  describe('Add', () => {
    it('Should add correctly over Fr type', () => {
      const a = new Fr(1n);
      const b = new Fr(2n);

      machineState.writeMemory(0, a);
      machineState.writeMemory(1, b);

      new Add(0, 1, 2).execute(machineState, stateManager);

      const expected = new Fr(3n);
      const actual = machineState.readMemory(2);
      expect(actual).toEqual(expected);
    });

    it('Should wrap around on addition', () => {
      const a = new Fr(1n);
      const b = new Fr(Fr.MODULUS - 1n);

      machineState.writeMemory(0, a);
      machineState.writeMemory(1, b);

      new Add(0, 1, 2).execute(machineState, stateManager);

      const expected = new Fr(0n);
      const actual = machineState.readMemory(3);
      expect(actual).toEqual(expected);
    });
  });

  describe('Sub', () => {
    it('Should subtract correctly over Fr type', () => {
      const a = new Fr(1n);
      const b = new Fr(2n);

      machineState.writeMemory(0, a);
      machineState.writeMemory(1, b);

      new Sub(0, 1, 2).execute(machineState, stateManager);

      const expected = new Fr(Fr.MODULUS - 1n);
      const actual = machineState.readMemory(2);
      expect(actual).toEqual(expected);
    });
  });

  describe('Mul', () => {
    it('Should multiply correctly over Fr type', () => {
      const a = new Fr(2n);
      const b = new Fr(3n);

      machineState.writeMemory(0, a);
      machineState.writeMemory(1, b);

      new Mul(0, 1, 2).execute(machineState, stateManager);

      const expected = new Fr(6n);
      const actual = machineState.readMemory(2);
      expect(actual).toEqual(expected);
    });

    it('Should wrap around on multiplication', () => {
      const a = new Fr(2n);
      const b = new Fr(Fr.MODULUS / 2n - 1n);

      machineState.writeMemory(0, a);
      machineState.writeMemory(1, b);

      new Mul(0, 1, 2).execute(machineState, stateManager);

      const expected = new Fr(Fr.MODULUS - 3n);
      const actual = machineState.readMemory(2);
      expect(actual).toEqual(expected);
    });
  });

  describe('Div', () => {
    it('Should perform field division', () => {
      const a = new Fr(2n);
      const b = new Fr(3n);

      machineState.writeMemory(0, a);
      machineState.writeMemory(1, b);

      new Div(0, 1, 2).execute(machineState, stateManager);

      // Note
      const actual = machineState.readMemory(2);
      const recovered = actual.mul(b);
      expect(recovered).toEqual(a);
    });
  });
});
