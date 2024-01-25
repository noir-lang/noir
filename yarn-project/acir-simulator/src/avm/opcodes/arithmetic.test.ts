import { mock } from 'jest-mock-extended';

import { AvmMachineState } from '../avm_machine_state.js';
import { Field } from '../avm_memory_types.js';
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
    it('Should add correctly over field elements', () => {
      const a = new Field(1n);
      const b = new Field(2n);

      machineState.memory.set(0, a);
      machineState.memory.set(1, b);

      new Add(0, 1, 2).execute(machineState, stateManager);

      const expected = new Field(3n);
      const actual = machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });

    it('Should wrap around on addition', () => {
      const a = new Field(1n);
      const b = new Field(Field.MODULUS - 1n);

      machineState.memory.set(0, a);
      machineState.memory.set(1, b);

      new Add(0, 1, 2).execute(machineState, stateManager);

      const expected = new Field(0n);
      const actual = machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });
  });

  describe('Sub', () => {
    it('Should subtract correctly over field elements', () => {
      const a = new Field(1n);
      const b = new Field(2n);

      machineState.memory.set(0, a);
      machineState.memory.set(1, b);

      new Sub(0, 1, 2).execute(machineState, stateManager);

      const expected = new Field(Field.MODULUS - 1n);
      const actual = machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });
  });

  describe('Mul', () => {
    it('Should multiply correctly over field elements', () => {
      const a = new Field(2n);
      const b = new Field(3n);

      machineState.memory.set(0, a);
      machineState.memory.set(1, b);

      new Mul(0, 1, 2).execute(machineState, stateManager);

      const expected = new Field(6n);
      const actual = machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });

    it('Should wrap around on multiplication', () => {
      const a = new Field(2n);
      const b = new Field(Field.MODULUS / 2n - 1n);

      machineState.memory.set(0, a);
      machineState.memory.set(1, b);

      new Mul(0, 1, 2).execute(machineState, stateManager);

      const expected = new Field(Field.MODULUS - 3n);
      const actual = machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });
  });

  describe('Div', () => {
    it('Should perform field division', () => {
      const a = new Field(2n);
      const b = new Field(3n);

      machineState.memory.set(0, a);
      machineState.memory.set(1, b);

      new Div(0, 1, 2).execute(machineState, stateManager);

      // Note
      const actual = machineState.memory.get(2);
      const recovered = actual.mul(b);
      expect(recovered).toEqual(a);
    });
  });
});
