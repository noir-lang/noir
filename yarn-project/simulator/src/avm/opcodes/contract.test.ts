import { randomContractInstanceWithAddress } from '@aztec/circuit-types';
import { AztecAddress } from '@aztec/circuits.js';
import { SerializableContractInstance } from '@aztec/types/contracts';

import { mock } from 'jest-mock-extended';

import { type PublicSideEffectTraceInterface } from '../../public/side_effect_trace_interface.js';
import { type AvmContext } from '../avm_context.js';
import { Field } from '../avm_memory_types.js';
import { initContext, initHostStorage, initPersistableStateManager } from '../fixtures/index.js';
import { type HostStorage } from '../journal/host_storage.js';
import { type AvmPersistableStateManager } from '../journal/journal.js';
import { mockGetContractInstance } from '../test_utils.js';
import { GetContractInstance } from './contract.js';

describe('Contract opcodes', () => {
  const address = AztecAddress.random();

  let hostStorage: HostStorage;
  let trace: PublicSideEffectTraceInterface;
  let persistableState: AvmPersistableStateManager;
  let context: AvmContext;

  beforeEach(() => {
    hostStorage = initHostStorage();
    trace = mock<PublicSideEffectTraceInterface>();
    persistableState = initPersistableStateManager({ hostStorage, trace });
    context = initContext({ persistableState });
  });

  describe('GETCONTRACTINSTANCE', () => {
    it('Should (de)serialize correctly', () => {
      const buf = Buffer.from([
        GetContractInstance.opcode, // opcode
        0x01, // indirect
        ...Buffer.from('12345678', 'hex'), // addressOffset
        ...Buffer.from('a2345678', 'hex'), // dstOffset
      ]);
      const inst = new GetContractInstance(
        /*indirect=*/ 0x01,
        /*addressOffset=*/ 0x12345678,
        /*dstOffset=*/ 0xa2345678,
      );

      expect(GetContractInstance.deserialize(buf)).toEqual(inst);
      expect(inst.serialize()).toEqual(buf);
    });

    it('should copy contract instance to memory if found', async () => {
      const contractInstance = randomContractInstanceWithAddress(/*(base instance) opts=*/ {}, /*address=*/ address);
      mockGetContractInstance(hostStorage, contractInstance);

      context.machineState.memory.set(0, new Field(address.toField()));
      await new GetContractInstance(/*indirect=*/ 0, /*addressOffset=*/ 0, /*dstOffset=*/ 1).execute(context);

      const actual = context.machineState.memory.getSlice(1, 6);
      expect(actual).toEqual([
        new Field(1), // found
        new Field(contractInstance.salt),
        new Field(contractInstance.deployer),
        new Field(contractInstance.contractClassId),
        new Field(contractInstance.initializationHash),
        new Field(contractInstance.publicKeysHash),
      ]);

      expect(trace.traceGetContractInstance).toHaveBeenCalledTimes(1);
      expect(trace.traceGetContractInstance).toHaveBeenCalledWith({ exists: true, ...contractInstance });
    });

    it('should return zeroes if not found', async () => {
      const emptyContractInstance = SerializableContractInstance.empty().withAddress(address);
      context.machineState.memory.set(0, new Field(address.toField()));

      await new GetContractInstance(/*indirect=*/ 0, /*addressOffset=*/ 0, /*dstOffset=*/ 1).execute(context);

      const actual = context.machineState.memory.getSlice(1, 6);
      expect(actual).toEqual([
        new Field(0), // found
        new Field(0),
        new Field(0),
        new Field(0),
        new Field(0),
        new Field(0),
      ]);

      expect(trace.traceGetContractInstance).toHaveBeenCalledTimes(1);
      expect(trace.traceGetContractInstance).toHaveBeenCalledWith({ exists: false, ...emptyContractInstance });
    });
  });
});
