import {
  CallContext,
  CircuitsWasm,
  ContractDeploymentData,
  FunctionData,
  L1_TO_L2_MSG_TREE_HEIGHT,
  MAX_NEW_COMMITMENTS_PER_CALL,
  PRIVATE_DATA_TREE_HEIGHT,
  PrivateHistoricTreeRoots,
  PublicCallRequest,
  TxContext,
} from '@aztec/circuits.js';
import {
  computeCommitmentNonce,
  computeContractAddressFromPartial,
  computeSecretMessageHash,
  computeTxHash,
  computeUniqueCommitment,
  siloCommitment,
} from '@aztec/circuits.js/abis';
import { Grumpkin, pedersenPlookupCommitInputs } from '@aztec/circuits.js/barretenberg';
import { FunctionAbi, encodeArguments, generateFunctionSelector } from '@aztec/foundation/abi';
import { asyncMap } from '@aztec/foundation/async-map';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { AppendOnlyTree, Pedersen, StandardTree, newTree } from '@aztec/merkle-tree';
import {
  ChildContractAbi,
  NonNativeTokenContractAbi,
  ParentContractAbi,
  PendingCommitmentsContractAbi,
  TestContractAbi,
  ZkTokenContractAbi,
} from '@aztec/noir-contracts/examples';
import { PackedArguments, TxExecutionRequest } from '@aztec/types';

import { jest } from '@jest/globals';
import { MockProxy, mock } from 'jest-mock-extended';
import { default as levelup } from 'levelup';
import { type MemDown, default as memdown } from 'memdown';

import { buildL1ToL2Message } from '../test/utils.js';
import { NoirPoint, computeSlotForMapping, toPublicKey } from '../utils.js';
import { DBOracle } from './db_oracle.js';
import { AcirSimulator } from './simulator.js';

jest.setTimeout(60_000);

const createMemDown = () => (memdown as any)() as MemDown<any, any>;

describe('Private Execution test suite', () => {
  let circuitsWasm: CircuitsWasm;
  let oracle: MockProxy<DBOracle>;
  let acirSimulator: AcirSimulator;
  let txNullifier: Fr;
  let historicRoots = PrivateHistoricTreeRoots.empty();
  let logger: DebugLogger;

  const defaultContractAddress = AztecAddress.random();
  const ownerPk = Buffer.from('5e30a2f886b4b6a11aea03bf4910fbd5b24e61aa27ea4d05c393b3ab592a8d33', 'hex');

  const treeHeights: { [name: string]: number } = {
    privateData: PRIVATE_DATA_TREE_HEIGHT,
    l1ToL2Messages: L1_TO_L2_MSG_TREE_HEIGHT,
  };
  const trees: { [name: keyof typeof treeHeights]: AppendOnlyTree } = {};
  const txContext = new TxContext(false, false, false, ContractDeploymentData.empty(), new Fr(69), new Fr(420));

  const runSimulator = async ({
    abi,
    args = [],
    origin = AztecAddress.random(),
    contractAddress = defaultContractAddress,
    isConstructor = false,
  }: {
    abi: FunctionAbi;
    origin?: AztecAddress;
    contractAddress?: AztecAddress;
    isConstructor?: boolean;
    args?: any[];
  }) => {
    const packedArguments = await PackedArguments.fromArgs(encodeArguments(abi, args), circuitsWasm);
    const txRequest = TxExecutionRequest.from({
      origin,
      argsHash: packedArguments.hash,
      functionData: new FunctionData(Buffer.alloc(4), true, isConstructor),
      txContext,
      packedArguments: [packedArguments],
    });

    txNullifier = computeTxHash(circuitsWasm, txRequest.toTxRequest());

    return acirSimulator.run(
      txRequest,
      abi,
      isConstructor ? AztecAddress.ZERO : contractAddress,
      EthAddress.ZERO,
      historicRoots,
    );
  };

  const insertLeaves = async (leaves: Fr[], name = 'privateData') => {
    if (!treeHeights[name]) {
      throw new Error(`Unknown tree ${name}`);
    }
    if (!trees[name]) {
      const db = levelup(createMemDown());
      const pedersen = new Pedersen(circuitsWasm);
      trees[name] = await newTree(StandardTree, db, pedersen, name, treeHeights[name]);
    }
    await trees[name].appendLeaves(leaves.map(l => l.toBuffer()));

    // Update root.
    const newRoot = trees[name].getRoot(false);
    const prevRoots = historicRoots.toBuffer();
    const rootIndex = name === 'privateData' ? 0 : 32 * 3;
    const newRoots = Buffer.concat([prevRoots.slice(0, rootIndex), newRoot, prevRoots.slice(rootIndex + 32)]);
    historicRoots = PrivateHistoricTreeRoots.fromBuffer(newRoots);

    return trees[name];
  };

  const hash = (data: Buffer[]) => pedersenPlookupCommitInputs(circuitsWasm, data);

  beforeAll(async () => {
    circuitsWasm = await CircuitsWasm.get();
    logger = createDebugLogger('aztec:test:private_execution');
  });

  beforeEach(() => {
    oracle = mock<DBOracle>();
    oracle.getSecretKey.mockResolvedValue(ownerPk);

    acirSimulator = new AcirSimulator(oracle);
  });

  describe('empty constructor', () => {
    it('should run the empty constructor', async () => {
      const abi = TestContractAbi.functions[0];
      const result = await runSimulator({ abi, isConstructor: true });
      expect(result.callStackItem.publicInputs.newCommitments).toEqual(
        new Array(MAX_NEW_COMMITMENTS_PER_CALL).fill(Fr.ZERO),
      );
    });
  });

  describe('zk token contract', () => {
    const contractAddress = defaultContractAddress;
    const recipientPk = Buffer.from('0c9ed344548e8f9ba8aa3c9f8651eaa2853130f6c1e9c050ccf198f7ea18a7ec', 'hex');
    let owner: NoirPoint;
    let recipient: NoirPoint;
    let currentNoteIndex = 0n;

    const buildNote = (amount: bigint, owner: NoirPoint) => {
      const nonce = new Fr(currentNoteIndex);
      const preimage = [new Fr(amount), new Fr(owner.x), new Fr(owner.y), Fr.random(), new Fr(1n)];
      return { index: currentNoteIndex++, nonce, preimage };
    };

    beforeAll(() => {
      const grumpkin = new Grumpkin(circuitsWasm);
      owner = toPublicKey(ownerPk, grumpkin);
      recipient = toPublicKey(recipientPk, grumpkin);
    });

    beforeEach(() => {
      oracle.getFunctionABI.mockImplementation((_, selector) =>
        Promise.resolve(
          ZkTokenContractAbi.functions.find(f => selector.equals(generateFunctionSelector(f.name, f.parameters)))!,
        ),
      );
    });

    it('should have an abi for computing note hash and nullifier', async () => {
      const storageSlot = Fr.random();
      const note = buildNote(60n, owner);

      // Should be the same as how we compute the values for the ValueNote in the noir library.
      const valueNoteHash = pedersenPlookupCommitInputs(
        circuitsWasm,
        note.preimage.map(f => f.toBuffer()),
      );
      const innerNoteHash = Fr.fromBuffer(
        pedersenPlookupCommitInputs(circuitsWasm, [storageSlot.toBuffer(), valueNoteHash]),
      );
      const uniqueNoteHash = computeUniqueCommitment(circuitsWasm, note.nonce, innerNoteHash);
      const siloedNoteHash = siloCommitment(circuitsWasm, contractAddress, uniqueNoteHash);
      const nullifier = Fr.fromBuffer(pedersenPlookupCommitInputs(circuitsWasm, [siloedNoteHash.toBuffer(), ownerPk]));

      const result = await acirSimulator.computeNoteHashAndNullifier(
        contractAddress,
        note.nonce,
        storageSlot,
        note.preimage,
      );

      expect(result).toEqual({
        innerNoteHash,
        uniqueNoteHash,
        siloedNoteHash,
        nullifier,
      });
    });

    it('should a constructor with arguments that creates notes', async () => {
      const abi = ZkTokenContractAbi.functions.find(f => f.name === 'constructor')!;

      const result = await runSimulator({ args: [140, owner], abi, isConstructor: true });

      expect(result.preimages.newNotes).toHaveLength(1);
      const newNote = result.preimages.newNotes[0];
      expect(newNote.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner, circuitsWasm));

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));
      expect(newCommitments).toHaveLength(1);

      const [commitment] = newCommitments;
      expect(commitment).toEqual(
        await acirSimulator.computeInnerNoteHash(contractAddress, newNote.storageSlot, newNote.preimage),
      );
    });

    it('should run the mint function', async () => {
      const abi = ZkTokenContractAbi.functions.find(f => f.name === 'mint')!;

      const result = await runSimulator({ args: [140, owner], abi });

      expect(result.preimages.newNotes).toHaveLength(1);
      const newNote = result.preimages.newNotes[0];
      expect(newNote.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner, circuitsWasm));

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));
      expect(newCommitments).toHaveLength(1);

      const [commitment] = newCommitments;
      expect(commitment).toEqual(
        await acirSimulator.computeInnerNoteHash(contractAddress, newNote.storageSlot, newNote.preimage),
      );
    });

    it('should run the transfer function', async () => {
      const amountToTransfer = 100n;
      const abi = ZkTokenContractAbi.functions.find(f => f.name === 'transfer')!;

      const storageSlot = computeSlotForMapping(new Fr(1n), owner, circuitsWasm);

      const notes = [buildNote(60n, owner), buildNote(80n, owner)];
      oracle.getNotes.mockResolvedValue(notes);

      const consumedNotes = await asyncMap(notes, ({ nonce, preimage }) =>
        acirSimulator.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, preimage),
      );
      await insertLeaves(consumedNotes.map(n => n.siloedNoteHash));

      const args = [amountToTransfer, owner, recipient];
      const result = await runSimulator({ args, abi });

      // The two notes were nullified
      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toEqual(consumedNotes.map(n => n.nullifier));

      expect(result.preimages.newNotes).toHaveLength(2);
      const [changeNote, recipientNote] = result.preimages.newNotes;
      expect(recipientNote.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), recipient, circuitsWasm));

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));
      expect(newCommitments).toHaveLength(2);

      const [changeNoteCommitment, recipientNoteCommitment] = newCommitments;
      const recipientStorageSlot = computeSlotForMapping(new Fr(1n), recipient, circuitsWasm);
      expect(recipientNoteCommitment).toEqual(
        await acirSimulator.computeInnerNoteHash(contractAddress, recipientStorageSlot, recipientNote.preimage),
      );
      expect(changeNoteCommitment).toEqual(
        await acirSimulator.computeInnerNoteHash(contractAddress, storageSlot, changeNote.preimage),
      );

      expect(recipientNote.preimage[0]).toEqual(new Fr(amountToTransfer));
      expect(changeNote.preimage[0]).toEqual(new Fr(40n));

      const readRequests = result.callStackItem.publicInputs.readRequests.filter(field => !field.equals(Fr.ZERO));
      expect(readRequests).toEqual(consumedNotes.map(n => n.uniqueNoteHash));
    });

    it('should be able to transfer with dummy notes', async () => {
      const amountToTransfer = 100n;
      const balance = 160n;
      const abi = ZkTokenContractAbi.functions.find(f => f.name === 'transfer')!;

      const storageSlot = computeSlotForMapping(new Fr(1n), owner, circuitsWasm);

      const notes = [buildNote(balance, owner)];
      oracle.getNotes.mockResolvedValue(notes);

      const consumedNotes = await asyncMap(notes, ({ nonce, preimage }) =>
        acirSimulator.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, preimage),
      );
      await insertLeaves(consumedNotes.map(n => n.siloedNoteHash));

      const args = [amountToTransfer, owner, recipient];
      const result = await runSimulator({ args, abi });

      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toEqual(consumedNotes.map(n => n.nullifier));

      expect(result.preimages.newNotes).toHaveLength(2);
      const [changeNote, recipientNote] = result.preimages.newNotes;
      expect(recipientNote.preimage[0]).toEqual(new Fr(amountToTransfer));
      expect(changeNote.preimage[0]).toEqual(new Fr(balance - amountToTransfer));
    });

    it('Should be able to claim a note by providing the correct secret', async () => {
      const contractAddress = AztecAddress.random();
      const amount = 100n;
      const secret = Fr.random();
      const abi = ZkTokenContractAbi.functions.find(f => f.name === 'claim')!;

      oracle.getNotes.mockResolvedValue([
        {
          nonce: Fr.ZERO,
          preimage: [new Fr(amount), secret],
          index: BigInt(1),
        },
      ]);

      const storageSlot = 2n;
      const customNoteHash = hash([toBufferBE(amount, 32), secret.toBuffer()]);
      const innerNoteHash = Fr.fromBuffer(hash([toBufferBE(storageSlot, 32), customNoteHash]));

      const result = await runSimulator({
        origin: contractAddress,
        abi,
        args: [amount, secret, recipient],
      });

      // Check a nullifier has been created.
      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toHaveLength(1);

      // Check the read request was created successfully.
      const readRequests = result.callStackItem.publicInputs.readRequests.filter(field => !field.equals(Fr.ZERO));
      const nonce = Fr.ZERO;
      const uniqueNoteHash = computeUniqueCommitment(circuitsWasm, nonce, innerNoteHash);
      expect(readRequests).toEqual([uniqueNoteHash]);
    });
  });

  describe('nested calls', () => {
    const privateIncrement = txContext.chainId.value + txContext.version.value;
    it('child function should be callable', async () => {
      const initialValue = 100n;
      const abi = ChildContractAbi.functions.find(f => f.name === 'value')!;
      const result = await runSimulator({ args: [initialValue], abi });

      expect(result.callStackItem.publicInputs.returnValues[0]).toEqual(new Fr(initialValue + privateIncrement));
    });

    it('parent should call child', async () => {
      const childAbi = ChildContractAbi.functions.find(f => f.name === 'value')!;
      const parentAbi = ParentContractAbi.functions.find(f => f.name === 'entryPoint')!;
      const parentAddress = AztecAddress.random();
      const childAddress = AztecAddress.random();
      const childSelector = Buffer.alloc(4, 1); // should match the call

      oracle.getFunctionABI.mockImplementation(() => Promise.resolve(childAbi));
      oracle.getPortalContractAddress.mockImplementation(() => Promise.resolve(EthAddress.ZERO));

      logger(`Parent deployed at ${parentAddress.toShortString()}`);
      logger(`Calling child function ${childSelector.toString('hex')} at ${childAddress.toShortString()}`);

      const args = [Fr.fromBuffer(childAddress.toBuffer()), Fr.fromBuffer(childSelector)];
      const result = await runSimulator({ args, abi: parentAbi, origin: parentAddress });

      expect(result.callStackItem.publicInputs.returnValues[0]).toEqual(new Fr(privateIncrement));
      expect(oracle.getFunctionABI.mock.calls[0]).toEqual([childAddress, childSelector]);
      expect(oracle.getPortalContractAddress.mock.calls[0]).toEqual([childAddress]);
      expect(result.nestedExecutions).toHaveLength(1);
      expect(result.nestedExecutions[0].callStackItem.publicInputs.returnValues[0]).toEqual(new Fr(privateIncrement));
    });
  });

  describe('Consuming Messages', () => {
    const contractAddress = defaultContractAddress;
    const recipientPk = Buffer.from('0c9ed344548e8f9ba8aa3c9f8651eaa2853130f6c1e9c050ccf198f7ea18a7ec', 'hex');
    let recipient: NoirPoint;

    beforeAll(() => {
      const grumpkin = new Grumpkin(circuitsWasm);
      recipient = toPublicKey(recipientPk, grumpkin);
    });

    it('Should be able to consume a dummy cross chain message', async () => {
      const bridgedAmount = 100n;
      const abi = NonNativeTokenContractAbi.functions.find(f => f.name === 'mint')!;

      const secret = new Fr(1n);
      const canceller = EthAddress.random();
      // Function selector: 0xeeb73071 keccak256('mint(uint256,bytes32,address)')
      const preimage = await buildL1ToL2Message(
        'eeb73071',
        [new Fr(bridgedAmount), new Fr(recipient.x), canceller.toField()],
        contractAddress,
        secret,
      );

      // stub message key
      const messageKey = Fr.random();
      const tree = await insertLeaves([messageKey], 'l1ToL2Messages');

      oracle.getL1ToL2Message.mockImplementation(async () => {
        return Promise.resolve({
          message: preimage.toFieldArray(),
          index: 0n,
          siblingPath: (await tree.getSiblingPath(0n, false)).toFieldArray(),
        });
      });

      const args = [bridgedAmount, recipient, recipient.x, messageKey, secret, canceller.toField()];
      const result = await runSimulator({ origin: contractAddress, contractAddress, abi, args });

      // Check a nullifier has been created
      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toHaveLength(1);
    });

    it('Should be able to consume a dummy public to private message', async () => {
      const amount = 100n;
      const abi = NonNativeTokenContractAbi.functions.find(f => f.name === 'redeemShield')!;

      const wasm = await CircuitsWasm.get();
      const secret = new Fr(1n);
      const secretHash = computeSecretMessageHash(wasm, secret);
      const commitment = Fr.fromBuffer(hash([toBufferBE(amount, 32), secretHash.toBuffer()]));
      const siloedCommitment = siloCommitment(wasm, contractAddress, commitment);

      const tree = await insertLeaves([siloedCommitment]);

      oracle.getCommitmentOracle.mockImplementation(async () => {
        // Check the calculated commitment is correct
        return Promise.resolve({
          commitment: siloedCommitment,
          index: 0n,
          siblingPath: (await tree.getSiblingPath(0n, false)).toFieldArray(),
        });
      });

      const result = await runSimulator({
        origin: contractAddress,
        abi,
        args: [amount, secret, recipient],
      });

      // Check a nullifier has been created.
      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toHaveLength(1);

      // Check the commitment read request was created successfully.
      const readRequests = result.callStackItem.publicInputs.readRequests.filter(field => !field.equals(Fr.ZERO));
      expect(readRequests).toHaveLength(1);
      expect(readRequests[0]).toEqual(commitment);
    });
  });

  describe('enqueued calls', () => {
    it('parent should enqueue call to child', async () => {
      const parentAbi = ParentContractAbi.functions.find(f => f.name === 'enqueueCallToChild')!;
      const childAddress = AztecAddress.random();
      const childPortalContractAddress = EthAddress.random();
      const childSelector = Buffer.alloc(4, 1); // should match the call
      const parentAddress = AztecAddress.random();

      oracle.getPortalContractAddress.mockImplementation(() => Promise.resolve(childPortalContractAddress));

      const args = [Fr.fromBuffer(childAddress.toBuffer()), Fr.fromBuffer(childSelector), 42n];
      const result = await runSimulator({
        origin: parentAddress,
        contractAddress: parentAddress,
        abi: parentAbi,
        args,
      });

      expect(result.enqueuedPublicFunctionCalls).toHaveLength(1);
      expect(result.enqueuedPublicFunctionCalls[0]).toEqual(
        PublicCallRequest.from({
          contractAddress: childAddress,
          functionData: new FunctionData(childSelector, false, false),
          args: [new Fr(42n)],
          callContext: CallContext.from({
            msgSender: parentAddress,
            storageContractAddress: childAddress,
            portalContractAddress: childPortalContractAddress,
            isContractDeployment: false,
            isDelegateCall: false,
            isStaticCall: false,
          }),
        }),
      );
    });
  });

  describe('pending commitments contract', () => {
    let owner: NoirPoint;

    beforeAll(() => {
      const grumpkin = new Grumpkin(circuitsWasm);
      owner = toPublicKey(ownerPk, grumpkin);
    });

    beforeEach(() => {
      oracle.getFunctionABI.mockImplementation((_, selector) =>
        Promise.resolve(
          PendingCommitmentsContractAbi.functions.find(f =>
            selector.equals(generateFunctionSelector(f.name, f.parameters)),
          )!,
        ),
      );
    });

    it('should be able to read pending commitments created in same function', async () => {
      oracle.getNotes.mockResolvedValue([]);

      const amountToTransfer = 100n;

      const contractAddress = AztecAddress.random();
      const abi = PendingCommitmentsContractAbi.functions.find(f => f.name === 'test_insert_then_read_flat')!;

      const args = [amountToTransfer, owner];
      const result = await runSimulator({
        args: args,
        abi: abi,
        origin: contractAddress,
        contractAddress: contractAddress,
      });

      expect(result.preimages.newNotes).toHaveLength(1);
      const note = result.preimages.newNotes[0];
      expect(note.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner, circuitsWasm));

      expect(note.preimage[0]).toEqual(new Fr(amountToTransfer));

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));

      expect(newCommitments).toHaveLength(1);

      const commitment = newCommitments[0];
      const storageSlot = computeSlotForMapping(new Fr(1n), owner, circuitsWasm);
      expect(commitment).toEqual(await acirSimulator.computeInnerNoteHash(contractAddress, storageSlot, note.preimage));
      // read request should match commitment
      const nonce = computeCommitmentNonce(circuitsWasm, txNullifier, 0);
      const readRequest = result.callStackItem.publicInputs.readRequests[0];
      expect(readRequest).toEqual(computeUniqueCommitment(circuitsWasm, nonce, commitment));

      const gotNoteValue = result.callStackItem.publicInputs.returnValues[0].value;
      expect(gotNoteValue).toEqual(amountToTransfer);
    });

    it('should be able to create and read pending commitments both in nested calls', async () => {
      oracle.getNotes.mockResolvedValue([]);

      const amountToTransfer = 100n;

      const contractAddress = AztecAddress.random();
      const abi = PendingCommitmentsContractAbi.functions.find(
        f => f.name === 'test_insert_then_read_both_in_nested_calls',
      )!;
      const createAbi = PendingCommitmentsContractAbi.functions.find(f => f.name === 'create_note')!;
      const getAndCheckAbi = PendingCommitmentsContractAbi.functions.find(f => f.name === 'get_and_check_note')!;

      const createFnSelector = generateFunctionSelector(createAbi.name, createAbi.parameters);
      const getAndCheckFnSelector = generateFunctionSelector(getAndCheckAbi.name, getAndCheckAbi.parameters);

      oracle.getPortalContractAddress.mockImplementation(() => Promise.resolve(EthAddress.ZERO));

      const args = [amountToTransfer, owner, createFnSelector, getAndCheckFnSelector];
      const result = await runSimulator({
        args: args,
        abi: abi,
        origin: contractAddress,
        contractAddress: contractAddress,
      });

      const execCreate = result.nestedExecutions[0];
      const execGetAndCheck = result.nestedExecutions[1];

      expect(execCreate.preimages.newNotes).toHaveLength(1);
      const note = execCreate.preimages.newNotes[0];
      expect(note.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner, circuitsWasm));

      expect(note.preimage[0]).toEqual(new Fr(amountToTransfer));

      const newCommitments = execCreate.callStackItem.publicInputs.newCommitments.filter(
        field => !field.equals(Fr.ZERO),
      );

      expect(newCommitments).toHaveLength(1);

      const commitment = newCommitments[0];
      const storageSlot = computeSlotForMapping(new Fr(1n), owner, circuitsWasm);
      expect(commitment).toEqual(await acirSimulator.computeInnerNoteHash(contractAddress, storageSlot, note.preimage));
      // read request should match commitment
      const nonce = computeCommitmentNonce(circuitsWasm, txNullifier, 0);
      const readRequest = execGetAndCheck.callStackItem.publicInputs.readRequests[0];
      expect(readRequest).toEqual(computeUniqueCommitment(circuitsWasm, nonce, commitment));

      const gotNoteValue = execGetAndCheck.callStackItem.publicInputs.returnValues[0].value;
      expect(gotNoteValue).toEqual(amountToTransfer);

      // TODO check read request is output that matches pending commitment
    });

    it('cant read a commitment that is created later in same function', async () => {
      oracle.getNotes.mockResolvedValue([]);

      const amountToTransfer = 100n;

      const contractAddress = AztecAddress.random();
      const abi = PendingCommitmentsContractAbi.functions.find(f => f.name === 'test_bad_read_then_insert_flat')!;

      const args = [amountToTransfer, owner];
      const result = await runSimulator({
        args: args,
        abi: abi,
        origin: contractAddress,
        contractAddress: contractAddress,
      });

      expect(result.preimages.newNotes).toHaveLength(1);
      const note = result.preimages.newNotes[0];
      expect(note.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner, circuitsWasm));

      expect(note.preimage[0]).toEqual(new Fr(amountToTransfer));

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));

      expect(newCommitments).toHaveLength(1);

      const commitment = newCommitments[0];
      const storageSlot = computeSlotForMapping(new Fr(1n), owner, circuitsWasm);
      expect(commitment).toEqual(await acirSimulator.computeInnerNoteHash(contractAddress, storageSlot, note.preimage));
      // read requests should be empty
      const readRequest = result.callStackItem.publicInputs.readRequests[0].value;
      expect(readRequest).toEqual(0n);

      const gotNoteValue = result.callStackItem.publicInputs.returnValues[0].value;
      // should get note value 0 because it actually gets a fake note since the real one hasn't been created yet!
      expect(gotNoteValue).toEqual(0n);
    });
  });

  describe('get public key', () => {
    it('gets the public key for an address', async () => {
      // Tweak the contract ABI so we can extract return values
      const abi = TestContractAbi.functions.find(f => f.name === 'getPublicKey')!;
      abi.returnTypes = [{ kind: 'field' }, { kind: 'field' }];

      // Generate a partial address, pubkey, and resulting address
      const partialAddress = Fr.random();
      const pubKey = Point.random();
      const wasm = await CircuitsWasm.get();
      const address = computeContractAddressFromPartial(wasm, pubKey, partialAddress);
      const args = [address];

      oracle.getPublicKey.mockResolvedValue([pubKey, partialAddress]);
      const result = await runSimulator({ origin: AztecAddress.random(), abi, args });
      expect(result.returnValues).toEqual([pubKey.x.value, pubKey.y.value]);
    });
  });
});
