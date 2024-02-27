import { L1ToL2Message, SiblingPath } from '@aztec/circuit-types';
import {
  AppendOnlyTreeSnapshot,
  CallContext,
  FunctionData,
  GlobalVariables,
  Header,
  L1_TO_L2_MSG_TREE_HEIGHT,
  L2ToL1Message,
} from '@aztec/circuits.js';
import { makeHeader } from '@aztec/circuits.js/testing';
import { FunctionArtifact, FunctionSelector, encodeArguments } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { pedersenHash } from '@aztec/foundation/crypto';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { ChildContractArtifact } from '@aztec/noir-contracts.js/Child';
import { ParentContractArtifact } from '@aztec/noir-contracts.js/Parent';
import { TestContractArtifact } from '@aztec/noir-contracts.js/Test';
import { TokenContractArtifact } from '@aztec/noir-contracts.js/Token';

import { MockProxy, mock } from 'jest-mock-extended';
import { type MemDown, default as memdown } from 'memdown';
import { getFunctionSelector } from 'viem';

import { MessageLoadOracleInputs } from '../index.js';
import { buildL1ToL2Message } from '../test/utils.js';
import { computeSlotForMapping } from '../utils.js';
import { CommitmentsDB, PublicContractsDB, PublicStateDB } from './db.js';
import { PublicExecution } from './execution.js';
import { PublicExecutor } from './executor.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

describe('ACIR public execution simulator', () => {
  let publicState: MockProxy<PublicStateDB>;
  let publicContracts: MockProxy<PublicContractsDB>;
  let commitmentsDb: MockProxy<CommitmentsDB>;
  let executor: PublicExecutor;
  let header: Header;

  beforeEach(() => {
    publicState = mock<PublicStateDB>();
    publicContracts = mock<PublicContractsDB>();
    commitmentsDb = mock<CommitmentsDB>();

    const randomInt = Math.floor(Math.random() * 1000000);
    header = makeHeader(randomInt);

    executor = new PublicExecutor(publicState, publicContracts, commitmentsDb, header);
  }, 10000);

  describe('Token contract', () => {
    let recipient: AztecAddress;

    beforeEach(() => {
      recipient = AztecAddress.random();
    });

    describe('mint', () => {
      it('should run the mint_public function', async () => {
        const contractAddress = AztecAddress.random();
        const mintArtifact = TokenContractArtifact.functions.find(f => f.name === 'mint_public')!;
        const functionData = FunctionData.fromAbi(mintArtifact);

        const mintAmount = 140n;
        const args = encodeArguments(mintArtifact, [recipient, mintAmount]);

        const msgSender = AztecAddress.random();
        const callContext = CallContext.from({
          msgSender,
          storageContractAddress: contractAddress,
          portalContractAddress: EthAddress.random(),
          functionSelector: FunctionSelector.empty(),
          isContractDeployment: false,
          isDelegateCall: false,
          isStaticCall: false,
          startSideEffectCounter: 0,
        });

        publicContracts.getBytecode.mockResolvedValue(Buffer.from(mintArtifact.bytecode, 'base64'));

        // Mock the old value for the recipient balance to be 20
        const isMinter = new Fr(1n); // 1n means true
        const previousBalance = new Fr(20n);
        const previousTotalSupply = new Fr(previousBalance.value + 100n);
        publicState.storageRead
          .mockResolvedValueOnce(isMinter) // reading whether msg_sender is minter
          .mockResolvedValueOnce(previousBalance) // reading user's balance
          .mockResolvedValueOnce(previousTotalSupply); // reading total supply

        const execution: PublicExecution = { contractAddress, functionData, args, callContext };
        const result = await executor.simulate(execution, GlobalVariables.empty());

        const recipientBalanceStorageSlot = computeSlotForMapping(new Fr(6n), recipient);
        const totalSupplyStorageSlot = new Fr(4n);

        const expectedBalance = new Fr(previousBalance.value + mintAmount);
        const expectedTotalSupply = new Fr(previousTotalSupply.value + mintAmount);
        // There should be 2 storage updates, one for the recipient's balance and one for the total supply
        expect(result.contractStorageUpdateRequests).toEqual([
          {
            storageSlot: recipientBalanceStorageSlot,
            newValue: expectedBalance,
            sideEffectCounter: 3,
          },
          {
            storageSlot: totalSupplyStorageSlot,
            newValue: expectedTotalSupply,
            sideEffectCounter: 4,
          },
        ]);

        const mintersStorageSlot = new Fr(2n);
        const isMinterStorageSlot = computeSlotForMapping(mintersStorageSlot, msgSender);
        // Note: There is only 1 storage read (for the isMinter value) because the other 2 reads get overwritten by
        // the updates
        expect(result.contractStorageReads).toEqual([
          {
            storageSlot: isMinterStorageSlot,
            currentValue: isMinter,
            sideEffectCounter: 0,
          },
        ]);
      });
    });

    describe('transfer', () => {
      let contractAddress: AztecAddress;
      let transferArtifact: FunctionArtifact;
      let functionData: FunctionData;
      let args: Fr[];
      let sender: AztecAddress;
      let callContext: CallContext;
      let recipientStorageSlot: Fr;
      let senderStorageSlot: Fr;
      let execution: PublicExecution;

      beforeEach(() => {
        contractAddress = AztecAddress.random();
        transferArtifact = TokenContractArtifact.functions.find(f => f.name === 'transfer_public')!;
        functionData = new FunctionData(FunctionSelector.empty(), false, false, false);
        sender = AztecAddress.random();
        args = encodeArguments(transferArtifact, [sender, recipient, 140n, 0n]);

        callContext = CallContext.from({
          msgSender: sender,
          storageContractAddress: contractAddress,
          portalContractAddress: EthAddress.random(),
          functionSelector: FunctionSelector.empty(),
          isContractDeployment: false,
          isDelegateCall: false,
          isStaticCall: false,
          startSideEffectCounter: 0,
        });

        recipientStorageSlot = computeSlotForMapping(new Fr(6n), recipient);
        senderStorageSlot = computeSlotForMapping(new Fr(6n), sender);

        publicContracts.getBytecode.mockResolvedValue(Buffer.from(transferArtifact.bytecode, 'base64'));

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

        expect(result.contractStorageUpdateRequests).toEqual([
          {
            storageSlot: senderStorageSlot,
            newValue: expectedSenderBalance,
            sideEffectCounter: 1, // 1 read (sender balance)
          },
          {
            storageSlot: recipientStorageSlot,
            newValue: expectedRecipientBalance,
            sideEffectCounter: 3, // 1 read (sender balance), 1 write (new sender balance), 1 read (recipient balance)
          },
        ]);

        expect(result.contractStorageReads).toEqual([]);
      });

      it('should throw underflow error when executing transfer function without enough sender balance', async () => {
        const senderBalance = new Fr(10n);
        const recipientBalance = new Fr(20n);
        mockStore(senderBalance, recipientBalance);

        await expect(executor.simulate(execution, GlobalVariables.empty())).rejects.toThrowError(
          'Assertion failed: attempt to subtract with underflow',
        );
      });
    });
  });

  describe('Parent/Child contracts', () => {
    it.each([false, true, undefined])(
      'calls the public entry point in the parent',
      async isInternal => {
        const parentContractAddress = AztecAddress.random();
        const parentEntryPointFn = ParentContractArtifact.functions.find(f => f.name === 'pubEntryPoint')!;
        const parentEntryPointFnSelector = FunctionSelector.fromNameAndParameters(
          parentEntryPointFn.name,
          parentEntryPointFn.parameters,
        );

        const childContractAddress = AztecAddress.random();
        const childValueFn = ChildContractArtifact.functions.find(f => f.name === 'pubGetValue')!;
        const childValueFnSelector = FunctionSelector.fromNameAndParameters(childValueFn.name, childValueFn.parameters);

        const initialValue = 3n;

        const functionData = new FunctionData(parentEntryPointFnSelector, isInternal ?? false, false, false);
        const args = encodeArguments(parentEntryPointFn, [childContractAddress, childValueFnSelector, initialValue]);

        const callContext = CallContext.from({
          msgSender: AztecAddress.random(),
          storageContractAddress: parentContractAddress,
          portalContractAddress: EthAddress.random(),
          functionSelector: FunctionSelector.empty(),
          isContractDeployment: false,
          isDelegateCall: false,
          isStaticCall: false,
          startSideEffectCounter: 0,
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
        const globalVariables = new GlobalVariables(
          new Fr(69),
          new Fr(420),
          new Fr(1),
          new Fr(7),
          EthAddress.fromField(new Fr(8)),
          AztecAddress.fromField(new Fr(9)),
        );

        if (isInternal === undefined) {
          await expect(executor.simulate(execution, globalVariables)).rejects.toThrowError(/Method not found -/);
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

    beforeEach(() => {
      contractAddress = AztecAddress.random();
      functionData = new FunctionData(FunctionSelector.empty(), false, false, false);
      amount = new Fr(1);
      params = [amount, new Fr(1)];
    });

    it('Should be able to create a note hash from the public context', async () => {
      const shieldArtifact = TokenContractArtifact.functions.find(f => f.name === 'shield')!;
      const msgSender = AztecAddress.random();
      const secretHash = Fr.random();

      const args = encodeArguments(shieldArtifact, [msgSender, amount, secretHash, Fr.ZERO]);

      const callContext = CallContext.from({
        msgSender: msgSender,
        storageContractAddress: contractAddress,
        portalContractAddress: EthAddress.random(),
        functionSelector: FunctionSelector.empty(),
        isContractDeployment: false,
        isDelegateCall: false,
        isStaticCall: false,
        startSideEffectCounter: 0,
      });

      publicContracts.getBytecode.mockResolvedValue(Buffer.from(shieldArtifact.bytecode, 'base64'));
      // mock initial balance to be greater than the amount being sent
      publicState.storageRead.mockResolvedValue(amount);

      const execution: PublicExecution = { contractAddress, functionData, args, callContext };
      const result = await executor.simulate(execution, GlobalVariables.empty());

      // Assert the note hash was created
      expect(result.newNoteHashes.length).toEqual(1);

      const expectedNoteHash = pedersenHash([amount.toBuffer(), secretHash.toBuffer()]);
      const storageSlot = new Fr(5); // for pending_shields
      const expectedInnerNoteHash = pedersenHash([storageSlot, expectedNoteHash].map(f => f.toBuffer()));
      expect(result.newNoteHashes[0].value).toEqual(expectedInnerNoteHash);
    });

    it('Should be able to create a L2 to L1 message from the public context', async () => {
      const createL2ToL1MessagePublicArtifact = TestContractArtifact.functions.find(
        f => f.name === 'create_l2_to_l1_message_public',
      )!;
      const args = encodeArguments(createL2ToL1MessagePublicArtifact, params);

      const portalContractAddress = EthAddress.random();

      const callContext = CallContext.from({
        msgSender: AztecAddress.random(),
        storageContractAddress: contractAddress,
        portalContractAddress,
        functionSelector: FunctionSelector.empty(),
        isContractDeployment: false,
        isDelegateCall: false,
        isStaticCall: false,
        startSideEffectCounter: 0,
      });

      publicContracts.getBytecode.mockResolvedValue(Buffer.from(createL2ToL1MessagePublicArtifact.bytecode, 'base64'));

      const execution: PublicExecution = { contractAddress, functionData, args, callContext };
      const result = await executor.simulate(execution, GlobalVariables.empty());

      // Assert the l2 to l1 message was created
      expect(result.newL2ToL1Messages.length).toEqual(1);

      const expectedNewMessage = new L2ToL1Message(portalContractAddress, pedersenHash(params.map(a => a.toBuffer())));

      expect(result.newL2ToL1Messages[0]).toEqual(expectedNewMessage);
    });

    it('Should be able to create a nullifier from the public context', async () => {
      const createNullifierPublicArtifact = TestContractArtifact.functions.find(
        f => f.name === 'create_nullifier_public',
      )!;

      const args = encodeArguments(createNullifierPublicArtifact, params);

      const callContext = CallContext.from({
        msgSender: AztecAddress.random(),
        storageContractAddress: contractAddress,
        portalContractAddress: EthAddress.random(),
        functionSelector: FunctionSelector.empty(),
        isContractDeployment: false,
        isDelegateCall: false,
        isStaticCall: false,
        startSideEffectCounter: 0,
      });

      publicContracts.getBytecode.mockResolvedValue(Buffer.from(createNullifierPublicArtifact.bytecode, 'base64'));

      const execution: PublicExecution = { contractAddress, functionData, args, callContext };
      const result = await executor.simulate(execution, GlobalVariables.empty());

      // Assert the l2 to l1 message was created
      expect(result.newNullifiers.length).toEqual(1);

      const expectedNewMessageValue = pedersenHash(params.map(a => a.toBuffer()));
      expect(result.newNullifiers[0].value).toEqual(expectedNewMessageValue);
    });

    describe('L1 to L2 messages', () => {
      const mintPublicArtifact = TestContractArtifact.functions.find(f => f.name === 'consume_mint_public_message')!;

      const canceller = EthAddress.random();
      const tokenRecipient = AztecAddress.random();
      let bridgedAmount = 20n;
      let secret = new Fr(1);

      let crossChainMsgRecipient: AztecAddress | undefined;
      let crossChainMsgSender: EthAddress | undefined;
      let messageKey: Fr | undefined;

      let preimage: L1ToL2Message;
      let globalVariables: GlobalVariables;

      let args: Fr[];
      let callContext: CallContext;

      beforeEach(() => {
        bridgedAmount = 20n;
        secret = new Fr(1);

        crossChainMsgRecipient = undefined;
        crossChainMsgSender = undefined;
        messageKey = undefined;
      });

      const computePreImage = () =>
        buildL1ToL2Message(
          getFunctionSelector('mint_public(bytes32,uint256,address)').substring(2),
          [tokenRecipient.toField(), new Fr(bridgedAmount), canceller.toField()],
          crossChainMsgRecipient ?? contractAddress,
          secret,
        );

      const computeArgs = () =>
        encodeArguments(mintPublicArtifact, [
          tokenRecipient,
          bridgedAmount,
          canceller,
          messageKey ?? preimage.hash(),
          secret,
        ]);

      const computeCallContext = () =>
        CallContext.from({
          msgSender: AztecAddress.random(),
          storageContractAddress: contractAddress,
          portalContractAddress: crossChainMsgSender ?? preimage.sender.sender,
          functionSelector: FunctionSelector.empty(),
          isContractDeployment: false,
          isDelegateCall: false,
          isStaticCall: false,
          startSideEffectCounter: 0,
        });

      const computeGlobalVariables = () =>
        new GlobalVariables(
          new Fr(preimage.sender.chainId),
          new Fr(preimage.recipient.version),
          Fr.ZERO,
          Fr.ZERO,
          EthAddress.ZERO,
          AztecAddress.ZERO,
        );

      const mockOracles = () => {
        publicContracts.getBytecode.mockResolvedValue(Buffer.from(mintPublicArtifact.bytecode, 'base64'));
        publicState.storageRead.mockResolvedValue(Fr.ZERO);

        const siblingPathBuffers = Array(L1_TO_L2_MSG_TREE_HEIGHT)
          .fill(Fr.random())
          .map(f => f.toBuffer());
        const siblingPath = new SiblingPath(L1_TO_L2_MSG_TREE_HEIGHT, siblingPathBuffers);

        let root = messageKey ?? preimage.hash();
        for (const sibling of siblingPathBuffers) {
          root = pedersenHash([root.toBuffer(), sibling]);
        }
        commitmentsDb.getL1ToL2Message.mockImplementation(() => {
          return Promise.resolve(new MessageLoadOracleInputs(preimage, 0n, siblingPath));
        });

        return new AppendOnlyTreeSnapshot(
          root,
          1, // we set 1 message in the tree
        );
      };

      it('Should be able to consume an L1 to L2 message in the public context', async () => {
        preimage = computePreImage();

        args = computeArgs();
        callContext = computeCallContext();

        // Prepare the state
        header.state.l1ToL2MessageTree = mockOracles();
        globalVariables = computeGlobalVariables();

        const execution: PublicExecution = { contractAddress, functionData, args, callContext };
        executor = new PublicExecutor(publicState, publicContracts, commitmentsDb, header);
        const result = await executor.simulate(execution, globalVariables);
        expect(result.newNullifiers.length).toEqual(1);
      });

      it('Message not matching requested key', async () => {
        // Using a random value for the message key
        messageKey = Fr.random();

        preimage = computePreImage();
        args = computeArgs();
        callContext = computeCallContext();

        // Prepare the state
        header.state.l1ToL2MessageTree = mockOracles();
        globalVariables = computeGlobalVariables();

        const execution: PublicExecution = { contractAddress, functionData, args, callContext };
        executor = new PublicExecutor(publicState, publicContracts, commitmentsDb, header);
        await expect(executor.simulate(execution, globalVariables)).rejects.toThrowError(
          'Message not matching requested key',
        );
      });

      it('Invalid membership proof', async () => {
        preimage = computePreImage();
        args = computeArgs();
        callContext = computeCallContext();

        // Mock oracles but don't update state
        mockOracles();

        // Prepare the state
        globalVariables = computeGlobalVariables();

        const execution: PublicExecution = { contractAddress, functionData, args, callContext };
        executor = new PublicExecutor(publicState, publicContracts, commitmentsDb, header);
        await expect(executor.simulate(execution, globalVariables)).rejects.toThrowError('Message not in state');
      });

      it('Invalid recipient', async () => {
        crossChainMsgRecipient = AztecAddress.random();
        preimage = computePreImage();
        args = computeArgs();
        callContext = computeCallContext();

        // Prepare the state
        header.state.l1ToL2MessageTree = mockOracles();
        globalVariables = computeGlobalVariables();

        const execution: PublicExecution = { contractAddress, functionData, args, callContext };
        executor = new PublicExecutor(publicState, publicContracts, commitmentsDb, header);
        await expect(executor.simulate(execution, globalVariables)).rejects.toThrowError('Invalid recipient');
      });

      it('Invalid sender', async () => {
        crossChainMsgSender = EthAddress.random();
        preimage = computePreImage();
        args = computeArgs();
        callContext = computeCallContext();

        // Prepare the state
        header.state.l1ToL2MessageTree = mockOracles();
        globalVariables = computeGlobalVariables();

        const execution: PublicExecution = { contractAddress, functionData, args, callContext };
        executor = new PublicExecutor(publicState, publicContracts, commitmentsDb, header);
        await expect(executor.simulate(execution, globalVariables)).rejects.toThrowError('Invalid sender');
      });

      it('Invalid chainid', async () => {
        preimage = computePreImage();
        args = computeArgs();
        callContext = computeCallContext();

        // Prepare the state
        header.state.l1ToL2MessageTree = mockOracles();
        globalVariables = computeGlobalVariables();
        globalVariables.chainId = Fr.random();

        const execution: PublicExecution = { contractAddress, functionData, args, callContext };
        executor = new PublicExecutor(publicState, publicContracts, commitmentsDb, header);
        await expect(executor.simulate(execution, globalVariables)).rejects.toThrowError('Invalid Chainid');
      });

      it('Invalid version', async () => {
        preimage = computePreImage();
        args = computeArgs();
        callContext = computeCallContext();

        // Prepare the state
        header.state.l1ToL2MessageTree = mockOracles();
        globalVariables = computeGlobalVariables();
        globalVariables.version = Fr.random();

        const execution: PublicExecution = { contractAddress, functionData, args, callContext };
        executor = new PublicExecutor(publicState, publicContracts, commitmentsDb, header);
        await expect(executor.simulate(execution, globalVariables)).rejects.toThrowError('Invalid Version');
      });

      it('Invalid Content', async () => {
        preimage = computePreImage();

        bridgedAmount = bridgedAmount + 1n; // Invalid amount
        args = computeArgs();
        callContext = computeCallContext();

        // Prepare the state
        header.state.l1ToL2MessageTree = mockOracles();
        globalVariables = computeGlobalVariables();

        const execution: PublicExecution = { contractAddress, functionData, args, callContext };
        executor = new PublicExecutor(publicState, publicContracts, commitmentsDb, header);
        await expect(executor.simulate(execution, globalVariables)).rejects.toThrowError('Invalid Content');
      });

      it('Invalid secret', async () => {
        preimage = computePreImage();

        secret = Fr.random(); // Invalid secret
        args = computeArgs();
        callContext = computeCallContext();

        // Prepare the state
        header.state.l1ToL2MessageTree = mockOracles();
        globalVariables = computeGlobalVariables();

        const execution: PublicExecution = { contractAddress, functionData, args, callContext };
        executor = new PublicExecutor(publicState, publicContracts, commitmentsDb, header);
        await expect(executor.simulate(execution, globalVariables)).rejects.toThrowError('Invalid message secret');
      });
    });
  });

  describe('Global variables in public context', () => {
    let contractAddress: AztecAddress;
    let callContext: CallContext;
    let assertGlobalVarsArtifact: FunctionArtifact;
    let functionData: FunctionData;

    const modifyGlobalVariables = (globalVariables: GlobalVariables, propertyIndex: number, value: any) => {
      const globalVariablesFields = GlobalVariables.getFields(globalVariables) as unknown as any[];
      globalVariablesFields[propertyIndex] = value;
      return GlobalVariables.fromFields(globalVariablesFields);
    };

    beforeAll(() => {
      contractAddress = AztecAddress.random();
      callContext = CallContext.from({
        msgSender: AztecAddress.random(),
        storageContractAddress: AztecAddress.random(),
        portalContractAddress: EthAddress.ZERO,
        functionSelector: FunctionSelector.empty(),
        isContractDeployment: false,
        isDelegateCall: false,
        isStaticCall: false,
        startSideEffectCounter: 0,
      });
      assertGlobalVarsArtifact = TestContractArtifact.functions.find(f => f.name === 'assert_public_global_vars')!;
      functionData = FunctionData.fromAbi(assertGlobalVarsArtifact);
    });

    beforeEach(() => {
      publicContracts.getBytecode.mockResolvedValue(Buffer.from(assertGlobalVarsArtifact.bytecode, 'base64'));
    });

    // Note: Order here has to match the order of the properties in GlobalVariables.getFields(...) function.
    const testCases = [
      { value: new Fr(1), invalidValue: Fr.random(), description: 'Chain ID' },
      { value: new Fr(1), invalidValue: Fr.random(), description: 'Version' },
      { value: new Fr(1), invalidValue: Fr.random(), description: 'Block number' },
      { value: new Fr(1), invalidValue: Fr.random(), description: 'Timestamp' },
      { value: EthAddress.random(), invalidValue: EthAddress.random(), description: 'Coinbase' },
      {
        value: AztecAddress.random(),
        invalidValue: AztecAddress.random(),
        description: 'Fee recipient',
      },
    ];

    testCases.forEach(({ value, invalidValue, description }, i: number) => {
      describe(`${description}`, () => {
        let globalVariables: GlobalVariables;

        beforeAll(() => {
          // We create a new global variables object containing non-zero value in place of the tested property
          globalVariables = modifyGlobalVariables(GlobalVariables.empty(), i, value);
        });

        it('Valid', () => {
          let args: Fr[];
          {
            // We create the args by just serializing the reference global variables object
            const rawArgs = GlobalVariables.getFields(globalVariables) as unknown as any[];
            args = encodeArguments(assertGlobalVarsArtifact, rawArgs);
          }

          const execution: PublicExecution = { contractAddress, functionData, args, callContext };
          executor = new PublicExecutor(publicState, publicContracts, commitmentsDb, header);

          expect(() => executor.simulate(execution, globalVariables)).not.toThrow();
        });

        it('Invalid', async () => {
          let args: Fr[];
          {
            // We create the args by modifying the global variables object to contain an invalid value in place of
            // the tested property
            const modifiedGlobalVariables = modifyGlobalVariables(globalVariables, i, invalidValue);
            const rawArgs = GlobalVariables.getFields(modifiedGlobalVariables) as unknown as any[];
            args = encodeArguments(assertGlobalVarsArtifact, rawArgs);
          }

          const execution: PublicExecution = { contractAddress, functionData, args, callContext };
          executor = new PublicExecutor(publicState, publicContracts, commitmentsDb, header);

          await expect(executor.simulate(execution, globalVariables)).rejects.toThrowError(
            `Invalid ${description.toLowerCase()}`,
          );
        });
      });
    });
  });

  describe('Historical header in public context', () => {
    let contractAddress: AztecAddress;
    let callContext: CallContext;
    let assertHeaderPublicArtifact: FunctionArtifact;
    let functionData: FunctionData;

    beforeAll(() => {
      contractAddress = AztecAddress.random();
      callContext = CallContext.from({
        msgSender: AztecAddress.random(),
        storageContractAddress: AztecAddress.random(),
        portalContractAddress: EthAddress.ZERO,
        functionSelector: FunctionSelector.empty(),
        isContractDeployment: false,
        isDelegateCall: false,
        isStaticCall: false,
        startSideEffectCounter: 0,
      });
      assertHeaderPublicArtifact = TestContractArtifact.functions.find(f => f.name === 'assert_header_public')!;
      functionData = FunctionData.fromAbi(assertHeaderPublicArtifact);
    });

    beforeEach(() => {
      publicContracts.getBytecode.mockResolvedValue(Buffer.from(assertHeaderPublicArtifact.bytecode, 'base64'));
    });

    it('Header is correctly set', () => {
      const args = encodeArguments(assertHeaderPublicArtifact, [header.hash()]);

      const execution: PublicExecution = { contractAddress, functionData, args, callContext };
      executor = new PublicExecutor(publicState, publicContracts, commitmentsDb, header);

      expect(() => executor.simulate(execution, GlobalVariables.empty())).not.toThrow();
    });

    it('Throws when header is not as expected', async () => {
      const unexpectedHeaderHash = Fr.random();
      const args = encodeArguments(assertHeaderPublicArtifact, [unexpectedHeaderHash]);

      const execution: PublicExecution = { contractAddress, functionData, args, callContext };
      executor = new PublicExecutor(publicState, publicContracts, commitmentsDb, header);

      await expect(executor.simulate(execution, GlobalVariables.empty())).rejects.toThrowError('Invalid header hash');
    });
  });
});
