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

  it('Should append a new note hash correctly', async () => {
    const value = new Field(69n);
    machineState.memory.set(0, value);

    await new EmitNoteHash(0).execute(machineState, journal);

    const journalState = journal.flush();
    const expected = [value.toFr()];
    expect(journalState.newNoteHashes).toEqual(expected);
  });

  it('Should append a new nullifier correctly', async () => {
    const value = new Field(69n);
    machineState.memory.set(0, value);

    await new EmitNullifier(0).execute(machineState, journal);

    const journalState = journal.flush();
    const expected = [value.toFr()];
    expect(journalState.newNullifiers).toEqual(expected);
  });

  it('Should append unencrypted logs correctly', async () => {
    const startOffset = 0;

    const values = [new Field(69n), new Field(420n), new Field(Field.MODULUS - 1n)];
    machineState.memory.setSlice(0, values);

    const length = values.length;

    await new EmitUnencryptedLog(startOffset, length).execute(machineState, journal);

    const journalState = journal.flush();
    const expected = values.map(v => v.toFr());
    expect(journalState.newLogs).toEqual([expected]);
  });

  it('Should append l1 to l2 messages correctly', async () => {
    const startOffset = 0;

    const values = [new Field(69n), new Field(420n), new Field(Field.MODULUS - 1n)];
    machineState.memory.setSlice(0, values);

    const length = values.length;

    await new SendL2ToL1Message(startOffset, length).execute(machineState, journal);

    const journalState = journal.flush();
    const expected = values.map(v => v.toFr());
    expect(journalState.newLogs).toEqual([expected]);
  });

  it('All substate instructions should fail within a static call', async () => {
    const executionEnvironment = initExecutionEnvironment({ isStaticCall: true });
    machineState = new AvmMachineState(executionEnvironment);

    const instructions = [
      new EmitNoteHash(0),
      new EmitNullifier(0),
      new EmitUnencryptedLog(0, 1),
      new SendL2ToL1Message(0, 1),
    ];

    for (const instruction of instructions) {
      const inst = () => instruction.execute(machineState, journal);
      await expect(inst()).rejects.toThrowError(StaticCallStorageAlterError);
    }
  });
});
