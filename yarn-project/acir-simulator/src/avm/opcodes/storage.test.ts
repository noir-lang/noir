import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import { MockProxy, mock } from 'jest-mock-extended';

import { AvmMachineState } from '../avm_machine_state.js';
import { Field } from '../avm_memory_types.js';
import { initExecutionEnvironment } from '../fixtures/index.js';
import { AvmJournal } from '../journal/journal.js';
import { SLoad, SStore, StaticCallStorageAlterError } from './storage.js';

describe('Storage Instructions', () => {
  let journal: MockProxy<AvmJournal>;
  let machineState: AvmMachineState;
  const address = AztecAddress.random();

  beforeEach(() => {
    journal = mock<AvmJournal>();

    const executionEnvironment = initExecutionEnvironment({ address, storageAddress: address });
    machineState = new AvmMachineState(executionEnvironment);
  });

  describe('SSTORE', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        SStore.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // srcOffset
        ...Buffer.from('a2345678', 'hex'), // slotOffset
      ]);
      const inst = new SStore(/*indirect=*/ 0x01, /*srcOffset=*/ 0x12345678, /*slotOffset=*/ 0xa2345678);

      expect(SStore.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Sstore should Write into storage', async () => {
      const a = new Field(1n);
      const b = new Field(2n);

      machineState.memory.set(0, a);
      machineState.memory.set(1, b);

      await new SStore(/*indirect=*/ 0, /*srcOffset=*/ 0, /*slotOffset=*/ 1).execute(machineState, journal);

      expect(journal.writeStorage).toHaveBeenCalledWith(address, new Fr(a.toBigInt()), new Fr(b.toBigInt()));
    });

    it('Should not be able to write to storage in a static call', async () => {
      const executionEnvironment = initExecutionEnvironment({ isStaticCall: true });
      machineState = new AvmMachineState(executionEnvironment);

      const a = new Field(1n);
      const b = new Field(2n);

      machineState.memory.set(0, a);
      machineState.memory.set(1, b);

      const instruction = () =>
        new SStore(/*indirect=*/ 0, /*srcOffset=*/ 0, /*slotOffset=*/ 1).execute(machineState, journal);
      await expect(instruction()).rejects.toThrow(StaticCallStorageAlterError);
    });
  });

  describe('SLOAD', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        SLoad.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // slotOffset
        ...Buffer.from('a2345678', 'hex'), // dstOffset
      ]);
      const inst = new SLoad(/*indirect=*/ 0x01, /*slotOffset=*/ 0x12345678, /*dstOffset=*/ 0xa2345678);

      expect(SLoad.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Sload should Read into storage', async () => {
      // Mock response
      const expectedResult = new Fr(1n);
      journal.readStorage.mockReturnValueOnce(Promise.resolve(expectedResult));

      const a = new Field(1n);
      const b = new Field(2n);

      machineState.memory.set(0, a);
      machineState.memory.set(1, b);

      await new SLoad(/*indirect=*/ 0, /*slotOffset=*/ 0, /*dstOffset=*/ 1).execute(machineState, journal);

      expect(journal.readStorage).toHaveBeenCalledWith(address, new Fr(a.toBigInt()));

      const actual = machineState.memory.get(1);
      expect(actual).toEqual(new Field(expectedResult));
    });
  });
});
