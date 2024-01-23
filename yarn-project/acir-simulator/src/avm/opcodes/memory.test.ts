import { Fr } from '@aztec/foundation/fields';

import { mock } from 'jest-mock-extended';

import { AvmMachineState } from '../avm_machine_state.js';
import { AvmStateManager } from '../avm_state_manager.js';
import { CMov, CalldataCopy, Cast, Mov, Set } from './memory.js';

describe('Memory instructions', () => {
  let machineState: AvmMachineState;
  let stateManager = mock<AvmStateManager>();

  beforeEach(() => {
    machineState = new AvmMachineState([]);
    stateManager = mock<AvmStateManager>();
  });

  it('Should SET memory correctly', () => {
    const value = 123456n;

    new Set(value, 1).execute(machineState, stateManager);

    const expected = new Fr(value);
    const actual = machineState.readMemory(1);
    expect(actual).toEqual(expected);
  });

  // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3987): tags are not implemented yet - this will behave as a mov
  describe('CAST', () => {
    it('Should work correctly on different memory cells', () => {
      const value = new Fr(123456n);

      machineState.writeMemory(0, value);

      new Cast(/*aOffset=*/ 0, /*dstOffset=*/ 1).execute(machineState, stateManager);

      const actual = machineState.readMemory(1);
      expect(actual).toEqual(value);
    });

    it('Should work correctly on same memory cell', () => {
      const value = new Fr(123456n);

      machineState.writeMemory(0, value);

      new Cast(/*aOffset=*/ 0, /*dstOffset=*/ 0).execute(machineState, stateManager);

      const actual = machineState.readMemory(0);
      expect(actual).toEqual(value);
    });
  });

  describe('MOV', () => {
    it('Should work correctly on different memory cells', () => {
      const value = new Fr(123456n);

      machineState.writeMemory(0, value);

      new Mov(/*aOffset=*/ 0, /*dstOffset=*/ 1).execute(machineState, stateManager);

      const actual = machineState.readMemory(1);
      expect(actual).toEqual(value);
    });

    it('Should work correctly on same memory cell', () => {
      const value = new Fr(123456n);

      machineState.writeMemory(0, value);

      new Mov(/*aOffset=*/ 0, /*dstOffset=*/ 0).execute(machineState, stateManager);

      const actual = machineState.readMemory(0);
      expect(actual).toEqual(value);
    });
  });

  describe('MOV', () => {
    it('Should move A if COND is true, on different memory cells', () => {
      const valueA = new Fr(123456n);
      const valueB = new Fr(80n);
      const valueCondition = new Fr(22n);

      machineState.writeMemory(0, valueA);
      machineState.writeMemory(1, valueB);
      machineState.writeMemory(2, valueCondition);

      new CMov(/*aOffset=*/ 0, /*bOffset=*/ 1, /*condOffset=*/ 2, /*dstOffset=*/ 3).execute(machineState, stateManager);

      const actual = machineState.readMemory(3);
      expect(actual).toEqual(valueA);
    });

    it('Should move B if COND is false, on different memory cells', () => {
      const valueA = new Fr(123456n);
      const valueB = new Fr(80n);
      const valueCondition = new Fr(0n);

      machineState.writeMemory(0, valueA);
      machineState.writeMemory(1, valueB);
      machineState.writeMemory(2, valueCondition);

      new CMov(/*aOffset=*/ 0, /*bOffset=*/ 1, /*condOffset=*/ 2, /*dstOffset=*/ 3).execute(machineState, stateManager);

      const actual = machineState.readMemory(3);
      expect(actual).toEqual(valueB);
    });

    it('Should move A if COND is true, on overlapping memory cells', () => {
      const valueA = new Fr(123456n);
      const valueB = new Fr(80n);
      const valueCondition = new Fr(22n);

      machineState.writeMemory(0, valueA);
      machineState.writeMemory(1, valueB);
      machineState.writeMemory(2, valueCondition);

      new CMov(/*aOffset=*/ 0, /*bOffset=*/ 1, /*condOffset=*/ 2, /*dstOffset=*/ 2).execute(machineState, stateManager);

      const actual = machineState.readMemory(2);
      expect(actual).toEqual(valueA);
    });

    it('Should move B if COND is false, on overlapping memory cells', () => {
      const valueA = new Fr(123456n);
      const valueB = new Fr(80n);
      const valueCondition = new Fr(0n);

      machineState.writeMemory(0, valueA);
      machineState.writeMemory(1, valueB);
      machineState.writeMemory(2, valueCondition);

      new CMov(/*aOffset=*/ 0, /*bOffset=*/ 1, /*condOffset=*/ 2, /*dstOffset=*/ 2).execute(machineState, stateManager);

      const actual = machineState.readMemory(2);
      expect(actual).toEqual(valueB);
    });
  });

  describe('CALLDATA', () => {
    it('Writes nothing if size is 0', () => {
      const previousValue = new Fr(123456n);
      const calldata = [new Fr(1n), new Fr(2n), new Fr(3n)];

      machineState = new AvmMachineState(calldata);
      machineState.writeMemory(0, previousValue);

      new CalldataCopy(/*cdOffset=*/ 2, /*copySize=*/ 0, /*dstOffset=*/ 0).execute(machineState, stateManager);

      const actual = machineState.readMemory(0);
      expect(actual).toEqual(previousValue);
    });

    it('Copies all calldata', () => {
      const previousValue = new Fr(123456n);
      const calldata = [new Fr(1n), new Fr(2n), new Fr(3n)];

      machineState = new AvmMachineState(calldata);
      machineState.writeMemory(0, previousValue);

      new CalldataCopy(/*cdOffset=*/ 0, /*copySize=*/ 3, /*dstOffset=*/ 0).execute(machineState, stateManager);

      const actual = machineState.readMemoryChunk(/*offset=*/ 0, /*size=*/ 3);
      expect(actual).toEqual(calldata);
    });

    it('Copies slice of calldata', () => {
      const previousValue = new Fr(123456n);
      const calldata = [new Fr(1n), new Fr(2n), new Fr(3n)];

      machineState = new AvmMachineState(calldata);
      machineState.writeMemory(0, previousValue);

      new CalldataCopy(/*cdOffset=*/ 1, /*copySize=*/ 2, /*dstOffset=*/ 0).execute(machineState, stateManager);

      const expected = calldata.slice(1);
      const actual = machineState.readMemoryChunk(/*offset=*/ 0, /*size=*/ 2);
      expect(actual).toEqual(expected);
    });

    // TODO: check bad cases (i.e., out of bounds)
  });
});
