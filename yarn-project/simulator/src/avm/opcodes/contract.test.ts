import { AztecAddress, EthAddress, Fr } from '@aztec/circuits.js';

import { type DeepMockProxy, mockDeep } from 'jest-mock-extended';

import { type AvmContext } from '../avm_context.js';
import { Field } from '../avm_memory_types.js';
import { initContext } from '../fixtures/index.js';
import { type AvmPersistableStateManager } from '../journal/journal.js';
import { GetContractInstance } from './contract.js';

describe('Contract opcodes', () => {
  let context: AvmContext;
  let journal: DeepMockProxy<AvmPersistableStateManager>;
  const address = AztecAddress.random();

  beforeEach(async () => {
    journal = mockDeep<AvmPersistableStateManager>();
    context = initContext({
      persistableState: journal,
    });
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
      context.machineState.memory.set(0, new Field(address.toField()));

      const contractInstance = {
        address: address,
        version: 1 as const,
        salt: new Fr(20),
        contractClassId: new Fr(30),
        initializationHash: new Fr(40),
        portalContractAddress: EthAddress.random(),
        publicKeysHash: new Fr(50),
        deployer: AztecAddress.random(),
      };

      journal.hostStorage.contractsDb.getContractInstance.mockReturnValue(Promise.resolve(contractInstance));

      await new GetContractInstance(/*indirect=*/ 0, /*addressOffset=*/ 0, /*dstOffset=*/ 1).execute(context);

      const actual = context.machineState.memory.getSlice(1, 7);
      expect(actual).toEqual([
        new Field(1), // found
        new Field(contractInstance.salt),
        new Field(contractInstance.deployer),
        new Field(contractInstance.contractClassId),
        new Field(contractInstance.initializationHash),
        new Field(contractInstance.portalContractAddress.toField()),
        new Field(contractInstance.publicKeysHash),
      ]);
    });

    it('should return zeroes if not found', async () => {
      context.machineState.memory.set(0, new Field(address.toField()));
      journal.hostStorage.contractsDb.getContractInstance.mockReturnValue(Promise.resolve(undefined));

      await new GetContractInstance(/*indirect=*/ 0, /*addressOffset=*/ 0, /*dstOffset=*/ 1).execute(context);

      const actual = context.machineState.memory.getSlice(1, 7);
      expect(actual).toEqual([
        new Field(0), // found
        new Field(0),
        new Field(0),
        new Field(0),
        new Field(0),
        new Field(0),
        new Field(0),
      ]);
    });
  });
});
