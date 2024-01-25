import { MockProxy, mock } from 'jest-mock-extended';

import { AvmMachineState } from '../avm_machine_state.js';
import { Field } from '../avm_memory_types.js';
import { initExecutionEnvironment } from '../fixtures/index.js';
import { AvmJournal } from '../journal/journal.js';
import { Add, Div, Mul, Sub } from './arithmetic.js';

describe('Arithmetic Instructions', () => {
  let machineState: AvmMachineState;
  let journal: MockProxy<AvmJournal>;

  beforeEach(async () => {
    machineState = new AvmMachineState(initExecutionEnvironment());
    journal = mock<AvmJournal>();
  });

  describe('Add', () => {
    it('Should add correctly over field elements', async () => {
      const a = new Field(1n);
      const b = new Field(2n);

      machineState.memory.set(0, a);
      machineState.memory.set(1, b);

      await new Add(0, 1, 2).execute(machineState, journal);

      const expected = new Field(3n);
      const actual = machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });

    it('Should wrap around on addition', async () => {
      const a = new Field(1n);
      const b = new Field(Field.MODULUS - 1n);

      machineState.memory.set(0, a);
      machineState.memory.set(1, b);

      await new Add(0, 1, 2).execute(machineState, journal);

      const expected = new Field(0n);
      const actual = machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });
  });

  describe('Sub', () => {
    it('Should subtract correctly over field elements', async () => {
      const a = new Field(1n);
      const b = new Field(2n);

      machineState.memory.set(0, a);
      machineState.memory.set(1, b);

      await new Sub(0, 1, 2).execute(machineState, journal);

      const expected = new Field(Field.MODULUS - 1n);
      const actual = machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });
  });

  describe('Mul', () => {
    it('Should multiply correctly over field elements', async () => {
      const a = new Field(2n);
      const b = new Field(3n);

      machineState.memory.set(0, a);
      machineState.memory.set(1, b);

      await new Mul(0, 1, 2).execute(machineState, journal);

      const expected = new Field(6n);
      const actual = machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });

    it('Should wrap around on multiplication', async () => {
      const a = new Field(2n);
      const b = new Field(Field.MODULUS / 2n - 1n);

      machineState.memory.set(0, a);
      machineState.memory.set(1, b);

      await new Mul(0, 1, 2).execute(machineState, journal);

      const expected = new Field(Field.MODULUS - 3n);
      const actual = machineState.memory.get(2);
      expect(actual).toEqual(expected);
    });
  });

  describe('Div', () => {
    it('Should perform field division', async () => {
      const a = new Field(2n);
      const b = new Field(3n);

      machineState.memory.set(0, a);
      machineState.memory.set(1, b);

      await new Div(0, 1, 2).execute(machineState, journal);

      // Note
      const actual = machineState.memory.get(2);
      const recovered = actual.mul(b);
      expect(recovered).toEqual(a);
    });
  });
});
