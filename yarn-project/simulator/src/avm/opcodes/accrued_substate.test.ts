import { mock } from 'jest-mock-extended';

import { CommitmentsDB } from '../../index.js';
import { AvmContext } from '../avm_context.js';
import { Field } from '../avm_memory_types.js';
import { InstructionExecutionError } from '../errors.js';
import { initContext, initExecutionEnvironment, initHostStorage } from '../fixtures/index.js';
import { AvmPersistableStateManager } from '../journal/journal.js';
import { EmitNoteHash, EmitNullifier, EmitUnencryptedLog, SendL2ToL1Message } from './accrued_substate.js';
import { StaticCallStorageAlterError } from './storage.js';

describe('Accrued Substate', () => {
  let context: AvmContext;

  beforeEach(() => {
    context = initContext();
  });

  describe('EmitNoteHash', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        EmitNoteHash.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // dstOffset
      ]);
      const inst = new EmitNoteHash(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678);

      expect(EmitNoteHash.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should append a new note hash correctly', async () => {
      const value = new Field(69n);
      context.machineState.memory.set(0, value);

      await new EmitNoteHash(/*indirect=*/ 0, /*offset=*/ 0).execute(context);

      const journalState = context.persistableState.flush();
      const expected = [value.toFr()];
      expect(journalState.newNoteHashes).toEqual(expected);
    });
  });

  describe('EmitNullifier', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        EmitNullifier.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // dstOffset
      ]);
      const inst = new EmitNullifier(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678);

      expect(EmitNullifier.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should append a new nullifier correctly', async () => {
      const value = new Field(69n);
      context.machineState.memory.set(0, value);

      await new EmitNullifier(/*indirect=*/ 0, /*offset=*/ 0).execute(context);

      const journalState = context.persistableState.flush();
      const expected = [value.toFr()];
      expect(journalState.newNullifiers).toEqual(expected);
    });

    it('Nullifier collision reverts (same nullifier emitted twice)', async () => {
      const value = new Field(69n);
      context.machineState.memory.set(0, value);

      await new EmitNullifier(/*indirect=*/ 0, /*offset=*/ 0).execute(context);
      await expect(new EmitNullifier(/*indirect=*/ 0, /*offset=*/ 0).execute(context)).rejects.toThrowError(
        new InstructionExecutionError(
          `Attempted to emit duplicate nullifier ${value.toFr()} (storage address: ${
            context.environment.storageAddress
          }).`,
        ),
      );
    });

    it('Nullifier collision reverts (nullifier exists in host state)', async () => {
      const value = new Field(69n);
      const storedLeafIndex = BigInt(42);

      // Mock the nullifiers db to return a stored leaf index
      const commitmentsDb = mock<CommitmentsDB>();
      commitmentsDb.getNullifierIndex.mockResolvedValue(Promise.resolve(storedLeafIndex));
      const hostStorage = initHostStorage({ commitmentsDb });
      context = initContext({ persistableState: new AvmPersistableStateManager(hostStorage) });

      context.machineState.memory.set(0, value);
      await expect(new EmitNullifier(/*indirect=*/ 0, /*offset=*/ 0).execute(context)).rejects.toThrowError(
        new InstructionExecutionError(
          `Attempted to emit duplicate nullifier ${value.toFr()} (storage address: ${
            context.environment.storageAddress
          }).`,
        ),
      );
    });
  });

  describe('EmitUnencryptedLog', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        EmitUnencryptedLog.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // offset
        ...Buffer.from('a2345678', 'hex'), // length
      ]);
      const inst = new EmitUnencryptedLog(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678, /*length=*/ 0xa2345678);

      expect(EmitUnencryptedLog.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should append unencrypted logs correctly', async () => {
      const startOffset = 0;

      const values = [new Field(69n), new Field(420n), new Field(Field.MODULUS - 1n)];
      context.machineState.memory.setSlice(0, values);

      const length = values.length;

      await new EmitUnencryptedLog(/*indirect=*/ 0, /*offset=*/ startOffset, length).execute(context);

      const journalState = context.persistableState.flush();
      const expected = values.map(v => v.toFr());
      expect(journalState.newLogs).toEqual([expected]);
    });
  });

  describe('SendL2ToL1Message', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        SendL2ToL1Message.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // offset
        ...Buffer.from('a2345678', 'hex'), // length
      ]);
      const inst = new SendL2ToL1Message(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678, /*length=*/ 0xa2345678);

      expect(SendL2ToL1Message.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should append l1 to l2 messages correctly', async () => {
      const startOffset = 0;

      const values = [new Field(69n), new Field(420n), new Field(Field.MODULUS - 1n)];
      context.machineState.memory.setSlice(0, values);

      const length = values.length;

      await new SendL2ToL1Message(/*indirect=*/ 0, /*offset=*/ startOffset, length).execute(context);

      const journalState = context.persistableState.flush();
      const expected = values.map(v => v.toFr());
      expect(journalState.newL1Messages).toEqual([expected]);
    });
  });

  it('All substate instructions should fail within a static call', async () => {
    context = initContext({ env: initExecutionEnvironment({ isStaticCall: true }) });

    const instructions = [
      new EmitNoteHash(/*indirect=*/ 0, /*offset=*/ 0),
      new EmitNullifier(/*indirect=*/ 0, /*offset=*/ 0),
      new EmitUnencryptedLog(/*indirect=*/ 0, /*offset=*/ 0, 1),
      new SendL2ToL1Message(/*indirect=*/ 0, /*offset=*/ 0, 1),
    ];

    for (const instruction of instructions) {
      await expect(instruction.execute(context)).rejects.toThrow(StaticCallStorageAlterError);
    }
  });
});
