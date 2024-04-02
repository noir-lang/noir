import { Fr } from '@aztec/foundation/fields';

import { jest } from '@jest/globals';
import { mock } from 'jest-mock-extended';

import { type CommitmentsDB, type PublicContractsDB, type PublicStateDB } from '../../index.js';
import { type AvmContext } from '../avm_context.js';
import { Field, Uint8 } from '../avm_memory_types.js';
import { adjustCalldataIndex, initContext } from '../fixtures/index.js';
import { HostStorage } from '../journal/host_storage.js';
import { AvmPersistableStateManager } from '../journal/journal.js';
import { encodeToBytecode } from '../serialization/bytecode_serialization.js';
import { Call, Return, Revert, StaticCall } from './external_calls.js';
import { type Instruction } from './instruction.js';
import { CalldataCopy } from './memory.js';
import { SStore } from './storage.js';

describe('External Calls', () => {
  let context: AvmContext;

  beforeEach(() => {
    const contractsDb = mock<PublicContractsDB>();
    const commitmentsDb = mock<CommitmentsDB>();
    const publicStateDb = mock<PublicStateDB>();
    const hostStorage = new HostStorage(publicStateDb, contractsDb, commitmentsDb);
    const journal = new AvmPersistableStateManager(hostStorage);
    context = initContext({ persistableState: journal });
  });

  describe('Call', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        Call.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // gasOffset
        ...Buffer.from('a2345678', 'hex'), // addrOffset
        ...Buffer.from('b2345678', 'hex'), // argsOffset
        ...Buffer.from('c2345678', 'hex'), // argsSize
        ...Buffer.from('d2345678', 'hex'), // retOffset
        ...Buffer.from('e2345678', 'hex'), // retSize
        ...Buffer.from('f2345678', 'hex'), // successOffset
        ...Buffer.from('f3345678', 'hex'), // temporaryFunctionSelectorOffset
      ]);
      const inst = new Call(
        /*indirect=*/ 0x01,
        /*gasOffset=*/ 0x12345678,
        /*addrOffset=*/ 0xa2345678,
        /*argsOffset=*/ 0xb2345678,
        /*argsSize=*/ 0xc2345678,
        /*retOffset=*/ 0xd2345678,
        /*retSize=*/ 0xe2345678,
        /*successOffset=*/ 0xf2345678,
        /*temporaryFunctionSelectorOffset=*/ 0xf3345678,
      );

      expect(Call.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3992): gas not implemented
    it('Should execute a call correctly', async () => {
      const gasOffset = 0;
      const gas = Fr.zero();
      const addrOffset = 1;
      const addr = new Fr(123456n);
      const argsOffset = 2;
      const args = [new Field(1n), new Field(2n), new Field(3n)];
      const argsSize = args.length;
      const retOffset = 8;
      const retSize = 2;
      const successOffset = 7;
      const otherContextInstructionsBytecode = encodeToBytecode([
        new CalldataCopy(
          /*indirect=*/ 0,
          /*csOffset=*/ adjustCalldataIndex(0),
          /*copySize=*/ argsSize,
          /*dstOffset=*/ 0,
        ),
        new SStore(/*indirect=*/ 0, /*srcOffset=*/ 0, /*size=*/ 1, /*slotOffset=*/ 0),
        new Return(/*indirect=*/ 0, /*retOffset=*/ 0, /*size=*/ 2),
      ]);

      context.machineState.memory.set(0, new Field(gas));
      context.machineState.memory.set(1, new Field(addr));
      context.machineState.memory.setSlice(2, args);
      jest
        .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
        .mockReturnValue(Promise.resolve(otherContextInstructionsBytecode));

      const instruction = new Call(
        /*indirect=*/ 0,
        gasOffset,
        addrOffset,
        argsOffset,
        argsSize,
        retOffset,
        retSize,
        successOffset,
        /*temporaryFunctionSelectorOffset=*/ 0,
      );
      await instruction.execute(context);

      const successValue = context.machineState.memory.get(successOffset);
      expect(successValue).toEqual(new Uint8(1n));

      const retValue = context.machineState.memory.getSlice(retOffset, retSize);
      expect(retValue).toEqual([new Field(1n), new Field(2n)]);

      // Check that the storage call has been merged into the parent journal
      const { currentStorageValue } = context.persistableState.flush();
      expect(currentStorageValue.size).toEqual(1);

      const nestedContractWrites = currentStorageValue.get(addr.toBigInt());
      expect(nestedContractWrites).toBeDefined();

      const slotNumber = 1n;
      const expectedStoredValue = new Fr(1n);
      expect(nestedContractWrites!.get(slotNumber)).toEqual(expectedStoredValue);
    });
  });

  describe('Static Call', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        StaticCall.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // gasOffset
        ...Buffer.from('a2345678', 'hex'), // addrOffset
        ...Buffer.from('b2345678', 'hex'), // argsOffset
        ...Buffer.from('c2345678', 'hex'), // argsSize
        ...Buffer.from('d2345678', 'hex'), // retOffset
        ...Buffer.from('e2345678', 'hex'), // retSize
        ...Buffer.from('f2345678', 'hex'), // successOffset
        ...Buffer.from('f3345678', 'hex'), // temporaryFunctionSelectorOffset
      ]);
      const inst = new StaticCall(
        /*indirect=*/ 0x01,
        /*gasOffset=*/ 0x12345678,
        /*addrOffset=*/ 0xa2345678,
        /*argsOffset=*/ 0xb2345678,
        /*argsSize=*/ 0xc2345678,
        /*retOffset=*/ 0xd2345678,
        /*retSize=*/ 0xe2345678,
        /*successOffset=*/ 0xf2345678,
        /*temporaryFunctionSelectorOffset=*/ 0xf3345678,
      );

      expect(StaticCall.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should fail if a static call attempts to touch storage', async () => {
      const gasOffset = 0;
      const gas = new Field(0);
      const addrOffset = 1;
      const addr = new Field(123456n);
      const argsOffset = 2;
      const args = [new Field(1n), new Field(2n), new Field(3n)];

      const argsSize = args.length;
      const retOffset = 8;
      const retSize = 2;
      const successOffset = 7;

      context.machineState.memory.set(0, gas);
      context.machineState.memory.set(1, addr);
      context.machineState.memory.setSlice(2, args);

      const otherContextInstructions: Instruction[] = [
        new CalldataCopy(
          /*indirect=*/ 0,
          /*csOffset=*/ adjustCalldataIndex(0),
          /*copySize=*/ argsSize,
          /*dstOffset=*/ 0,
        ),
        new SStore(/*indirect=*/ 0, /*srcOffset=*/ 1, /*size=*/ 1, /*slotOffset=*/ 0),
      ];

      const otherContextInstructionsBytecode = encodeToBytecode(otherContextInstructions);

      jest
        .spyOn(context.persistableState.hostStorage.contractsDb, 'getBytecode')
        .mockReturnValue(Promise.resolve(otherContextInstructionsBytecode));

      const instruction = new StaticCall(
        /*indirect=*/ 0,
        gasOffset,
        addrOffset,
        argsOffset,
        argsSize,
        retOffset,
        retSize,
        successOffset,
        /*temporaryFunctionSelectorOffset=*/ 0,
      );
      await instruction.execute(context);

      // No revert has occurred, but the nested execution has failed
      const successValue = context.machineState.memory.get(successOffset);
      expect(successValue).toEqual(new Uint8(0n));
    });
  });

  describe('RETURN', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        Return.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // returnOffset
        ...Buffer.from('a2345678', 'hex'), // copySize
      ]);
      const inst = new Return(/*indirect=*/ 0x01, /*returnOffset=*/ 0x12345678, /*copySize=*/ 0xa2345678);

      expect(Return.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should return data from the return opcode', async () => {
      const returnData = [new Fr(1n), new Fr(2n), new Fr(3n)];

      context.machineState.memory.set(0, new Field(1n));
      context.machineState.memory.set(1, new Field(2n));
      context.machineState.memory.set(2, new Field(3n));

      const instruction = new Return(/*indirect=*/ 0, /*returnOffset=*/ 0, returnData.length);
      await instruction.execute(context);

      expect(context.machineState.halted).toBe(true);
      expect(context.machineState.getResults()).toEqual({
        reverted: false,
        output: returnData,
      });
    });
  });

  describe('REVERT', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        Revert.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // returnOffset
        ...Buffer.from('a2345678', 'hex'), // retSize
      ]);
      const inst = new Revert(/*indirect=*/ 0x01, /*returnOffset=*/ 0x12345678, /*retSize=*/ 0xa2345678);

      expect(Revert.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should return data and revert from the revert opcode', async () => {
      const returnData = [new Fr(1n), new Fr(2n), new Fr(3n)];

      context.machineState.memory.set(0, new Field(1n));
      context.machineState.memory.set(1, new Field(2n));
      context.machineState.memory.set(2, new Field(3n));

      const instruction = new Revert(/*indirect=*/ 0, /*returnOffset=*/ 0, returnData.length);
      await instruction.execute(context);

      expect(context.machineState.halted).toBe(true);
      expect(context.machineState.getResults()).toEqual({
        reverted: true,
        output: returnData,
      });
    });
  });
});
