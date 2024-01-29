import { Fr } from '@aztec/foundation/fields';

import { jest } from '@jest/globals';
import { MockProxy, mock } from 'jest-mock-extended';

import { CommitmentsDB, PublicContractsDB, PublicStateDB } from '../../index.js';
import { AvmMachineState } from '../avm_machine_state.js';
import { Field } from '../avm_memory_types.js';
import { initExecutionEnvironment } from '../fixtures/index.js';
import { HostStorage } from '../journal/host_storage.js';
import { AvmJournal } from '../journal/journal.js';
import { encodeToBytecode } from './encode_to_bytecode.js';
import { Call } from './external_calls.js';
import { Opcode } from './opcodes.js';

describe('External Calls', () => {
  let machineState: AvmMachineState;
  let journal: AvmJournal;

  let contractsDb: MockProxy<PublicContractsDB>;

  beforeEach(() => {
    machineState = new AvmMachineState(initExecutionEnvironment());

    contractsDb = mock<PublicContractsDB>();

    const commitmentsDb = mock<CommitmentsDB>();
    const publicStateDb = mock<PublicStateDB>();
    const hostStorage = new HostStorage(publicStateDb, contractsDb, commitmentsDb);
    journal = new AvmJournal(hostStorage);
  });

  describe('Call', () => {
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

      machineState.memory.set(0, new Field(gas));
      machineState.memory.set(1, new Field(addr));
      machineState.memory.setSlice(2, args);

      const otherContextInstructions: [Opcode, any[]][] = [
        // Place [1,2,3] into memory
        [Opcode.CALLDATACOPY, [/*value=*/ 0, /*copySize=*/ argsSize, /*dstOffset=*/ 0]],
        // Store 1 into slot 1
        [Opcode.SSTORE, [/*slotOffset=*/ 0, /*dataOffset=*/ 0]],
        // Return [1,2] from memory
        [Opcode.RETURN, [/*retOffset=*/ 0, /*size=*/ 2]],
      ];

      const otherContextInstructionsBytecode = Buffer.concat(
        otherContextInstructions.map(([opcode, args]) => encodeToBytecode(opcode, args)),
      );
      jest
        .spyOn(journal.hostStorage.contractsDb, 'getBytecode')
        .mockReturnValue(Promise.resolve(otherContextInstructionsBytecode));

      const instruction = new Call(gasOffset, addrOffset, argsOffset, argsSize, retOffset, retSize, successOffset);
      await instruction.execute(machineState, journal);

      const successValue = machineState.memory.get(successOffset);
      expect(successValue).toEqual(new Field(1n));

      const retValue = machineState.memory.getSlice(retOffset, retSize);
      expect(retValue).toEqual([new Field(1n), new Field(2n)]);

      // Check that the storage call has been merged into the parent journal
      const { storageWrites } = journal.flush();
      expect(storageWrites.size).toEqual(1);

      const nestedContractWrites = storageWrites.get(addr.toBigInt());
      expect(nestedContractWrites).toBeDefined();

      const slotNumber = 1n;
      const expectedStoredValue = new Fr(1n);
      expect(nestedContractWrites!.get(slotNumber)).toEqual(expectedStoredValue);
    });
  });

  describe('Static Call', () => {
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

      machineState.memory.set(0, gas);
      machineState.memory.set(1, addr);
      machineState.memory.setSlice(2, args);

      const otherContextInstructions: [Opcode, any[]][] = [
        // Place [1,2,3] into memory
        [Opcode.CALLDATACOPY, [/*value=*/ 0, /*copySize=*/ argsSize, /*dstOffset=*/ 0]],
        [Opcode.SSTORE, [/*slotOffset*/ 1, /*dataOffset=*/ 0]],
      ];

      const otherContextInstructionsBytecode = Buffer.concat(
        otherContextInstructions.map(([opcode, args]) => encodeToBytecode(opcode, args)),
      );
      jest
        .spyOn(journal.hostStorage.contractsDb, 'getBytecode')
        .mockReturnValue(Promise.resolve(otherContextInstructionsBytecode));

      const instruction = new Call(gasOffset, addrOffset, argsOffset, argsSize, retOffset, retSize, successOffset);
      await instruction.execute(machineState, journal);

      // No revert has occurred, but the nested execution has failed
      const successValue = machineState.memory.get(successOffset);
      expect(successValue).toEqual(new Field(0n));
    });
  });
});
