import { Fr } from '@aztec/foundation/fields';

import { MockProxy, mock } from 'jest-mock-extended';

import { AvmMachineState } from '../avm_machine_state.js';
import { initExecutionEnvironment, initGlobalVariables } from '../fixtures/index.js';
import { AvmJournal } from '../journal/journal.js';
import {
  Address,
  BlockNumber,
  ChainId,
  FeePerDAGas,
  FeePerL1Gas,
  FeePerL2Gas,
  Origin,
  Portal,
  Sender,
  StorageAddress,
  Timestamp,
  Version,
} from './environment_getters.js';

describe('Environment getters instructions', () => {
  let machineState: AvmMachineState;
  let journal: MockProxy<AvmJournal>;

  beforeEach(async () => {
    journal = mock<AvmJournal>();
  });

  type EnvInstruction = Portal | FeePerL1Gas | FeePerL2Gas | FeePerDAGas | Origin | Sender | StorageAddress | Address;
  const envGetterTest = async (key: string, value: Fr, instruction: EnvInstruction) => {
    machineState = new AvmMachineState(initExecutionEnvironment({ [key]: value }));

    await instruction.execute(machineState, journal);
    const actual = machineState.memory.get(0).toFr();
    expect(actual).toEqual(value);
  };

  describe('Address', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        Address.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // dstOffset
      ]);
      const inst = new Address(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678);

      expect(Address.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should read address correctly', async () => {
      const address = new Fr(123456n);
      await envGetterTest('address', address, new Address(/*indirect=*/ 0, /*dstOffset=*/ 0));
    });
  });

  describe('StorageAddress', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        StorageAddress.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // dstOffset
      ]);
      const inst = new StorageAddress(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678);

      expect(StorageAddress.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should read storage address correctly', async () => {
      const address = new Fr(123456n);
      await envGetterTest('storageAddress', address, new StorageAddress(/*indirect=*/ 0, /*dstOffset=*/ 0));
    });
  });

  describe('Portal', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        Portal.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // dstOffset
      ]);
      const inst = new Portal(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678);

      expect(Portal.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should read Portal correctly', async () => {
      const portal = new Fr(123456n);
      await envGetterTest('portal', portal, new Portal(/*indirect=*/ 0, /*dstOffset=*/ 0));
    });
  });

  describe('FeePerL1Gas', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        FeePerL1Gas.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // dstOffset
      ]);
      const inst = new FeePerL1Gas(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678);

      expect(FeePerL1Gas.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should read FeePerL1Gas correctly', async () => {
      const feePerL1Gas = new Fr(123456n);
      await envGetterTest('feePerL1Gas', feePerL1Gas, new FeePerL1Gas(/*indirect=*/ 0, /*dstOffset=*/ 0));
    });
  });

  describe('FeePerL2Gas', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        FeePerL2Gas.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // dstOffset
      ]);
      const inst = new FeePerL2Gas(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678);

      expect(FeePerL2Gas.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should read FeePerL2Gas correctly', async () => {
      const feePerL2Gas = new Fr(123456n);
      await envGetterTest('feePerL2Gas', feePerL2Gas, new FeePerL2Gas(/*indirect=*/ 0, /*dstOffset=*/ 0));
    });
  });

  describe('FeePerDAGas', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        FeePerDAGas.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // dstOffset
      ]);
      const inst = new FeePerDAGas(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678);

      expect(FeePerDAGas.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should read FeePerDAGas correctly', async () => {
      const feePerDaGas = new Fr(123456n);
      await envGetterTest('feePerDaGas', feePerDaGas, new FeePerDAGas(/*indirect=*/ 0, /*dstOffset=*/ 0));
    });
  });

  describe('Origin', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        Origin.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // dstOffset
      ]);
      const inst = new Origin(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678);

      expect(Origin.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should read Origin correctly', async () => {
      const origin = new Fr(123456n);
      await envGetterTest('origin', origin, new Origin(/*indirect=*/ 0, /*dstOffset=*/ 0));
    });
  });

  describe('Sender', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        Sender.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // dstOffset
      ]);
      const inst = new Sender(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678);

      expect(Sender.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('Should read Sender correctly', async () => {
      const sender = new Fr(123456n);
      await envGetterTest('sender', sender, new Sender(/*indirect=*/ 0, /*dstOffset=*/ 0));
    });
  });

  describe('Global Variables', () => {
    type GlobalsInstruction = ChainId | Version | BlockNumber | Timestamp;
    const readGlobalVariableTest = async (key: string, value: Fr, instruction: GlobalsInstruction) => {
      const globals = initGlobalVariables({ [key]: value });
      machineState = new AvmMachineState(initExecutionEnvironment({ globals }));

      await instruction.execute(machineState, journal);
      const actual = machineState.memory.get(0).toFr();
      expect(actual).toEqual(value);
    };

    describe('chainId', () => {
      it('Should (de)serialize correctly', () => {
        const buf = Buffer.from([
          ChainId.opcode, // opcode
          0x01, // indirect
          ...Buffer.from('12345678', 'hex'), // dstOffset
        ]);
        const inst = new ChainId(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678);

        expect(ChainId.deserialize(buf)).toEqual(inst);
        expect(inst.serialize()).toEqual(buf);
      });

      it('Should read chainId', async () => {
        const chainId = new Fr(123456n);
        await readGlobalVariableTest('chainId', chainId, new ChainId(/*indirect=*/ 0, /*dstOffset=*/ 0));
      });
    });

    describe('version', () => {
      it('Should (de)serialize correctly', () => {
        const buf = Buffer.from([
          Version.opcode, // opcode
          0x01, // indirect
          ...Buffer.from('12345678', 'hex'), // dstOffset
        ]);
        const inst = new Version(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678);

        expect(Version.deserialize(buf)).toEqual(inst);
        expect(inst.serialize()).toEqual(buf);
      });

      it('Should read version', async () => {
        const version = new Fr(123456n);
        await readGlobalVariableTest('version', version, new Version(/*indirect=*/ 0, /*dstOffset=*/ 0));
      });
    });

    describe('block', () => {
      it('Should (de)serialize correctly', () => {
        const buf = Buffer.from([
          BlockNumber.opcode, // opcode
          0x01, // indirect
          ...Buffer.from('12345678', 'hex'), // dstOffset
        ]);
        const inst = new BlockNumber(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678);

        expect(BlockNumber.deserialize(buf)).toEqual(inst);
        expect(inst.serialize()).toEqual(buf);
      });

      it('Should read block number', async () => {
        const blockNumber = new Fr(123456n);
        await readGlobalVariableTest('blockNumber', blockNumber, new BlockNumber(/*indirect=*/ 0, /*dstOffset=*/ 0));
      });
    });

    describe('timestamp', () => {
      it('Should (de)serialize correctly', () => {
        const buf = Buffer.from([
          Timestamp.opcode, // opcode
          0x01, // indirect
          ...Buffer.from('12345678', 'hex'), // dstOffset
        ]);
        const inst = new Timestamp(/*indirect=*/ 0x01, /*dstOffset=*/ 0x12345678);

        expect(Timestamp.deserialize(buf)).toEqual(inst);
        expect(inst.serialize()).toEqual(buf);
      });

      it('Should read timestamp', async () => {
        const timestamp = new Fr(123456n);
        await readGlobalVariableTest('timestamp', timestamp, new Timestamp(/*indirect=*/ 0, /*dstOffset=*/ 0));
      });
    });
  });
});
