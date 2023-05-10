import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { CallContext, FunctionData } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { FunctionAbi } from '@aztec/foundation/abi';
import { ChildAbi, ParentAbi, PublicTokenContractAbi } from '@aztec/noir-contracts/examples';
import { MockProxy, mock } from 'jest-mock-extended';
import { default as memdown, type MemDown } from 'memdown';
import { encodeArguments } from '../abi_coder/encoder.js';
import { NoirPoint, computeSlotForMapping, toPublicKey } from '../utils.js';
import { PublicContractsDB, PublicStateDB } from './db.js';
import { PublicExecution } from './execution.js';
import { PublicExecutor } from './executor.js';
import { toBigInt } from '@aztec/foundation/serialize';
import { keccak } from '@aztec/foundation/crypto';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

describe('ACIR public execution simulator', () => {
  let bbWasm: BarretenbergWasm;
  let publicState: MockProxy<PublicStateDB>;
  let publicContracts: MockProxy<PublicContractsDB>;
  let executor: PublicExecutor;

  beforeAll(async () => {
    bbWasm = await BarretenbergWasm.get();
  });

  beforeEach(() => {
    publicState = mock<PublicStateDB>();
    publicContracts = mock<PublicContractsDB>();

    executor = new PublicExecutor(publicState, publicContracts);
  });

  describe('PublicToken contract', () => {
    let recipientPk: Buffer;
    let recipient: NoirPoint;

    beforeAll(() => {
      recipientPk = Buffer.from('0c9ed344548e8f9ba8aa3c9f8651eaa2853130f6c1e9c050ccf198f7ea18a7ec', 'hex');

      const grumpkin = new Grumpkin(bbWasm);
      recipient = toPublicKey(recipientPk, grumpkin);
    });

    describe('mint', () => {
      it('should run the mint function', async () => {
        const contractAddress = AztecAddress.random();
        const mintAbi = PublicTokenContractAbi.functions.find(f => f.name === 'mint')!;
        const functionData = new FunctionData(Buffer.alloc(4), false, false);
        const args = encodeArguments(mintAbi, [140, recipient], false);

        const callContext = CallContext.from({
          msgSender: AztecAddress.random(),
          storageContractAddress: contractAddress,
          portalContractAddress: EthAddress.random(),
          isContractDeployment: false,
          isDelegateCall: false,
          isStaticCall: false,
        });

        publicContracts.getBytecode.mockResolvedValue(Buffer.from(mintAbi.bytecode, 'hex'));

        // Mock the old value for the recipient balance to be 20
        const previousBalance = new Fr(20n);
        publicState.storageRead.mockResolvedValue(previousBalance);

        const execution: PublicExecution = { contractAddress, functionData, args, callContext };
        const result = await executor.execute(execution);

        const expectedBalance = new Fr(160n);
        expect(result.returnValues).toEqual([expectedBalance]);

        const storageSlot = computeSlotForMapping(new Fr(1n), recipient, bbWasm);
        expect(result.contractStorageUpdateRequests).toEqual([
          { storageSlot, oldValue: previousBalance, newValue: expectedBalance },
        ]);

        expect(result.contractStorageReads).toEqual([]);
      });
    });

    describe('transfer', () => {
      let contractAddress: AztecAddress;
      let abi: FunctionAbi;
      let functionData: FunctionData;
      let args: Fr[];
      let sender: AztecAddress;
      let callContext: CallContext;
      let recipientStorageSlot: Fr;
      let senderStorageSlot: Fr;
      let execution: PublicExecution;

      beforeEach(() => {
        contractAddress = AztecAddress.random();
        abi = PublicTokenContractAbi.functions.find(f => f.name === 'transfer')!;
        functionData = new FunctionData(Buffer.alloc(4), false, false);
        args = encodeArguments(abi, [140, recipient], false);
        sender = AztecAddress.random();

        callContext = CallContext.from({
          msgSender: sender,
          storageContractAddress: contractAddress,
          portalContractAddress: EthAddress.random(),
          isContractDeployment: false,
          isDelegateCall: false,
          isStaticCall: false,
        });

        recipientStorageSlot = computeSlotForMapping(new Fr(1n), recipient, bbWasm);
        senderStorageSlot = computeSlotForMapping(new Fr(1n), Fr.fromBuffer(sender.toBuffer()), bbWasm);

        publicContracts.getBytecode.mockResolvedValue(Buffer.from(abi.bytecode, 'hex'));

        execution = { contractAddress, functionData, args, callContext };
      });

      const mockStore = (senderBalance: Fr, recipientBalance: Fr) => {
        // eslint-disable-next-line require-await
        publicState.storageRead.mockImplementation(async (_addr: AztecAddress, slot: Fr) => {
          if (slot.equals(recipientStorageSlot)) {
            return recipientBalance;
          } else if (slot.equals(senderStorageSlot)) {
            return senderBalance;
          } else {
            return Fr.ZERO;
          }
        });
      };

      it('should run the transfer function', async () => {
        const senderBalance = new Fr(200n);
        const recipientBalance = new Fr(20n);
        mockStore(senderBalance, recipientBalance);

        const result = await executor.execute(execution);

        const expectedRecipientBalance = new Fr(160n);
        const expectedSenderBalance = new Fr(60n);

        expect(result.returnValues).toEqual([expectedRecipientBalance]);

        expect(result.contractStorageUpdateRequests).toEqual([
          { storageSlot: senderStorageSlot, oldValue: senderBalance, newValue: expectedSenderBalance },
          { storageSlot: recipientStorageSlot, oldValue: recipientBalance, newValue: expectedRecipientBalance },
        ]);

        expect(result.contractStorageReads).toEqual([]);
      });

      // Contract storage reads and update requests are implemented as built-ins, which at the moment Noir does not
      // now whether they have side-effects or not, so they get run even when their code path
      // is not picked by a conditional. Once that's fixed, we should re-enable this test.
      it.skip('should run the transfer function without enough sender balance', async () => {
        const senderBalance = new Fr(10n);
        const recipientBalance = new Fr(20n);
        mockStore(senderBalance, recipientBalance);

        const result = await executor.execute(execution);

        expect(result.returnValues).toEqual([recipientBalance]);

        expect(result.contractStorageReads).toEqual([
          { storageSlot: recipientStorageSlot, value: recipientBalance },
          { storageSlot: senderStorageSlot, value: senderBalance },
        ]);

        expect(result.contractStorageUpdateRequests).toEqual([]);
      });
    });
  });

  describe('Parent/Child contracts', () => {
    it('calls the public entry point in the parent', async () => {
      const parentContractAddress = AztecAddress.random();
      const parentEntryPointFn = ParentAbi.functions.find(f => f.name === 'pubEntryPoint')!;
      const parentEntryPointFnSelector = keccak(Buffer.from(parentEntryPointFn.name)).subarray(0, 4);

      const childContractAddress = AztecAddress.random();
      const childValueFn = ChildAbi.functions.find(f => f.name === 'pubValue')!;
      const childValueFnSelector = keccak(Buffer.from(childValueFn.name)).subarray(0, 4);

      const initialValue = 3n;

      const functionData = new FunctionData(parentEntryPointFnSelector, false, false);
      const args = encodeArguments(
        parentEntryPointFn,
        [childContractAddress.toField().value, toBigInt(childValueFnSelector), initialValue],
        false,
      );

      const callContext = CallContext.from({
        msgSender: AztecAddress.random(),
        storageContractAddress: parentContractAddress,
        portalContractAddress: EthAddress.random(),
        isContractDeployment: false,
        isDelegateCall: false,
        isStaticCall: false,
      });

      // eslint-disable-next-line require-await
      publicContracts.getBytecode.mockImplementation(async (addr: AztecAddress, selector: Buffer) => {
        if (addr.equals(parentContractAddress) && selector.equals(parentEntryPointFnSelector)) {
          return Buffer.from(parentEntryPointFn.bytecode, 'hex');
        } else if (addr.equals(childContractAddress) && selector.equals(childValueFnSelector)) {
          return Buffer.from(childValueFn.bytecode, 'hex');
        } else {
          return undefined;
        }
      });

      const execution: PublicExecution = { contractAddress: parentContractAddress, functionData, args, callContext };
      const result = await executor.execute(execution);

      expect(result.returnValues).toEqual([new Fr(42n + initialValue)]);
    });
  });
});
