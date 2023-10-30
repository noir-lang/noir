import {
  CallContext,
  CircuitsWasm,
  CompleteAddress,
  ContractDeploymentData,
  EMPTY_NULLIFIED_COMMITMENT,
  FieldsOf,
  FunctionData,
  HistoricBlockData,
  L1_TO_L2_MSG_TREE_HEIGHT,
  MAX_NEW_COMMITMENTS_PER_CALL,
  NOTE_HASH_TREE_HEIGHT,
  PublicCallRequest,
  PublicKey,
  TxContext,
} from '@aztec/circuits.js';
import {
  computeCallStackItemHash,
  computeCommitmentNonce,
  computeSecretMessageHash,
  computeVarArgsHash,
  siloCommitment,
} from '@aztec/circuits.js/abis';
import { pedersenHashInputs } from '@aztec/circuits.js/barretenberg';
import { makeContractDeploymentData } from '@aztec/circuits.js/factories';
import { FunctionArtifact, FunctionSelector, encodeArguments } from '@aztec/foundation/abi';
import { asyncMap } from '@aztec/foundation/async-map';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr, GrumpkinScalar } from '@aztec/foundation/fields';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { AppendOnlyTree, Pedersen, StandardTree, newTree } from '@aztec/merkle-tree';
import {
  ChildContractArtifact,
  ImportTestContractArtifact,
  ParentContractArtifact,
  PendingCommitmentsContractArtifact,
  StatefulTestContractArtifact,
  TestContractArtifact,
  TokenContractArtifact,
} from '@aztec/noir-contracts/artifacts';
import { PackedArguments, TxExecutionRequest } from '@aztec/types';

import { jest } from '@jest/globals';
import { MockProxy, mock } from 'jest-mock-extended';
import { default as levelup } from 'levelup';
import { type MemDown, default as memdown } from 'memdown';
import { getFunctionSelector } from 'viem';

import { buildL1ToL2Message, getFunctionArtifact, getFunctionArtifactWithSelector } from '../test/utils.js';
import { computeSlotForMapping } from '../utils.js';
import { DBOracle } from './db_oracle.js';
import { AcirSimulator } from './simulator.js';

jest.setTimeout(60_000);

const createMemDown = () => (memdown as any)() as MemDown<any, any>;

describe('Private Execution test suite', () => {
  let circuitsWasm: CircuitsWasm;
  let oracle: MockProxy<DBOracle>;
  let acirSimulator: AcirSimulator;

  let blockData = HistoricBlockData.empty();
  let logger: DebugLogger;

  const defaultContractAddress = AztecAddress.random();
  const ownerPk = GrumpkinScalar.fromString('2dcc5485a58316776299be08c78fa3788a1a7961ae30dc747fb1be17692a8d32');
  const recipientPk = GrumpkinScalar.fromString('0c9ed344548e8f9ba8aa3c9f8651eaa2853130f6c1e9c050ccf198f7ea18a7ec');
  let owner: AztecAddress;
  let recipient: AztecAddress;
  let ownerCompleteAddress: CompleteAddress;
  let recipientCompleteAddress: CompleteAddress;

  const treeHeights: { [name: string]: number } = {
    noteHash: NOTE_HASH_TREE_HEIGHT,
    l1ToL2Messages: L1_TO_L2_MSG_TREE_HEIGHT,
  };

  const trees: { [name: keyof typeof treeHeights]: AppendOnlyTree } = {};
  const txContextFields: FieldsOf<TxContext> = {
    isContractDeploymentTx: false,
    isFeePaymentTx: false,
    isRebatePaymentTx: false,
    chainId: new Fr(10),
    version: new Fr(20),
    contractDeploymentData: ContractDeploymentData.empty(),
  };

  const runSimulator = async ({
    artifact,
    args = [],
    msgSender = AztecAddress.ZERO,
    contractAddress = defaultContractAddress,
    portalContractAddress = EthAddress.ZERO,
    txContext = {},
  }: {
    artifact: FunctionArtifact;
    msgSender?: AztecAddress;
    contractAddress?: AztecAddress;
    portalContractAddress?: EthAddress;
    args?: any[];
    txContext?: Partial<FieldsOf<TxContext>>;
  }) => {
    const packedArguments = await PackedArguments.fromArgs(encodeArguments(artifact, args), circuitsWasm);
    const functionData = FunctionData.fromAbi(artifact);
    const txRequest = TxExecutionRequest.from({
      origin: contractAddress,
      argsHash: packedArguments.hash,
      functionData,
      txContext: TxContext.from({ ...txContextFields, ...txContext }),
      packedArguments: [packedArguments],
      authWitnesses: [],
    });

    return acirSimulator.run(
      txRequest,
      artifact,
      functionData.isConstructor ? AztecAddress.ZERO : contractAddress,
      portalContractAddress,
      msgSender,
    );
  };

  const insertLeaves = async (leaves: Fr[], name = 'noteHash') => {
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
    const prevRoots = blockData.toBuffer();
    const rootIndex = name === 'noteHash' ? 0 : 32 * 3;
    const newRoots = Buffer.concat([prevRoots.subarray(0, rootIndex), newRoot, prevRoots.subarray(rootIndex + 32)]);
    blockData = HistoricBlockData.fromBuffer(newRoots);

    return trees[name];
  };

  const hash = (data: Buffer[]) => pedersenHashInputs(circuitsWasm, data);
  const hashFields = (data: Fr[]) =>
    Fr.fromBuffer(
      pedersenHashInputs(
        circuitsWasm,
        data.map(f => f.toBuffer()),
      ),
    );

  beforeAll(async () => {
    circuitsWasm = await CircuitsWasm.get();
    logger = createDebugLogger('aztec:test:private_execution');

    ownerCompleteAddress = await CompleteAddress.fromPrivateKeyAndPartialAddress(ownerPk, Fr.random());
    recipientCompleteAddress = await CompleteAddress.fromPrivateKeyAndPartialAddress(recipientPk, Fr.random());

    owner = ownerCompleteAddress.address;
    recipient = recipientCompleteAddress.address;
  });

  beforeEach(() => {
    oracle = mock<DBOracle>();
    oracle.getSecretKey.mockImplementation((contractAddress: AztecAddress, pubKey: PublicKey) => {
      if (pubKey.equals(ownerCompleteAddress.publicKey)) return Promise.resolve(ownerPk);
      if (pubKey.equals(recipientCompleteAddress.publicKey)) return Promise.resolve(recipientPk);
      throw new Error(`Unknown address ${pubKey}`);
    });
    oracle.getHistoricBlockData.mockResolvedValue(blockData);

    acirSimulator = new AcirSimulator(oracle);
  });

  describe('empty constructor', () => {
    it('should run the empty constructor', async () => {
      const artifact = getFunctionArtifact(TestContractArtifact, 'constructor');
      const contractDeploymentData = makeContractDeploymentData(100);
      const txContext = { isContractDeploymentTx: true, contractDeploymentData };
      const result = await runSimulator({ artifact, txContext });

      const emptyCommitments = new Array(MAX_NEW_COMMITMENTS_PER_CALL).fill(Fr.ZERO);
      expect(result.callStackItem.publicInputs.newCommitments).toEqual(emptyCommitments);
      expect(result.callStackItem.publicInputs.contractDeploymentData).toEqual(contractDeploymentData);
    });
  });

  describe('stateful test contract', () => {
    const contractAddress = defaultContractAddress;
    const mockFirstNullifier = new Fr(1111);
    let currentNoteIndex = 0n;

    const buildNote = (amount: bigint, owner: AztecAddress, storageSlot = Fr.random()) => {
      // WARNING: this is not actually how nonces are computed!
      // For the purpose of this test we use a mocked firstNullifier and and a random number
      // to compute the nonce. Proper nonces are only enforced later by the kernel/later circuits
      // which are not relevant to this test. In practice, the kernel first squashes all transient
      // noteHashes with their matching nullifiers. It then reorders the remaining "persistable"
      // noteHashes. A TX's real first nullifier (generated by the initial kernel) and a noteHash's
      // array index at the output of the final kernel/ordering circuit are used to derive nonce via:
      // `hash(firstNullifier, noteHashIndex)`
      const noteHashIndex = Math.floor(Math.random()); // mock index in TX's final newNoteHashes array
      const nonce = computeCommitmentNonce(circuitsWasm, mockFirstNullifier, noteHashIndex);
      const preimage = [new Fr(amount), owner.toField(), Fr.random()];
      const innerNoteHash = Fr.fromBuffer(hash(preimage.map(p => p.toBuffer())));
      return {
        contractAddress,
        storageSlot,
        nonce,
        preimage,
        innerNoteHash,
        siloedNullifier: new Fr(0),
        index: currentNoteIndex++,
      };
    };

    beforeEach(() => {
      oracle.getCompleteAddress.mockImplementation((address: AztecAddress) => {
        if (address.equals(owner)) return Promise.resolve(ownerCompleteAddress);
        if (address.equals(recipient)) return Promise.resolve(recipientCompleteAddress);
        throw new Error(`Unknown address ${address}`);
      });

      oracle.getFunctionArtifactByName.mockImplementation((_, functionName: string) =>
        Promise.resolve(getFunctionArtifact(StatefulTestContractArtifact, functionName)),
      );
    });

    it('should have a constructor with arguments that inserts notes', async () => {
      const artifact = getFunctionArtifact(StatefulTestContractArtifact, 'constructor');

      const result = await runSimulator({ args: [owner, 140], artifact });

      expect(result.newNotes).toHaveLength(1);
      const newNote = result.newNotes[0];
      expect(newNote.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm));

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));
      expect(newCommitments).toHaveLength(1);

      const [commitment] = newCommitments;
      expect(commitment).toEqual(
        await acirSimulator.computeInnerNoteHash(contractAddress, newNote.storageSlot, newNote.preimage),
      );
    });

    it('should run the create_note function', async () => {
      const artifact = getFunctionArtifact(StatefulTestContractArtifact, 'create_note');

      const result = await runSimulator({ args: [owner, 140], artifact });

      expect(result.newNotes).toHaveLength(1);
      const newNote = result.newNotes[0];
      expect(newNote.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm));

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));
      expect(newCommitments).toHaveLength(1);

      const [commitment] = newCommitments;
      expect(commitment).toEqual(
        await acirSimulator.computeInnerNoteHash(contractAddress, newNote.storageSlot, newNote.preimage),
      );
    });

    it('should run the destroy_and_create function', async () => {
      const amountToTransfer = 100n;
      const artifact = getFunctionArtifact(StatefulTestContractArtifact, 'destroy_and_create');

      const storageSlot = computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm);
      const recipientStorageSlot = computeSlotForMapping(new Fr(1n), recipient.toField(), circuitsWasm);

      const notes = [buildNote(60n, owner, storageSlot), buildNote(80n, owner, storageSlot)];
      oracle.getNotes.mockResolvedValue(notes);

      const consumedNotes = await asyncMap(notes, ({ nonce, preimage }) =>
        acirSimulator.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, preimage),
      );
      await insertLeaves(consumedNotes.map(n => n.siloedNoteHash));

      const args = [recipient, amountToTransfer];
      const result = await runSimulator({ args, artifact, msgSender: owner });

      // The two notes were nullified
      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toHaveLength(consumedNotes.length);
      expect(newNullifiers).toEqual(expect.arrayContaining(consumedNotes.map(n => n.innerNullifier)));

      expect(result.newNotes).toHaveLength(2);
      const [changeNote, recipientNote] = result.newNotes;
      expect(recipientNote.storageSlot).toEqual(recipientStorageSlot);

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));
      expect(newCommitments).toHaveLength(2);

      const [changeNoteCommitment, recipientNoteCommitment] = newCommitments;
      expect(recipientNoteCommitment).toEqual(
        await acirSimulator.computeInnerNoteHash(contractAddress, recipientStorageSlot, recipientNote.preimage),
      );
      expect(changeNoteCommitment).toEqual(
        await acirSimulator.computeInnerNoteHash(contractAddress, storageSlot, changeNote.preimage),
      );

      expect(recipientNote.preimage[0]).toEqual(new Fr(amountToTransfer));
      expect(changeNote.preimage[0]).toEqual(new Fr(40n));

      const readRequests = result.callStackItem.publicInputs.readRequests.filter(field => !field.equals(Fr.ZERO));
      expect(readRequests).toHaveLength(consumedNotes.length);
      expect(readRequests).toEqual(expect.arrayContaining(consumedNotes.map(n => n.uniqueSiloedNoteHash)));
    });

    it('should be able to destroy_and_create with dummy notes', async () => {
      const amountToTransfer = 100n;
      const balance = 160n;
      const artifact = getFunctionArtifact(StatefulTestContractArtifact, 'destroy_and_create');

      const storageSlot = computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm);

      const notes = [buildNote(balance, owner, storageSlot)];
      oracle.getNotes.mockResolvedValue(notes);

      const consumedNotes = await asyncMap(notes, ({ nonce, preimage }) =>
        acirSimulator.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, preimage),
      );
      await insertLeaves(consumedNotes.map(n => n.siloedNoteHash));

      const args = [recipient, amountToTransfer];
      const result = await runSimulator({ args, artifact, msgSender: owner });

      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toEqual(consumedNotes.map(n => n.innerNullifier));

      expect(result.newNotes).toHaveLength(2);
      const [changeNote, recipientNote] = result.newNotes;
      expect(recipientNote.preimage[0]).toEqual(new Fr(amountToTransfer));
      expect(changeNote.preimage[0]).toEqual(new Fr(balance - amountToTransfer));
    });
  });

  describe('nested calls', () => {
    const privateIncrement = txContextFields.chainId.value + txContextFields.version.value;

    it('child function should be callable', async () => {
      const initialValue = 100n;
      const artifact = getFunctionArtifact(ChildContractArtifact, 'value');
      const result = await runSimulator({ args: [initialValue], artifact });

      expect(result.callStackItem.publicInputs.returnValues[0]).toEqual(new Fr(initialValue + privateIncrement));
    });

    it('parent should call child', async () => {
      const childArtifact = getFunctionArtifact(ChildContractArtifact, 'value');
      const parentArtifact = getFunctionArtifact(ParentContractArtifact, 'entryPoint');
      const parentAddress = AztecAddress.random();
      const childAddress = AztecAddress.random();
      const childSelector = FunctionSelector.fromNameAndParameters(childArtifact.name, childArtifact.parameters);

      oracle.getFunctionArtifact.mockImplementation(() => Promise.resolve(childArtifact));
      oracle.getPortalContractAddress.mockImplementation(() => Promise.resolve(EthAddress.ZERO));

      logger(`Parent deployed at ${parentAddress.toShortString()}`);
      logger(`Calling child function ${childSelector.toString()} at ${childAddress.toShortString()}`);

      const args = [Fr.fromBuffer(childAddress.toBuffer()), Fr.fromBuffer(childSelector.toBuffer())];
      const result = await runSimulator({ args, artifact: parentArtifact });

      expect(result.callStackItem.publicInputs.returnValues[0]).toEqual(new Fr(privateIncrement));
      expect(oracle.getFunctionArtifact.mock.calls[0]).toEqual([childAddress, childSelector]);
      expect(oracle.getPortalContractAddress.mock.calls[0]).toEqual([childAddress]);
      expect(result.nestedExecutions).toHaveLength(1);
      expect(result.nestedExecutions[0].callStackItem.publicInputs.returnValues[0]).toEqual(new Fr(privateIncrement));

      // check that Aztec.nr calculated the call stack item hash like cpp does
      const wasm = await CircuitsWasm.get();
      const expectedCallStackItemHash = computeCallStackItemHash(wasm, result.nestedExecutions[0].callStackItem);
      expect(result.callStackItem.publicInputs.privateCallStack[0]).toEqual(expectedCallStackItemHash);
    });
  });

  describe('nested calls through autogenerated interface', () => {
    let args: any[];
    let argsHash: Fr;
    let testCodeGenArtifact: FunctionArtifact;

    beforeAll(async () => {
      // These args should match the ones hardcoded in importer contract
      const dummyNote = { amount: 1, secretHash: 2 };
      const deepStruct = { aField: 1, aBool: true, aNote: dummyNote, manyNotes: [dummyNote, dummyNote, dummyNote] };
      args = [1, true, 1, [1, 2], dummyNote, deepStruct];
      testCodeGenArtifact = getFunctionArtifact(TestContractArtifact, 'test_code_gen');
      const serializedArgs = encodeArguments(testCodeGenArtifact, args);
      argsHash = await computeVarArgsHash(await CircuitsWasm.get(), serializedArgs);
    });

    it('test function should be directly callable', async () => {
      logger(`Calling testCodeGen function`);
      const result = await runSimulator({ args, artifact: testCodeGenArtifact });

      expect(result.callStackItem.publicInputs.returnValues[0]).toEqual(argsHash);
    });

    it('test function should be callable through autogenerated interface', async () => {
      const testAddress = AztecAddress.random();
      const parentArtifact = getFunctionArtifact(ImportTestContractArtifact, 'main');
      const testCodeGenSelector = FunctionSelector.fromNameAndParameters(
        testCodeGenArtifact.name,
        testCodeGenArtifact.parameters,
      );

      oracle.getFunctionArtifact.mockResolvedValue(testCodeGenArtifact);
      oracle.getPortalContractAddress.mockResolvedValue(EthAddress.ZERO);

      logger(`Calling importer main function`);
      const args = [testAddress];
      const result = await runSimulator({ args, artifact: parentArtifact });

      expect(result.callStackItem.publicInputs.returnValues[0]).toEqual(argsHash);
      expect(oracle.getFunctionArtifact.mock.calls[0]).toEqual([testAddress, testCodeGenSelector]);
      expect(oracle.getPortalContractAddress.mock.calls[0]).toEqual([testAddress]);
      expect(result.nestedExecutions).toHaveLength(1);
      expect(result.nestedExecutions[0].callStackItem.publicInputs.returnValues[0]).toEqual(argsHash);
    });
  });

  describe('consuming messages', () => {
    const contractAddress = defaultContractAddress;

    beforeEach(() => {
      oracle.getCompleteAddress.mockImplementation((address: AztecAddress) => {
        if (address.equals(recipient)) return Promise.resolve(recipientCompleteAddress);
        throw new Error(`Unknown address ${address}`);
      });
    });

    it('Should be able to consume a dummy cross chain message', async () => {
      const bridgedAmount = 100n;
      const artifact = getFunctionArtifact(TestContractArtifact, 'consume_mint_private_message');

      const secretForL1ToL2MessageConsumption = new Fr(1n);
      const secretHashForRedeemingNotes = new Fr(2n);
      const canceller = EthAddress.random();
      const preimage = await buildL1ToL2Message(
        getFunctionSelector('mint_private(bytes32,uint256,address)').substring(2),
        [secretHashForRedeemingNotes, new Fr(bridgedAmount), canceller.toField()],
        contractAddress,
        secretForL1ToL2MessageConsumption,
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

      const args = [
        secretHashForRedeemingNotes,
        bridgedAmount,
        canceller.toField(),
        messageKey,
        secretForL1ToL2MessageConsumption,
      ];
      const result = await runSimulator({ contractAddress, artifact, args });

      // Check a nullifier has been inserted
      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toHaveLength(1);
    });

    it('Should be able to consume a dummy public to private message', async () => {
      const amount = 100n;
      const artifact = getFunctionArtifact(TokenContractArtifact, 'redeem_shield');

      const wasm = await CircuitsWasm.get();
      const secret = new Fr(1n);
      const secretHash = computeSecretMessageHash(wasm, secret);
      const preimage = [toBufferBE(amount, 32), secretHash.toBuffer()];
      const noteHash = Fr.fromBuffer(hash(preimage));
      const storageSlot = new Fr(5);
      const innerNoteHash = Fr.fromBuffer(hash([storageSlot.toBuffer(), noteHash.toBuffer()]));
      const siloedNoteHash = siloCommitment(wasm, contractAddress, innerNoteHash);
      oracle.getNotes.mockResolvedValue([
        {
          contractAddress,
          storageSlot,
          nonce: Fr.ZERO,
          preimage: preimage.map(p => Fr.fromBuffer(p)),
          innerNoteHash: new Fr(EMPTY_NULLIFIED_COMMITMENT),
          siloedNullifier: Fr.random(),
          index: 1n,
        },
      ]);

      const result = await runSimulator({
        artifact,
        args: [recipient, amount, secret],
      });

      // Check a nullifier has been inserted.
      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toHaveLength(1);

      // Check the commitment read request was created successfully.
      const readRequests = result.callStackItem.publicInputs.readRequests.filter(field => !field.equals(Fr.ZERO));
      expect(readRequests).toHaveLength(1);
      expect(readRequests[0]).toEqual(siloedNoteHash);
    });
  });

  describe('enqueued calls', () => {
    it.each([false, true])('parent should enqueue call to child (internal %p)', async isInternal => {
      const parentArtifact = getFunctionArtifact(ParentContractArtifact, 'enqueueCallToChild');
      const childContractArtifact = ParentContractArtifact.functions[0];
      const childAddress = AztecAddress.random();
      const childPortalContractAddress = EthAddress.random();
      const childSelector = FunctionSelector.fromNameAndParameters(
        childContractArtifact.name,
        childContractArtifact.parameters,
      );
      const parentAddress = AztecAddress.random();

      oracle.getPortalContractAddress.mockImplementation(() => Promise.resolve(childPortalContractAddress));
      oracle.getFunctionArtifact.mockImplementation(() => Promise.resolve({ ...childContractArtifact, isInternal }));

      const args = [Fr.fromBuffer(childAddress.toBuffer()), childSelector.toField(), 42n];
      const result = await runSimulator({
        msgSender: parentAddress,
        contractAddress: parentAddress,
        artifact: parentArtifact,
        args,
      });

      // Alter function data to match the manipulated oracle
      const functionData = FunctionData.fromAbi(childContractArtifact);
      functionData.isInternal = isInternal;

      const publicCallRequest = PublicCallRequest.from({
        contractAddress: childAddress,
        functionData: functionData,
        args: [new Fr(42n)],
        callContext: CallContext.from({
          msgSender: parentAddress,
          storageContractAddress: childAddress,
          portalContractAddress: childPortalContractAddress,
          functionSelector: childSelector,
          isContractDeployment: false,
          isDelegateCall: false,
          isStaticCall: false,
        }),
        sideEffectCounter: 0,
      });

      const publicCallRequestHash = computeCallStackItemHash(
        await CircuitsWasm.get(),
        await publicCallRequest.toPublicCallStackItem(),
      );

      expect(result.enqueuedPublicFunctionCalls).toHaveLength(1);
      expect(result.enqueuedPublicFunctionCalls[0]).toEqual(publicCallRequest);
      expect(result.callStackItem.publicInputs.publicCallStack[0]).toEqual(publicCallRequestHash);
    });
  });

  describe('pending commitments contract', () => {
    beforeEach(() => {
      oracle.getCompleteAddress.mockImplementation((address: AztecAddress) => {
        if (address.equals(owner)) return Promise.resolve(ownerCompleteAddress);
        throw new Error(`Unknown address ${address}`);
      });
    });

    beforeEach(() => {
      oracle.getFunctionArtifact.mockImplementation((_, selector) =>
        Promise.resolve(getFunctionArtifactWithSelector(PendingCommitmentsContractArtifact, selector)),
      );
      oracle.getFunctionArtifactByName.mockImplementation((_, functionName: string) =>
        Promise.resolve(getFunctionArtifact(PendingCommitmentsContractArtifact, functionName)),
      );
    });

    it('should be able to insert, read, and nullify pending commitments in one call', async () => {
      oracle.getNotes.mockResolvedValue([]);

      const amountToTransfer = 100n;

      const contractAddress = AztecAddress.random();
      const artifact = getFunctionArtifact(
        PendingCommitmentsContractArtifact,
        'test_insert_then_get_then_nullify_flat',
      );

      const args = [amountToTransfer, owner];
      const result = await runSimulator({
        args: args,
        artifact: artifact,
        contractAddress,
      });

      expect(result.newNotes).toHaveLength(1);
      const note = result.newNotes[0];
      expect(note.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm));

      expect(note.preimage[0]).toEqual(new Fr(amountToTransfer));

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));
      expect(newCommitments).toHaveLength(1);

      const commitment = newCommitments[0];
      const storageSlot = computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm);
      const innerNoteHash = await acirSimulator.computeInnerNoteHash(contractAddress, storageSlot, note.preimage);
      expect(commitment).toEqual(innerNoteHash);

      // read request should match innerNoteHash for pending notes (there is no nonce, so can't compute "unique" hash)
      const readRequest = result.callStackItem.publicInputs.readRequests[0];
      expect(readRequest).toEqual(innerNoteHash);

      const gotNoteValue = result.callStackItem.publicInputs.returnValues[0].value;
      expect(gotNoteValue).toEqual(amountToTransfer);

      const nullifier = result.callStackItem.publicInputs.newNullifiers[0];
      const expectedNullifier = hashFields([innerNoteHash, ownerPk.low, ownerPk.high]);
      expect(nullifier).toEqual(expectedNullifier);
    });

    it('should be able to insert, read, and nullify pending commitments in nested calls', async () => {
      oracle.getNotes.mockResolvedValue([]);

      const amountToTransfer = 100n;

      const contractAddress = AztecAddress.random();
      const artifact = getFunctionArtifact(
        PendingCommitmentsContractArtifact,
        'test_insert_then_get_then_nullify_all_in_nested_calls',
      );
      const insertArtifact = getFunctionArtifact(PendingCommitmentsContractArtifact, 'insert_note');

      const getThenNullifyArtifact = getFunctionArtifact(PendingCommitmentsContractArtifact, 'get_then_nullify_note');

      const getZeroArtifact = getFunctionArtifact(PendingCommitmentsContractArtifact, 'get_note_zero_balance');

      const insertFnSelector = FunctionSelector.fromNameAndParameters(insertArtifact.name, insertArtifact.parameters);
      const getThenNullifyFnSelector = FunctionSelector.fromNameAndParameters(
        getThenNullifyArtifact.name,
        getThenNullifyArtifact.parameters,
      );
      const getZeroFnSelector = FunctionSelector.fromNameAndParameters(
        getZeroArtifact.name,
        getZeroArtifact.parameters,
      );

      oracle.getPortalContractAddress.mockImplementation(() => Promise.resolve(EthAddress.ZERO));

      const args = [
        amountToTransfer,
        owner,
        insertFnSelector.toField(),
        getThenNullifyFnSelector.toField(),
        getZeroFnSelector.toField(),
      ];
      const result = await runSimulator({
        args: args,
        artifact: artifact,
        contractAddress: contractAddress,
      });

      const execInsert = result.nestedExecutions[0];
      const execGetThenNullify = result.nestedExecutions[1];
      const getNotesAfterNullify = result.nestedExecutions[2];

      expect(execInsert.newNotes).toHaveLength(1);
      const note = execInsert.newNotes[0];
      expect(note.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm));

      expect(note.preimage[0]).toEqual(new Fr(amountToTransfer));

      const newCommitments = execInsert.callStackItem.publicInputs.newCommitments.filter(
        field => !field.equals(Fr.ZERO),
      );
      expect(newCommitments).toHaveLength(1);

      const commitment = newCommitments[0];
      const storageSlot = computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm);
      const innerNoteHash = await acirSimulator.computeInnerNoteHash(contractAddress, storageSlot, note.preimage);
      expect(commitment).toEqual(innerNoteHash);

      // read request should match innerNoteHash for pending notes (there is no nonce, so can't compute "unique" hash)
      const readRequest = execGetThenNullify.callStackItem.publicInputs.readRequests[0];
      expect(readRequest).toEqual(innerNoteHash);

      const gotNoteValue = execGetThenNullify.callStackItem.publicInputs.returnValues[0].value;
      expect(gotNoteValue).toEqual(amountToTransfer);

      const nullifier = execGetThenNullify.callStackItem.publicInputs.newNullifiers[0];
      const expectedNullifier = hashFields([innerNoteHash, ownerPk.low, ownerPk.high]);
      expect(nullifier).toEqual(expectedNullifier);

      // check that the last get_notes call return no note
      const afterNullifyingNoteValue = getNotesAfterNullify.callStackItem.publicInputs.returnValues[0].value;
      expect(afterNullifyingNoteValue).toEqual(0n);
    });

    it('cant read a commitment that is inserted later in same call', async () => {
      oracle.getNotes.mockResolvedValue([]);

      const amountToTransfer = 100n;

      const contractAddress = AztecAddress.random();

      const artifact = getFunctionArtifact(PendingCommitmentsContractArtifact, 'test_bad_get_then_insert_flat');

      const args = [amountToTransfer, owner];
      const result = await runSimulator({
        args: args,
        artifact: artifact,
        contractAddress,
      });

      expect(result.newNotes).toHaveLength(1);
      const note = result.newNotes[0];
      expect(note.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm));

      expect(note.preimage[0]).toEqual(new Fr(amountToTransfer));

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));
      expect(newCommitments).toHaveLength(1);

      const commitment = newCommitments[0];
      const storageSlot = computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm);
      expect(commitment).toEqual(await acirSimulator.computeInnerNoteHash(contractAddress, storageSlot, note.preimage));

      // read requests should be empty
      const readRequest = result.callStackItem.publicInputs.readRequests[0].value;
      expect(readRequest).toEqual(0n);

      // should get note value 0 because it actually gets a fake note since the real one hasn't been inserted yet!
      const gotNoteValue = result.callStackItem.publicInputs.returnValues[0].value;
      expect(gotNoteValue).toEqual(0n);

      // there should be no nullifiers
      const nullifier = result.callStackItem.publicInputs.newNullifiers[0].value;
      expect(nullifier).toEqual(0n);
    });
  });

  describe('get public key', () => {
    it('gets the public key for an address', async () => {
      // Tweak the contract artifact so we can extract return values
      const artifact = getFunctionArtifact(TestContractArtifact, 'get_public_key');
      artifact.returnTypes = [{ kind: 'array', length: 2, type: { kind: 'field' } }];

      // Generate a partial address, pubkey, and resulting address
      const completeAddress = await CompleteAddress.random();
      const args = [completeAddress.address];
      const pubKey = completeAddress.publicKey;

      oracle.getCompleteAddress.mockResolvedValue(completeAddress);
      const result = await runSimulator({ artifact, args });
      expect(result.returnValues).toEqual([pubKey.x.value, pubKey.y.value]);
    });
  });

  describe('Context oracles', () => {
    it("Should be able to get and return the contract's portal contract address", async () => {
      const portalContractAddress = EthAddress.random();
      const aztecAddressToQuery = AztecAddress.random();

      // Tweak the contract artifact so we can extract return values
      const artifact = getFunctionArtifact(TestContractArtifact, 'get_portal_contract_address');
      artifact.returnTypes = [{ kind: 'field' }];

      const args = [aztecAddressToQuery.toField()];

      // Overwrite the oracle return value
      oracle.getPortalContractAddress.mockResolvedValue(portalContractAddress);
      const result = await runSimulator({ artifact, args });
      expect(result.returnValues).toEqual(portalContractAddress.toField().value);
    });

    it('this_address should return the current context address', async () => {
      const contractAddress = AztecAddress.random();

      // Tweak the contract artifact so we can extract return values
      const artifact = getFunctionArtifact(TestContractArtifact, 'get_this_address');
      artifact.returnTypes = [{ kind: 'field' }];

      // Overwrite the oracle return value
      const result = await runSimulator({ artifact, args: [], contractAddress });
      expect(result.returnValues).toEqual(contractAddress.toField().value);
    });

    it("this_portal_address should return the current context's portal address", async () => {
      const portalContractAddress = EthAddress.random();

      // Tweak the contract artifact so we can extract return values
      const artifact = getFunctionArtifact(TestContractArtifact, 'get_this_portal_address');
      artifact.returnTypes = [{ kind: 'field' }];

      // Overwrite the oracle return value
      const result = await runSimulator({ artifact, args: [], portalContractAddress });
      expect(result.returnValues).toEqual(portalContractAddress.toField().value);
    });
  });
});
