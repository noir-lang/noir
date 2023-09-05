import {
  CallContext,
  CircuitsWasm,
  ContractStorageRead,
  FunctionData,
  GlobalVariables,
  HistoricBlockData,
  L1_TO_L2_MSG_TREE_HEIGHT,
} from '@aztec/circuits.js';
import { pedersenPlookupCommitInputs } from '@aztec/circuits.js/barretenberg';
import { FunctionAbi, FunctionSelector, encodeArguments } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { toBigInt } from '@aztec/foundation/serialize';
import {
  ChildContractAbi,
  NonNativeTokenContractAbi,
  ParentContractAbi,
  PublicTokenContractAbi,
  TestContractAbi,
} from '@aztec/noir-contracts/artifacts';

import { MockProxy, mock } from 'jest-mock-extended';
import { type MemDown, default as memdown } from 'memdown';

import { buildL1ToL2Message } from '../test/utils.js';
import { computeSlotForMapping } from '../utils.js';
import { CommitmentsDB, PublicContractsDB, PublicStateDB } from './db.js';
import { PublicExecution } from './execution.js';
import { PublicExecutor } from './executor.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

describe('ACIR public execution simulator', () => {
  let circuitsWasm: CircuitsWasm;
  let publicState: MockProxy<PublicStateDB>;
  let publicContracts: MockProxy<PublicContractsDB>;
  let commitmentsDb: MockProxy<CommitmentsDB>;
  let executor: PublicExecutor;
  let blockData: HistoricBlockData;

  beforeAll(async () => {
    circuitsWasm = await CircuitsWasm.get();
  });

  beforeEach(() => {
    publicState = mock<PublicStateDB>();
    publicContracts = mock<PublicContractsDB>();
    commitmentsDb = mock<CommitmentsDB>();

    blockData = HistoricBlockData.empty();
    executor = new PublicExecutor(publicState, publicContracts, commitmentsDb, blockData);
  }, 10000);

  describe('PublicToken contract', () => {
    let recipient: AztecAddress;

    beforeEach(() => {
      recipient = AztecAddress.random();
    });

    describe('mint', () => {
      it('should run the mint function', async () => {
        const contractAddress = AztecAddress.random();
        const mintAbi = PublicTokenContractAbi.functions.find(f => f.name === 'mint')!;
        const functionData = FunctionData.fromAbi(mintAbi);
        const args = encodeArguments(mintAbi, [140, recipient]);

        const callContext = CallContext.from({
          msgSender: AztecAddress.random(),
          storageContractAddress: contractAddress,
          portalContractAddress: EthAddress.random(),
          isContractDeployment: false,
          isDelegateCall: false,
          isStaticCall: false,
        });

        publicContracts.getBytecode.mockResolvedValue(Buffer.from(mintAbi.bytecode, 'base64'));

        // Mock the old value for the recipient balance to be 20
        const previousBalance = new Fr(20n);
        publicState.storageRead.mockResolvedValue(previousBalance);

        const execution: PublicExecution = { contractAddress, functionData, args, callContext };
        const result = await executor.simulate(execution, GlobalVariables.empty());

        const expectedBalance = new Fr(160n);
        expect(result.returnValues[0]).toEqual(expectedBalance);

        const storageSlot = computeSlotForMapping(new Fr(1n), recipient.toField(), circuitsWasm);
        expect(result.contractStorageUpdateRequests).toEqual([
          { storageSlot, oldValue: previousBalance, newValue: expectedBalance, sideEffectCounter: 1 }, // 0th is a read
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
        functionData = new FunctionData(FunctionSelector.empty(), false, false, false);
        args = encodeArguments(abi, [140, recipient]);
        sender = AztecAddress.random();

        callContext = CallContext.from({
          msgSender: sender,
          storageContractAddress: contractAddress,
          portalContractAddress: EthAddress.random(),
          isContractDeployment: false,
          isDelegateCall: false,
          isStaticCall: false,
        });

        recipientStorageSlot = computeSlotForMapping(new Fr(1n), recipient.toField(), circuitsWasm);
        senderStorageSlot = computeSlotForMapping(new Fr(1n), Fr.fromBuffer(sender.toBuffer()), circuitsWasm);

        publicContracts.getBytecode.mockResolvedValue(Buffer.from(abi.bytecode, 'base64'));

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

        const result = await executor.simulate(execution, GlobalVariables.empty());

        const expectedRecipientBalance = new Fr(160n);
        const expectedSenderBalance = new Fr(60n);

        expect(result.returnValues[0]).toEqual(expectedRecipientBalance);

        expect(result.contractStorageUpdateRequests).toEqual([
          {
            storageSlot: senderStorageSlot,
            oldValue: senderBalance,
            newValue: expectedSenderBalance,
            sideEffectCounter: 2,
          }, // 0th, 1st are reads
          {
            storageSlot: recipientStorageSlot,
            oldValue: recipientBalance,
            newValue: expectedRecipientBalance,
            sideEffectCounter: 3,
          },
        ]);

        expect(result.contractStorageReads).toEqual([]);
      });

      it('should fail the transfer function without enough sender balance', async () => {
        const senderBalance = new Fr(10n);
        const recipientBalance = new Fr(20n);
        mockStore(senderBalance, recipientBalance);

        const result = await executor.simulate(execution, GlobalVariables.empty());
        expect(result.returnValues[0]).toEqual(recipientBalance);

        expect(result.contractStorageReads).toEqual(
          [
            { storageSlot: senderStorageSlot, currentValue: senderBalance, sideEffectCounter: 0 },
            { storageSlot: recipientStorageSlot, currentValue: recipientBalance, sideEffectCounter: 1 },
          ].map(ContractStorageRead.from),
        );

        expect(result.contractStorageUpdateRequests).toEqual([]);
      });
    });
  });

  describe('Parent/Child contracts', () => {
    it.each([false, true, undefined])(
      'calls the public entry point in the parent',
      async isInternal => {
        const parentContractAddress = AztecAddress.random();
        const parentEntryPointFn = ParentContractAbi.functions.find(f => f.name === 'pubEntryPoint')!;
        const parentEntryPointFnSelector = FunctionSelector.fromNameAndParameters(
          parentEntryPointFn.name,
          parentEntryPointFn.parameters,
        );

        const childContractAddress = AztecAddress.random();
        const childValueFn = ChildContractAbi.functions.find(f => f.name === 'pubGetValue')!;
        const childValueFnSelector = FunctionSelector.fromNameAndParameters(childValueFn.name, childValueFn.parameters);

        const initialValue = 3n;

        const functionData = new FunctionData(parentEntryPointFnSelector, isInternal ?? false, false, false);
        const args = encodeArguments(parentEntryPointFn, [
          childContractAddress.toField().value,
          toBigInt(childValueFnSelector.toBuffer()),
          initialValue,
        ]);

        const callContext = CallContext.from({
          msgSender: AztecAddress.random(),
          storageContractAddress: parentContractAddress,
          portalContractAddress: EthAddress.random(),
          isContractDeployment: false,
          isDelegateCall: false,
          isStaticCall: false,
        });

        // eslint-disable-next-line require-await
        publicContracts.getBytecode.mockImplementation(async (addr: AztecAddress, selector: FunctionSelector) => {
          if (addr.equals(parentContractAddress) && selector.equals(parentEntryPointFnSelector)) {
            return Buffer.from(parentEntryPointFn.bytecode, 'base64');
          } else if (addr.equals(childContractAddress) && selector.equals(childValueFnSelector)) {
            return Buffer.from(childValueFn.bytecode, 'base64');
          } else {
            return undefined;
          }
        });

        publicContracts.getIsInternal.mockImplementation(() => {
          return Promise.resolve(isInternal);
        });

        const execution: PublicExecution = { contractAddress: parentContractAddress, functionData, args, callContext };
        const globalVariables = new GlobalVariables(new Fr(69), new Fr(420), new Fr(1), new Fr(7));

        if (isInternal === undefined) {
          await expect(executor.simulate(execution, globalVariables)).rejects.toThrowError(
            /ContractsDb don't contain isInternal for/,
          );
        } else {
          const result = await executor.simulate(execution, globalVariables);

          expect(result.returnValues[0]).toEqual(
            new Fr(
              initialValue +
                globalVariables.chainId.value +
                globalVariables.version.value +
                globalVariables.blockNumber.value +
                globalVariables.timestamp.value,
            ),
          );
        }
      },
      20_000,
    );
  });

  describe('Public -> Private / Cross Chain messaging', () => {
    let contractAddress: AztecAddress;
    let functionData: FunctionData;
    let amount: Fr;
    let params: Fr[];
    let wasm: CircuitsWasm;

    beforeEach(async () => {
      contractAddress = AztecAddress.random();
      functionData = new FunctionData(FunctionSelector.empty(), false, false, false);
      amount = new Fr(140);
      params = [amount, Fr.random()];
      wasm = await CircuitsWasm.get();
    });

    it('Should be able to create a commitment from the public context', async () => {
      const shieldAbi = NonNativeTokenContractAbi.functions.find(f => f.name === 'shield')!;
      const args = encodeArguments(shieldAbi, params);

      const callContext = CallContext.from({
        msgSender: AztecAddress.random(),
        storageContractAddress: contractAddress,
        portalContractAddress: EthAddress.random(),
        isContractDeployment: false,
        isDelegateCall: false,
        isStaticCall: false,
      });

      publicContracts.getBytecode.mockResolvedValue(Buffer.from(shieldAbi.bytecode, 'base64'));
      // mock initial balance to be greater than the amount being sent
      publicState.storageRead.mockResolvedValue(amount);

      const execution: PublicExecution = { contractAddress, functionData, args, callContext };
      const result = await executor.simulate(execution, GlobalVariables.empty());

      // Assert the commitment was created
      expect(result.newCommitments.length).toEqual(1);

      const expectedNoteHash = pedersenPlookupCommitInputs(
        wasm,
        params.map(a => a.toBuffer()),
      );
      const storageSlot = new Fr(2); // matches storage.nr
      const expectedInnerNoteHash = pedersenPlookupCommitInputs(wasm, [storageSlot.toBuffer(), expectedNoteHash]);
      expect(result.newCommitments[0].toBuffer()).toEqual(expectedInnerNoteHash);
    });

    it('Should be able to create a L2 to L1 message from the public context', async () => {
      const createL2ToL1MessagePublicAbi = TestContractAbi.functions.find(f => f.name === 'createL2ToL1MessagePublic')!;
      const args = encodeArguments(createL2ToL1MessagePublicAbi, params);

      const callContext = CallContext.from({
        msgSender: AztecAddress.random(),
        storageContractAddress: contractAddress,
        portalContractAddress: EthAddress.random(),
        isContractDeployment: false,
        isDelegateCall: false,
        isStaticCall: false,
      });

      publicContracts.getBytecode.mockResolvedValue(Buffer.from(createL2ToL1MessagePublicAbi.bytecode, 'base64'));

      const execution: PublicExecution = { contractAddress, functionData, args, callContext };
      const result = await executor.simulate(execution, GlobalVariables.empty());

      // Assert the l2 to l1 message was created
      expect(result.newL2ToL1Messages.length).toEqual(1);

      const expectedNewMessageValue = pedersenPlookupCommitInputs(
        wasm,
        params.map(a => a.toBuffer()),
      );
      expect(result.newL2ToL1Messages[0].toBuffer()).toEqual(expectedNewMessageValue);
    });

    it('Should be able to consume an Ll to L2 message in the public context', async () => {
      const mintPublicAbi = NonNativeTokenContractAbi.functions.find(f => f.name === 'mintPublic')!;

      // Set up cross chain message
      const canceller = EthAddress.random();

      const bridgedAmount = 20n;
      const secret = new Fr(1n);
      const recipient = AztecAddress.random();

      // Function selector: 0xeeb73071 keccak256('mint(uint256,bytes32,address)')
      const preimage = await buildL1ToL2Message(
        'eeb73071',
        [new Fr(bridgedAmount), recipient.toField(), canceller.toField()],
        contractAddress,
        secret,
      );

      // Stub message key
      const messageKey = Fr.random();
      const args = encodeArguments(mintPublicAbi, [
        bridgedAmount,
        recipient.toField(),
        messageKey,
        secret,
        canceller.toField(),
      ]);

      const callContext = CallContext.from({
        msgSender: AztecAddress.random(),
        storageContractAddress: contractAddress,
        portalContractAddress: EthAddress.random(),
        isContractDeployment: false,
        isDelegateCall: false,
        isStaticCall: false,
      });

      publicContracts.getBytecode.mockResolvedValue(Buffer.from(mintPublicAbi.bytecode, 'base64'));
      publicState.storageRead.mockResolvedValue(Fr.ZERO);

      // Mock response
      commitmentsDb.getL1ToL2Message.mockImplementation(async () => {
        return await Promise.resolve({
          message: preimage.toFieldArray(),
          index: 0n,
          siblingPath: Array(L1_TO_L2_MSG_TREE_HEIGHT).fill(Fr.random()),
        });
      });

      const execution: PublicExecution = { contractAddress, functionData, args, callContext };
      const result = await executor.simulate(execution, GlobalVariables.empty());

      expect(result.newNullifiers.length).toEqual(1);
    });

    it('Should be able to create a nullifier from the public context', async () => {
      const createNullifierPublicAbi = TestContractAbi.functions.find(f => f.name === 'createNullifierPublic')!;

      const args = encodeArguments(createNullifierPublicAbi, params);

      const callContext = CallContext.from({
        msgSender: AztecAddress.random(),
        storageContractAddress: contractAddress,
        portalContractAddress: EthAddress.random(),
        isContractDeployment: false,
        isDelegateCall: false,
        isStaticCall: false,
      });

      publicContracts.getBytecode.mockResolvedValue(Buffer.from(createNullifierPublicAbi.bytecode, 'base64'));

      const execution: PublicExecution = { contractAddress, functionData, args, callContext };
      const result = await executor.simulate(execution, GlobalVariables.empty());

      // Assert the l2 to l1 message was created
      expect(result.newNullifiers.length).toEqual(1);

      const expectedNewMessageValue = pedersenPlookupCommitInputs(
        wasm,
        params.map(a => a.toBuffer()),
      );
      expect(result.newNullifiers[0].toBuffer()).toEqual(expectedNewMessageValue);
    });
  });
});
