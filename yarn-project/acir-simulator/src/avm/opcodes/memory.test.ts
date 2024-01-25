import { Fr } from '@aztec/foundation/fields';

import { mock } from 'jest-mock-extended';

import { AvmMachineState } from '../avm_machine_state.js';
import { Field, TypeTag, Uint8, Uint16, Uint32, Uint64, Uint128 } from '../avm_memory_types.js';
import { AvmStateManager } from '../avm_state_manager.js';
import { CMov, CalldataCopy, Cast, Mov, Set } from './memory.js';

describe('Memory instructions', () => {
  let machineState: AvmMachineState;
  let stateManager = mock<AvmStateManager>();

  beforeEach(() => {
    machineState = new AvmMachineState([]);
    stateManager = mock<AvmStateManager>();
  });

  describe('SET', () => {
    it('should correctly set value and tag (uninitialized)', () => {
      new Set(/*value=*/ 1234n, /*offset=*/ 1, TypeTag.UINT16).execute(machineState, stateManager);

      const actual = machineState.memory.get(1);
      const tag = machineState.memory.getTag(1);

      expect(actual).toEqual(new Uint16(1234n));
      expect(tag).toEqual(TypeTag.UINT16);
    });

    it('should correctly set value and tag (overwriting)', () => {
      machineState.memory.set(1, new Field(27));

      new Set(/*value=*/ 1234n, /*offset=*/ 1, TypeTag.UINT32).execute(machineState, stateManager);

      const actual = machineState.memory.get(1);
      const tag = machineState.memory.getTag(1);

      expect(actual).toEqual(new Uint32(1234n));
      expect(tag).toEqual(TypeTag.UINT32);
    });
  });

  describe('CAST', () => {
    it('Should upcast between integral types', () => {
      machineState.memory.set(0, new Uint8(20n));
      machineState.memory.set(1, new Uint16(65000n));
      machineState.memory.set(2, new Uint32(1n << 30n));
      machineState.memory.set(3, new Uint64(1n << 50n));
      machineState.memory.set(4, new Uint128(1n << 100n));

      [
        new Cast(/*aOffset=*/ 0, /*dstOffset=*/ 10, TypeTag.UINT16),
        new Cast(/*aOffset=*/ 1, /*dstOffset=*/ 11, TypeTag.UINT32),
        new Cast(/*aOffset=*/ 2, /*dstOffset=*/ 12, TypeTag.UINT64),
        new Cast(/*aOffset=*/ 3, /*dstOffset=*/ 13, TypeTag.UINT128),
        new Cast(/*aOffset=*/ 4, /*dstOffset=*/ 14, TypeTag.UINT128),
      ].forEach(i => i.execute(machineState, stateManager));

      const actual = machineState.memory.getSlice(/*offset=*/ 10, /*size=*/ 5);
      expect(actual).toEqual([
        new Uint16(20n),
        new Uint32(65000n),
        new Uint64(1n << 30n),
        new Uint128(1n << 50n),
        new Uint128(1n << 100n),
      ]);
      const tags = machineState.memory.getSliceTags(/*offset=*/ 10, /*size=*/ 5);
      expect(tags).toEqual([TypeTag.UINT16, TypeTag.UINT32, TypeTag.UINT64, TypeTag.UINT128, TypeTag.UINT128]);
    });

    it('Should downcast (truncating) between integral types', () => {
      machineState.memory.set(0, new Uint8(20n));
      machineState.memory.set(1, new Uint16(65000n));
      machineState.memory.set(2, new Uint32((1n << 30n) - 1n));
      machineState.memory.set(3, new Uint64((1n << 50n) - 1n));
      machineState.memory.set(4, new Uint128((1n << 100n) - 1n));

      [
        new Cast(/*aOffset=*/ 0, /*dstOffset=*/ 10, TypeTag.UINT8),
        new Cast(/*aOffset=*/ 1, /*dstOffset=*/ 11, TypeTag.UINT8),
        new Cast(/*aOffset=*/ 2, /*dstOffset=*/ 12, TypeTag.UINT16),
        new Cast(/*aOffset=*/ 3, /*dstOffset=*/ 13, TypeTag.UINT32),
        new Cast(/*aOffset=*/ 4, /*dstOffset=*/ 14, TypeTag.UINT64),
      ].forEach(i => i.execute(machineState, stateManager));

      const actual = machineState.memory.getSlice(/*offset=*/ 10, /*size=*/ 5);
      expect(actual).toEqual([
        new Uint8(20n),
        new Uint8(232),
        new Uint16((1n << 16n) - 1n),
        new Uint32((1n << 32n) - 1n),
        new Uint64((1n << 64n) - 1n),
      ]);
      const tags = machineState.memory.getSliceTags(/*offset=*/ 10, /*size=*/ 5);
      expect(tags).toEqual([TypeTag.UINT8, TypeTag.UINT8, TypeTag.UINT16, TypeTag.UINT32, TypeTag.UINT64]);
    });

    it('Should upcast from integral types to field', () => {
      machineState.memory.set(0, new Uint8(20n));
      machineState.memory.set(1, new Uint16(65000n));
      machineState.memory.set(2, new Uint32(1n << 30n));
      machineState.memory.set(3, new Uint64(1n << 50n));
      machineState.memory.set(4, new Uint128(1n << 100n));

      [
        new Cast(/*aOffset=*/ 0, /*dstOffset=*/ 10, TypeTag.FIELD),
        new Cast(/*aOffset=*/ 1, /*dstOffset=*/ 11, TypeTag.FIELD),
        new Cast(/*aOffset=*/ 2, /*dstOffset=*/ 12, TypeTag.FIELD),
        new Cast(/*aOffset=*/ 3, /*dstOffset=*/ 13, TypeTag.FIELD),
        new Cast(/*aOffset=*/ 4, /*dstOffset=*/ 14, TypeTag.FIELD),
      ].forEach(i => i.execute(machineState, stateManager));

      const actual = machineState.memory.getSlice(/*offset=*/ 10, /*size=*/ 5);
      expect(actual).toEqual([
        new Field(20n),
        new Field(65000n),
        new Field(1n << 30n),
        new Field(1n << 50n),
        new Field(1n << 100n),
      ]);
      const tags = machineState.memory.getSliceTags(/*offset=*/ 10, /*size=*/ 5);
      expect(tags).toEqual([TypeTag.FIELD, TypeTag.FIELD, TypeTag.FIELD, TypeTag.FIELD, TypeTag.FIELD]);
    });

    it('Should downcast (truncating) from field to integral types', () => {
      machineState.memory.set(0, new Field((1n << 200n) - 1n));
      machineState.memory.set(1, new Field((1n << 200n) - 1n));
      machineState.memory.set(2, new Field((1n << 200n) - 1n));
      machineState.memory.set(3, new Field((1n << 200n) - 1n));
      machineState.memory.set(4, new Field((1n << 200n) - 1n));

      [
        new Cast(/*aOffset=*/ 0, /*dstOffset=*/ 10, TypeTag.UINT8),
        new Cast(/*aOffset=*/ 1, /*dstOffset=*/ 11, TypeTag.UINT16),
        new Cast(/*aOffset=*/ 2, /*dstOffset=*/ 12, TypeTag.UINT32),
        new Cast(/*aOffset=*/ 3, /*dstOffset=*/ 13, TypeTag.UINT64),
        new Cast(/*aOffset=*/ 4, /*dstOffset=*/ 14, TypeTag.UINT128),
      ].forEach(i => i.execute(machineState, stateManager));

      const actual = machineState.memory.getSlice(/*offset=*/ 10, /*size=*/ 5);
      expect(actual).toEqual([
        new Uint8((1n << 8n) - 1n),
        new Uint16((1n << 16n) - 1n),
        new Uint32((1n << 32n) - 1n),
        new Uint64((1n << 64n) - 1n),
        new Uint128((1n << 128n) - 1n),
      ]);
      const tags = machineState.memory.getSliceTags(/*offset=*/ 10, /*size=*/ 5);
      expect(tags).toEqual([TypeTag.UINT8, TypeTag.UINT16, TypeTag.UINT32, TypeTag.UINT64, TypeTag.UINT128]);
    });

    it('Should cast between field elements', () => {
      machineState.memory.set(0, new Field(12345678n));

      new Cast(/*aOffset=*/ 0, /*dstOffset=*/ 1, TypeTag.FIELD).execute(machineState, stateManager);

      const actual = machineState.memory.get(1);
      expect(actual).toEqual(new Field(12345678n));
      const tags = machineState.memory.getTag(1);
      expect(tags).toEqual(TypeTag.FIELD);
    });
  });

  describe('MOV', () => {
    it('Should move integrals on different memory cells', () => {
      machineState.memory.set(1, new Uint16(27));
      new Mov(/*offsetA=*/ 1, /*offsetA=*/ 2).execute(machineState, stateManager);

      const actual = machineState.memory.get(2);
      const tag = machineState.memory.getTag(2);

      expect(actual).toEqual(new Uint16(27n));
      expect(tag).toEqual(TypeTag.UINT16);
    });

    it('Should move field elements on different memory cells', () => {
      machineState.memory.set(1, new Field(27));
      new Mov(/*offsetA=*/ 1, /*offsetA=*/ 2).execute(machineState, stateManager);

      const actual = machineState.memory.get(2);
      const tag = machineState.memory.getTag(2);

      expect(actual).toEqual(new Field(27n));
      expect(tag).toEqual(TypeTag.FIELD);
    });
  });

  describe('CMOV', () => {
    it('Should move A if COND is true, on different memory cells (integral condition)', () => {
      machineState.memory.set(0, new Uint32(123)); // A
      machineState.memory.set(1, new Uint16(456)); // B
      machineState.memory.set(2, new Uint8(2)); // Condition

      new CMov(/*aOffset=*/ 0, /*bOffset=*/ 1, /*condOffset=*/ 2, /*dstOffset=*/ 3).execute(machineState, stateManager);

      const actual = machineState.memory.get(3);
      const tag = machineState.memory.getTag(3);
      expect(actual).toEqual(new Uint32(123));
      expect(tag).toEqual(TypeTag.UINT32);
    });

    it('Should move B if COND is false, on different memory cells (integral condition)', () => {
      machineState.memory.set(0, new Uint32(123)); // A
      machineState.memory.set(1, new Uint16(456)); // B
      machineState.memory.set(2, new Uint8(0)); // Condition

      new CMov(/*aOffset=*/ 0, /*bOffset=*/ 1, /*condOffset=*/ 2, /*dstOffset=*/ 3).execute(machineState, stateManager);

      const actual = machineState.memory.get(3);
      const tag = machineState.memory.getTag(3);
      expect(actual).toEqual(new Uint16(456));
      expect(tag).toEqual(TypeTag.UINT16);
    });

    it('Should move A if COND is true, on different memory cells (field condition)', () => {
      machineState.memory.set(0, new Uint32(123)); // A
      machineState.memory.set(1, new Uint16(456)); // B
      machineState.memory.set(2, new Field(1)); // Condition

      new CMov(/*aOffset=*/ 0, /*bOffset=*/ 1, /*condOffset=*/ 2, /*dstOffset=*/ 3).execute(machineState, stateManager);

      const actual = machineState.memory.get(3);
      const tag = machineState.memory.getTag(3);
      expect(actual).toEqual(new Uint32(123));
      expect(tag).toEqual(TypeTag.UINT32);
    });

    it('Should move B if COND is false, on different memory cells (integral condition)', () => {
      machineState.memory.set(0, new Uint32(123)); // A
      machineState.memory.set(1, new Uint16(456)); // B
      machineState.memory.set(2, new Field(0)); // Condition

      new CMov(/*aOffset=*/ 0, /*bOffset=*/ 1, /*condOffset=*/ 2, /*dstOffset=*/ 3).execute(machineState, stateManager);

      const actual = machineState.memory.get(3);
      const tag = machineState.memory.getTag(3);
      expect(actual).toEqual(new Uint16(456));
      expect(tag).toEqual(TypeTag.UINT16);
    });
  });

  describe('CALLDATACOPY', () => {
    it('Writes nothing if size is 0', () => {
      const calldata = [new Fr(1n), new Fr(2n), new Fr(3n)];
      machineState = new AvmMachineState(calldata);
      machineState.memory.set(0, new Uint16(12)); // Some previous data to be overwritten

      new CalldataCopy(/*cdOffset=*/ 0, /*copySize=*/ 0, /*dstOffset=*/ 0).execute(machineState, stateManager);

      const actual = machineState.memory.get(0);
      expect(actual).toEqual(new Uint16(12));
    });

    it('Copies all calldata', () => {
      const calldata = [new Fr(1n), new Fr(2n), new Fr(3n)];
      machineState = new AvmMachineState(calldata);
      machineState.memory.set(0, new Uint16(12)); // Some previous data to be overwritten

      new CalldataCopy(/*cdOffset=*/ 0, /*copySize=*/ 3, /*dstOffset=*/ 0).execute(machineState, stateManager);

      const actual = machineState.memory.getSlice(/*offset=*/ 0, /*size=*/ 3);
      expect(actual).toEqual([new Field(1), new Field(2), new Field(3)]);
    });

    it('Copies slice of calldata', () => {
      const calldata = [new Fr(1n), new Fr(2n), new Fr(3n)];
      machineState = new AvmMachineState(calldata);
      machineState.memory.set(0, new Uint16(12)); // Some previous data to be overwritten

      new CalldataCopy(/*cdOffset=*/ 1, /*copySize=*/ 2, /*dstOffset=*/ 0).execute(machineState, stateManager);

      const actual = machineState.memory.getSlice(/*offset=*/ 0, /*size=*/ 2);
      expect(actual).toEqual([new Field(2), new Field(3)]);
    });

    // TODO: check bad cases (i.e., out of bounds)
  });
});
