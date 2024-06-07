import { AztecAddress, Fr } from '@aztec/circuits.js';
import { type ContractInstanceWithAddress } from '@aztec/types/contracts';

import { mock } from 'jest-mock-extended';

import { type PublicContractsDB } from '../../public/db_interfaces.js';
import { type AvmContext } from '../avm_context.js';
import { Field } from '../avm_memory_types.js';
import { initContext, initHostStorage } from '../fixtures/index.js';
import { AvmPersistableStateManager } from '../journal/journal.js';
import { GetContractInstance } from './contract.js';

describe('Contract opcodes', () => {
  let context: AvmContext;
  const address = AztecAddress.random();

  beforeEach(async () => {
    context = initContext();
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
        publicKeysHash: new Fr(50),
        deployer: AztecAddress.random(),
      } as ContractInstanceWithAddress;

      const contractsDb = mock<PublicContractsDB>();
      contractsDb.getContractInstance.mockResolvedValue(Promise.resolve(contractInstance));
      context.persistableState = new AvmPersistableStateManager(initHostStorage({ contractsDb }));

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
    });

    it('should return zeroes if not found', async () => {
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
    });
  });
});
