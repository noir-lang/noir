import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import { type MockProxy, mock } from 'jest-mock-extended';

import { type AvmContext } from '../avm_context.js';
import { Field } from '../avm_memory_types.js';
import { initContext, initExecutionEnvironment } from '../fixtures/index.js';
import { type AvmPersistableStateManager } from '../journal/journal.js';
import { SLoad, SStore, StaticCallStorageAlterError } from './storage.js';

describe('Storage Instructions', () => {
  let context: AvmContext;
  let journal: MockProxy<AvmPersistableStateManager>;
  const address = AztecAddress.random();

  beforeEach(async () => {
    journal = mock<AvmPersistableStateManager>();
    context = initContext({
      persistableState: journal,
      env: initExecutionEnvironment({ address, storageAddress: address }),
    });
  });

  describe('SSTORE', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        SStore.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // srcOffset
        ...Buffer.from('a2345678', 'hex'), // size
        ...Buffer.from('3456789a', 'hex'), // slotOffset
      ]);
      const inst = new SStore(
        /*indirect=*/ 0x01,
        /*srcOffset=*/ 0x12345678,
        /*size=*/ 0xa2345678,
        /*slotOffset=*/ 0x3456789a,
      );

      expect(SStore.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Sstore should Write into storage', async () => {
      const a = new Field(1n);
      const b = new Field(2n);

      context.machineState.memory.set(0, a);
      context.machineState.memory.set(1, b);

      await new SStore(/*indirect=*/ 0, /*srcOffset=*/ 1, /*size=*/ 1, /*slotOffset=*/ 0).execute(context);

      expect(journal.writeStorage).toHaveBeenCalledWith(address, new Fr(a.toBigInt()), new Fr(b.toBigInt()));
    });

    it('Should not be able to write to storage in a static call', async () => {
      context = initContext({
        persistableState: journal,
        env: initExecutionEnvironment({ address, storageAddress: address, isStaticCall: true }),
      });

      const a = new Field(1n);
      const b = new Field(2n);

      context.machineState.memory.set(0, a);
      context.machineState.memory.set(1, b);

      const instruction = () =>
        new SStore(/*indirect=*/ 0, /*srcOffset=*/ 0, /*size=*/ 1, /*slotOffset=*/ 1).execute(context);
      await expect(instruction()).rejects.toThrow(StaticCallStorageAlterError);
    });
  });

  describe('SLOAD', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        SLoad.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // slotOffset
        ...Buffer.from('a2345678', 'hex'), // size
        ...Buffer.from('3456789a', 'hex'), // dstOffset
      ]);
      const inst = new SLoad(
        /*indirect=*/ 0x01,
        /*slotOffset=*/ 0x12345678,
        /*size=*/ 0xa2345678,
        /*dstOffset=*/ 0x3456789a,
      );

      expect(SLoad.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Sload should Read into storage', async () => {
      // Mock response
      const expectedResult = new Fr(1n);
      journal.readStorage.mockReturnValueOnce(Promise.resolve(expectedResult));

      const a = new Field(1n);
      const b = new Field(2n);

      context.machineState.memory.set(0, a);
      context.machineState.memory.set(1, b);

      await new SLoad(/*indirect=*/ 0, /*slotOffset=*/ 0, /*size=*/ 1, /*dstOffset=*/ 1).execute(context);

      expect(journal.readStorage).toHaveBeenCalledWith(address, new Fr(a.toBigInt()));

      const actual = context.machineState.memory.get(1);
      expect(actual).toEqual(new Field(expectedResult));
    });
  });
});
