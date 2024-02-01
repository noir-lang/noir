import { mock } from 'jest-mock-extended';

import { AvmMachineState } from '../avm_machine_state.js';
import { Field } from '../avm_memory_types.js';
import { initExecutionEnvironment } from '../fixtures/index.js';
import { HostStorage } from '../journal/host_storage.js';
import { AvmJournal } from '../journal/journal.js';
import { EmitNoteHash, EmitNullifier, EmitUnencryptedLog, SendL2ToL1Message } from './accrued_substate.js';
import { StaticCallStorageAlterError } from './storage.js';

describe('Accrued Substate', () => {
  let journal: AvmJournal;
  let machineState: AvmMachineState;

  beforeEach(() => {
    const hostStorage = mock<HostStorage>();
    journal = new AvmJournal(hostStorage);
    machineState = new AvmMachineState(initExecutionEnvironment());
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
      machineState.memory.set(0, value);

      await new EmitNoteHash(/*indirect=*/ 0, /*offset=*/ 0).execute(machineState, journal);

      const journalState = journal.flush();
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
      machineState.memory.set(0, value);

      await new EmitNullifier(/*indirect=*/ 0, /*offset=*/ 0).execute(machineState, journal);

      const journalState = journal.flush();
      const expected = [value.toFr()];
      expect(journalState.newNullifiers).toEqual(expected);
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
      machineState.memory.setSlice(0, values);

      const length = values.length;

      await new EmitUnencryptedLog(/*indirect=*/ 0, /*offset=*/ startOffset, length).execute(machineState, journal);

      const journalState = journal.flush();
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
      machineState.memory.setSlice(0, values);

      const length = values.length;

      await new SendL2ToL1Message(/*indirect=*/ 0, /*offset=*/ startOffset, length).execute(machineState, journal);

      const journalState = journal.flush();
      const expected = values.map(v => v.toFr());
      expect(journalState.newL1Messages).toEqual([expected]);
    });
  });

  it('All substate instructions should fail within a static call', async () => {
    const executionEnvironment = initExecutionEnvironment({ isStaticCall: true });
    machineState = new AvmMachineState(executionEnvironment);

    const instructions = [
      new EmitNoteHash(/*indirect=*/ 0, /*offset=*/ 0),
      new EmitNullifier(/*indirect=*/ 0, /*offset=*/ 0),
      new EmitUnencryptedLog(/*indirect=*/ 0, /*offset=*/ 0, 1),
      new SendL2ToL1Message(/*indirect=*/ 0, /*offset=*/ 0, 1),
    ];

    for (const instruction of instructions) {
      await expect(instruction.execute(machineState, journal)).rejects.toThrow(StaticCallStorageAlterError);
    }
  });
});
