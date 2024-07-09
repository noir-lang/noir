import {
  type AztecNode,
  EncryptedNoteFunctionL2Logs,
  type L1ToL2Message,
  type L2BlockNumber,
  Note,
  PackedValues,
  PublicDataWitness,
  SiblingPath,
  TxExecutionRequest,
} from '@aztec/circuit-types';
import {
  AppendOnlyTreeSnapshot,
  CallContext,
  CompleteAddress,
  GasSettings,
  GeneratorIndex,
  type GrumpkinScalar,
  Header,
  KeyValidationRequest,
  L1_TO_L2_MSG_TREE_HEIGHT,
  NOTE_HASH_TREE_HEIGHT,
  PUBLIC_DATA_TREE_HEIGHT,
  PartialStateReference,
  PublicCallRequest,
  PublicDataTreeLeafPreimage,
  StateReference,
  TxContext,
  computeAppNullifierSecretKey,
  computeOvskApp,
  deriveKeys,
  getContractInstanceFromDeployParams,
  getNonEmptyItems,
} from '@aztec/circuits.js';
import { computeNoteHashNonce, computeSecretHash, computeVarArgsHash } from '@aztec/circuits.js/hash';
import { makeHeader } from '@aztec/circuits.js/testing';
import {
  type FunctionArtifact,
  FunctionSelector,
  type NoteSelector,
  encodeArguments,
  getFunctionArtifact,
} from '@aztec/foundation/abi';
import { asyncMap } from '@aztec/foundation/async-map';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { times } from '@aztec/foundation/collection';
import { pedersenHash, poseidon2Hash, randomInt } from '@aztec/foundation/crypto';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { type DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { type FieldsOf } from '@aztec/foundation/types';
import { openTmpStore } from '@aztec/kv-store/utils';
import { type AppendOnlyTree, INITIAL_LEAF, Pedersen, StandardTree, newTree } from '@aztec/merkle-tree';
import {
  ChildContractArtifact,
  ImportTestContractArtifact,
  ParentContractArtifact,
  PendingNoteHashesContractArtifact,
  StatefulTestContractArtifact,
  TestContractArtifact,
} from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';
import { type MockProxy, mock } from 'jest-mock-extended';
import { toFunctionSelector } from 'viem';

import { MessageLoadOracleInputs } from '../acvm/index.js';
import { buildL1ToL2Message } from '../test/utils.js';
import { computeSlotForMapping } from '../utils.js';
import { type DBOracle } from './db_oracle.js';
import { type ExecutionResult, collectSortedEncryptedLogs } from './execution_result.js';
import { AcirSimulator } from './simulator.js';

jest.setTimeout(60_000);

describe('Private Execution test suite', () => {
  let oracle: MockProxy<DBOracle>;
  let node: MockProxy<AztecNode>;

  let acirSimulator: AcirSimulator;

  let header = Header.empty();
  let logger: DebugLogger;

  const defaultContractAddress = AztecAddress.random();
  const ownerSk = Fr.fromString('2dcc5485a58316776299be08c78fa3788a1a7961ae30dc747fb1be17692a8d32');
  const recipientSk = Fr.fromString('0c9ed344548e8f9ba8aa3c9f8651eaa2853130f6c1e9c050ccf198f7ea18a7ec');
  let owner: AztecAddress;
  let recipient: AztecAddress;
  let ownerCompleteAddress: CompleteAddress;
  let recipientCompleteAddress: CompleteAddress;

  let ownerNskM: GrumpkinScalar;
  let ownerOvskM: GrumpkinScalar;
  let recipientNskM: GrumpkinScalar;
  let recipientOvskM: GrumpkinScalar;

  const treeHeights: { [name: string]: number } = {
    noteHash: NOTE_HASH_TREE_HEIGHT,
    l1ToL2Messages: L1_TO_L2_MSG_TREE_HEIGHT,
    publicData: PUBLIC_DATA_TREE_HEIGHT,
  };

  let trees: { [name: keyof typeof treeHeights]: AppendOnlyTree<Fr> } = {};
  const txContextFields: FieldsOf<TxContext> = {
    chainId: new Fr(10),
    version: new Fr(20),
    gasSettings: GasSettings.default(),
  };

  const runSimulator = ({
    artifact,
    args = [],
    msgSender = AztecAddress.ZERO,
    contractAddress = defaultContractAddress,
    txContext = {},
  }: {
    artifact: FunctionArtifact;
    msgSender?: AztecAddress;
    contractAddress?: AztecAddress;
    args?: any[];
    txContext?: Partial<FieldsOf<TxContext>>;
  }) => {
    const packedArguments = PackedValues.fromValues(encodeArguments(artifact, args));
    const txRequest = TxExecutionRequest.from({
      origin: contractAddress,
      firstCallArgsHash: packedArguments.hash,
      functionSelector: FunctionSelector.fromNameAndParameters(artifact.name, artifact.parameters),
      txContext: TxContext.from({ ...txContextFields, ...txContext }),
      argsOfCalls: [packedArguments],
      authWitnesses: [],
    });

    return acirSimulator.run(txRequest, artifact, contractAddress, msgSender);
  };

  const insertLeaves = async (leaves: Fr[], name = 'noteHash') => {
    if (!treeHeights[name]) {
      throw new Error(`Unknown tree ${name}`);
    }
    if (!trees[name]) {
      const db = openTmpStore();
      const pedersen = new Pedersen();
      trees[name] = await newTree(StandardTree, db, pedersen, name, Fr, treeHeights[name]);
    }
    const tree = trees[name];

    await tree.appendLeaves(leaves);

    // Create a new snapshot.
    const newSnap = new AppendOnlyTreeSnapshot(Fr.fromBuffer(tree.getRoot(true)), Number(tree.getNumLeaves(true)));

    if (name === 'noteHash' || name === 'l1ToL2Messages' || name === 'publicData') {
      header = new Header(
        header.lastArchive,
        header.contentCommitment,
        new StateReference(
          name === 'l1ToL2Messages' ? newSnap : header.state.l1ToL2MessageTree,
          new PartialStateReference(
            name === 'noteHash' ? newSnap : header.state.partial.noteHashTree,
            header.state.partial.nullifierTree,
            name === 'publicData' ? newSnap : header.state.partial.publicDataTree,
          ),
        ),
        header.globalVariables,
        header.totalFees,
      );
    } else {
      header = new Header(
        header.lastArchive,
        header.contentCommitment,
        new StateReference(newSnap, header.state.partial),
        header.globalVariables,
        header.totalFees,
      );
    }

    return trees[name];
  };

  const getEncryptedNoteSerializedLength = (result: ExecutionResult) => {
    const fnLogs = new EncryptedNoteFunctionL2Logs(result.noteEncryptedLogs.map(l => l.log));
    return fnLogs.getKernelLength();
  };

  beforeAll(() => {
    logger = createDebugLogger('aztec:test:private_execution');

    const ownerPartialAddress = Fr.random();
    ownerCompleteAddress = CompleteAddress.fromSecretKeyAndPartialAddress(ownerSk, ownerPartialAddress);
    ({ masterNullifierSecretKey: ownerNskM, masterOutgoingViewingSecretKey: ownerOvskM } = deriveKeys(ownerSk));

    const recipientPartialAddress = Fr.random();
    recipientCompleteAddress = CompleteAddress.fromSecretKeyAndPartialAddress(recipientSk, recipientPartialAddress);
    ({ masterNullifierSecretKey: recipientNskM, masterOutgoingViewingSecretKey: recipientOvskM } =
      deriveKeys(recipientSk));

    owner = ownerCompleteAddress.address;
    recipient = recipientCompleteAddress.address;
  });

  beforeEach(async () => {
    trees = {};
    oracle = mock<DBOracle>();
    oracle.getKeyValidationRequest.mockImplementation((pkMHash: Fr, contractAddress: AztecAddress) => {
      if (pkMHash.equals(ownerCompleteAddress.publicKeys.masterNullifierPublicKey.hash())) {
        return Promise.resolve(
          new KeyValidationRequest(
            ownerCompleteAddress.publicKeys.masterNullifierPublicKey,
            computeAppNullifierSecretKey(ownerNskM, contractAddress),
          ),
        );
      }
      if (pkMHash.equals(ownerCompleteAddress.publicKeys.masterOutgoingViewingPublicKey.hash())) {
        return Promise.resolve(
          new KeyValidationRequest(
            ownerCompleteAddress.publicKeys.masterOutgoingViewingPublicKey,
            computeOvskApp(ownerOvskM, contractAddress),
          ),
        );
      }
      if (pkMHash.equals(recipientCompleteAddress.publicKeys.masterNullifierPublicKey.hash())) {
        return Promise.resolve(
          new KeyValidationRequest(
            recipientCompleteAddress.publicKeys.masterNullifierPublicKey,
            computeAppNullifierSecretKey(recipientNskM, contractAddress),
          ),
        );
      }
      if (pkMHash.equals(recipientCompleteAddress.publicKeys.masterOutgoingViewingPublicKey.hash())) {
        return Promise.resolve(
          new KeyValidationRequest(
            recipientCompleteAddress.publicKeys.masterOutgoingViewingPublicKey,
            computeOvskApp(recipientOvskM, contractAddress),
          ),
        );
      }
      throw new Error(`Unknown master public key hash: ${pkMHash}`);
    });

    // We call insertLeaves here with no leaves to populate empty public data tree root --> this is necessary to be
    // able to get ivpk_m during execution
    await insertLeaves([], 'publicData');
    oracle.getHeader.mockResolvedValue(header);

    oracle.getCompleteAddress.mockImplementation((address: AztecAddress) => {
      if (address.equals(owner)) {
        return Promise.resolve(ownerCompleteAddress);
      }
      if (address.equals(recipient)) {
        return Promise.resolve(recipientCompleteAddress);
      }
      throw new Error(`Unknown address: ${address}. Recipient: ${recipient}, Owner: ${owner}`);
    });
    // This oracle gets called when reading ivpk_m from key registry --> we return zero witness indicating that
    // the keys were not registered. This triggers non-registered keys flow in which getCompleteAddress oracle
    // gets called and we constrain the result by hashing address preimage and checking it matches.
    oracle.getPublicDataTreeWitness.mockResolvedValue(
      new PublicDataWitness(
        0n,
        PublicDataTreeLeafPreimage.empty(),
        SiblingPath.ZERO(PUBLIC_DATA_TREE_HEIGHT, INITIAL_LEAF, new Pedersen()),
      ),
    );

    node = mock<AztecNode>();
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    node.getPublicStorageAt.mockImplementation((address: Fr, storageSlot: Fr, blockNumber: L2BlockNumber) => {
      return Promise.resolve(Fr.ZERO);
    });

    acirSimulator = new AcirSimulator(oracle, node);
  });

  describe('no constructor', () => {
    it('emits a field array as an encrypted log', async () => {
      // NB: this test does NOT cover correct enc/dec of values, just whether
      // the kernels correctly populate non-note encrypted logs
      const artifact = getFunctionArtifact(TestContractArtifact, 'emit_array_as_encrypted_log');
      // We emit the outgoing here to recipient for no reason at all
      const outgoingViewer = recipient;
      const args = [times(5, () => Fr.random()), owner, outgoingViewer, false];
      const result = await runSimulator({ artifact, msgSender: owner, args });

      const newEncryptedLogs = getNonEmptyItems(result.callStackItem.publicInputs.encryptedLogsHashes);
      expect(newEncryptedLogs).toHaveLength(1);
      const functionLogs = collectSortedEncryptedLogs(result);
      expect(functionLogs.logs).toHaveLength(1);

      const [encryptedLog] = newEncryptedLogs;
      expect(encryptedLog.value).toEqual(Fr.fromBuffer(functionLogs.logs[0].hash()));
      expect(encryptedLog.length).toEqual(new Fr(functionLogs.getKernelLength()));
      // 5 is hardcoded in the test contract
      expect(encryptedLog.randomness).toEqual(new Fr(5));
      const expectedMaskedAddress = pedersenHash([result.callStackItem.contractAddress, new Fr(5)], 0);
      expect(expectedMaskedAddress).toEqual(functionLogs.logs[0].maskedContractAddress);
    });
  });

  describe('stateful test contract', () => {
    const valueNoteTypeId = StatefulTestContractArtifact.notes['ValueNote'].id;

    const contractAddress = defaultContractAddress;
    const mockFirstNullifier = new Fr(1111);
    let currentNoteIndex = 0n;

    const buildNote = (amount: bigint, ownerNpkMHash: Fr, storageSlot: Fr, noteTypeId: NoteSelector) => {
      // WARNING: this is not actually how nonces are computed!
      // For the purpose of this test we use a mocked firstNullifier and and a random number
      // to compute the nonce. Proper nonces are only enforced later by the kernel/later circuits
      // which are not relevant to this test. In practice, the kernel first squashes all transient
      // noteHashes with their matching nullifiers. It then reorders the remaining "persistable"
      // noteHashes. A TX's real first nullifier (generated by the initial kernel) and a noteHash's
      // array index at the output of the final kernel/ordering circuit are used to derive nonce via:
      // `hash(firstNullifier, noteHashIndex)`
      const noteHashIndex = randomInt(1); // mock index in TX's final noteHashes array
      const nonce = computeNoteHashNonce(mockFirstNullifier, noteHashIndex);
      const note = new Note([new Fr(amount), ownerNpkMHash, Fr.random()]);
      const innerNoteHash = pedersenHash(note.items);
      return {
        contractAddress,
        storageSlot,
        noteTypeId,
        nonce,
        note,
        innerNoteHash,
        siloedNullifier: new Fr(0),
        index: currentNoteIndex++,
      };
    };

    beforeEach(() => {
      oracle.getFunctionArtifactByName.mockImplementation((_, functionName: string) =>
        Promise.resolve(getFunctionArtifact(StatefulTestContractArtifact, functionName)),
      );

      oracle.getFunctionArtifact.mockImplementation((_, selector: FunctionSelector) =>
        Promise.resolve(getFunctionArtifact(StatefulTestContractArtifact, selector)),
      );
    });

    it('should have a constructor with arguments that inserts notes', async () => {
      const initArgs = [owner, owner, 140];
      const instance = getContractInstanceFromDeployParams(StatefulTestContractArtifact, { constructorArgs: initArgs });
      oracle.getContractInstance.mockResolvedValue(instance);
      const artifact = getFunctionArtifact(StatefulTestContractArtifact, 'constructor');
      const topLevelResult = await runSimulator({ args: initArgs, artifact, contractAddress: instance.address });
      const result = topLevelResult.nestedExecutions[0];

      expect(result.newNotes).toHaveLength(1);
      const newNote = result.newNotes[0];
      expect(newNote.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner));
      expect(newNote.noteTypeId).toEqual(valueNoteTypeId); // ValueNote

      const noteHashes = getNonEmptyItems(result.callStackItem.publicInputs.noteHashes);
      expect(noteHashes).toHaveLength(1);
      expect(noteHashes[0].value).toEqual(
        await acirSimulator.computeInnerNoteHash(
          contractAddress,
          newNote.storageSlot,
          newNote.noteTypeId,
          newNote.note,
        ),
      );

      const newEncryptedLogs = getNonEmptyItems(result.callStackItem.publicInputs.noteEncryptedLogsHashes);
      expect(newEncryptedLogs).toHaveLength(1);

      const [encryptedLog] = newEncryptedLogs;
      expect(encryptedLog.noteHashCounter).toEqual(noteHashes[0].counter);
      expect(encryptedLog.value).toEqual(Fr.fromBuffer(result.noteEncryptedLogs[0].log.hash()));
      expect(encryptedLog.length).toEqual(new Fr(getEncryptedNoteSerializedLength(result)));
    });

    it('should run the create_note function', async () => {
      const artifact = getFunctionArtifact(StatefulTestContractArtifact, 'create_note_no_init_check');

      const result = await runSimulator({ args: [owner, owner, 140], artifact });

      expect(result.newNotes).toHaveLength(1);
      const newNote = result.newNotes[0];
      expect(newNote.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner));
      expect(newNote.noteTypeId).toEqual(valueNoteTypeId); // ValueNote

      const noteHashes = getNonEmptyItems(result.callStackItem.publicInputs.noteHashes);
      expect(noteHashes).toHaveLength(1);
      expect(noteHashes[0].value).toEqual(
        await acirSimulator.computeInnerNoteHash(
          contractAddress,
          newNote.storageSlot,
          newNote.noteTypeId,
          newNote.note,
        ),
      );

      const newEncryptedLogs = getNonEmptyItems(result.callStackItem.publicInputs.noteEncryptedLogsHashes);
      expect(newEncryptedLogs).toHaveLength(1);

      const [encryptedLog] = newEncryptedLogs;
      expect(encryptedLog.noteHashCounter).toEqual(noteHashes[0].counter);
      expect(encryptedLog.value).toEqual(Fr.fromBuffer(result.noteEncryptedLogs[0].log.hash()));
      expect(encryptedLog.length).toEqual(new Fr(getEncryptedNoteSerializedLength(result)));
    });

    it('should run the destroy_and_create function', async () => {
      const amountToTransfer = 100n;
      const artifact = getFunctionArtifact(StatefulTestContractArtifact, 'destroy_and_create_no_init_check');

      const storageSlot = computeSlotForMapping(StatefulTestContractArtifact.storageLayout['notes'].slot, owner);
      const recipientStorageSlot = computeSlotForMapping(
        StatefulTestContractArtifact.storageLayout['notes'].slot,
        recipient,
      );

      const notes = [
        buildNote(60n, ownerCompleteAddress.publicKeys.masterNullifierPublicKey.hash(), storageSlot, valueNoteTypeId),
        buildNote(80n, ownerCompleteAddress.publicKeys.masterNullifierPublicKey.hash(), storageSlot, valueNoteTypeId),
      ];
      oracle.getNotes.mockResolvedValue(notes);

      const consumedNotes = await asyncMap(notes, ({ nonce, note }) =>
        acirSimulator.computeNoteHashAndOptionallyANullifier(
          contractAddress,
          nonce,
          storageSlot,
          valueNoteTypeId,
          true,
          note,
        ),
      );
      await insertLeaves(consumedNotes.map(n => n.siloedNoteHash));

      const args = [recipient, amountToTransfer];
      const result = await runSimulator({ args, artifact, msgSender: owner });

      // The two notes were nullified
      const nullifiers = getNonEmptyItems(result.callStackItem.publicInputs.nullifiers).map(n => n.value);
      expect(nullifiers).toHaveLength(consumedNotes.length);
      expect(nullifiers).toEqual(expect.arrayContaining(consumedNotes.map(n => n.innerNullifier)));

      expect(result.newNotes).toHaveLength(2);
      const [changeNote, recipientNote] = result.newNotes;
      expect(recipientNote.storageSlot).toEqual(recipientStorageSlot);
      expect(recipientNote.noteTypeId).toEqual(valueNoteTypeId);

      const noteHashes = getNonEmptyItems(result.callStackItem.publicInputs.noteHashes);
      expect(noteHashes).toHaveLength(2);
      const [changeNoteHash, recipientNoteHash] = noteHashes;
      const [recipientInnerNoteHash, changeInnerNoteHash] = [
        await acirSimulator.computeInnerNoteHash(
          contractAddress,
          recipientStorageSlot,
          valueNoteTypeId,
          recipientNote.note,
        ),
        await acirSimulator.computeInnerNoteHash(contractAddress, storageSlot, valueNoteTypeId, changeNote.note),
      ];
      expect(recipientNoteHash.value).toEqual(recipientInnerNoteHash);
      expect(changeNoteHash.value).toEqual(changeInnerNoteHash);

      expect(recipientNote.note.items[0]).toEqual(new Fr(amountToTransfer));
      expect(changeNote.note.items[0]).toEqual(new Fr(40n));

      const newEncryptedLogs = getNonEmptyItems(result.callStackItem.publicInputs.noteEncryptedLogsHashes);
      expect(newEncryptedLogs).toHaveLength(2);

      const [encryptedChangeLog, encryptedRecipientLog] = newEncryptedLogs;
      expect(encryptedChangeLog.value).toEqual(Fr.fromBuffer(result.noteEncryptedLogs[0].log.hash()));
      expect(encryptedChangeLog.noteHashCounter).toEqual(changeNoteHash.counter);
      expect(encryptedRecipientLog.value).toEqual(Fr.fromBuffer(result.noteEncryptedLogs[1].log.hash()));
      expect(encryptedRecipientLog.noteHashCounter).toEqual(recipientNoteHash.counter);
      expect(encryptedChangeLog.length.add(encryptedRecipientLog.length)).toEqual(
        new Fr(getEncryptedNoteSerializedLength(result)),
      );

      const readRequests = getNonEmptyItems(result.callStackItem.publicInputs.noteHashReadRequests).map(r => r.value);
      expect(readRequests).toHaveLength(consumedNotes.length);
      expect(readRequests).toEqual(expect.arrayContaining(consumedNotes.map(n => n.uniqueNoteHash)));
    });

    it('should be able to destroy_and_create with dummy notes', async () => {
      const amountToTransfer = 100n;
      const balance = 160n;
      const artifact = getFunctionArtifact(StatefulTestContractArtifact, 'destroy_and_create_no_init_check');

      const storageSlot = computeSlotForMapping(new Fr(1n), owner);

      const notes = [
        buildNote(
          balance,
          ownerCompleteAddress.publicKeys.masterNullifierPublicKey.hash(),
          storageSlot,
          valueNoteTypeId,
        ),
      ];
      oracle.getNotes.mockResolvedValue(notes);

      const consumedNotes = await asyncMap(notes, ({ nonce, note }) =>
        acirSimulator.computeNoteHashAndOptionallyANullifier(
          contractAddress,
          nonce,
          storageSlot,
          valueNoteTypeId,
          true,
          note,
        ),
      );
      await insertLeaves(consumedNotes.map(n => n.siloedNoteHash));

      const args = [recipient, amountToTransfer];
      const result = await runSimulator({ args, artifact, msgSender: owner });

      const nullifiers = getNonEmptyItems(result.callStackItem.publicInputs.nullifiers).map(n => n.value);
      expect(nullifiers).toEqual(consumedNotes.map(n => n.innerNullifier));

      expect(result.newNotes).toHaveLength(2);
      const [changeNote, recipientNote] = result.newNotes;
      expect(recipientNote.note.items[0]).toEqual(new Fr(amountToTransfer));
      expect(changeNote.note.items[0]).toEqual(new Fr(balance - amountToTransfer));

      const newEncryptedLogs = getNonEmptyItems(result.callStackItem.publicInputs.noteEncryptedLogsHashes);
      expect(newEncryptedLogs).toHaveLength(2);
      const [encryptedChangeLog, encryptedRecipientLog] = newEncryptedLogs;
      expect(encryptedChangeLog.value).toEqual(Fr.fromBuffer(result.noteEncryptedLogs[0].log.hash()));
      expect(encryptedChangeLog.noteHashCounter).toEqual(result.callStackItem.publicInputs.noteHashes[0].counter);
      expect(encryptedRecipientLog.value).toEqual(Fr.fromBuffer(result.noteEncryptedLogs[1].log.hash()));
      expect(encryptedRecipientLog.noteHashCounter).toEqual(result.callStackItem.publicInputs.noteHashes[1].counter);
      expect(encryptedChangeLog.length.add(encryptedRecipientLog.length)).toEqual(
        new Fr(getEncryptedNoteSerializedLength(result)),
      );
    });
  });

  describe('nested calls', () => {
    const privateIncrement = txContextFields.chainId.value + txContextFields.version.value;

    it('child function should be callable', async () => {
      const initialValue = 100n;
      const artifact = getFunctionArtifact(ChildContractArtifact, 'value');
      const result = await runSimulator({ args: [initialValue], artifact });

      expect(result.returnValues).toEqual([new Fr(initialValue + privateIncrement)]);
    });

    it('parent should call child', async () => {
      const childArtifact = getFunctionArtifact(ChildContractArtifact, 'value');
      const parentArtifact = getFunctionArtifact(ParentContractArtifact, 'entry_point');
      const parentAddress = AztecAddress.random();
      const childAddress = AztecAddress.random();
      const childSelector = FunctionSelector.fromNameAndParameters(childArtifact.name, childArtifact.parameters);

      oracle.getFunctionArtifact.mockImplementation(() => Promise.resolve(childArtifact));

      logger.info(`Parent deployed at ${parentAddress.toShortString()}`);
      logger.info(`Calling child function ${childSelector.toString()} at ${childAddress.toShortString()}`);

      const args = [childAddress, childSelector];
      const result = await runSimulator({ args, artifact: parentArtifact });

      expect(result.returnValues).toEqual([new Fr(privateIncrement)]);

      expect(oracle.getFunctionArtifact.mock.calls[0]).toEqual([childAddress, childSelector]);
      expect(result.nestedExecutions).toHaveLength(1);
      expect(result.nestedExecutions[0].returnValues).toEqual([new Fr(privateIncrement)]);

      // check that Aztec.nr calculated the call stack item hash like cpp does
      expect(
        result.callStackItem.publicInputs.privateCallRequests[0].matchesStackItem(
          result.nestedExecutions[0].callStackItem,
        ),
      ).toBeTruthy();
    });
  });

  describe('nested calls through autogenerated interface', () => {
    let args: any[];
    let argsHash: Fr;
    let testCodeGenArtifact: FunctionArtifact;

    beforeAll(() => {
      // These args should match the ones hardcoded in importer contract
      // eslint-disable-next-line camelcase
      const dummyNote = { amount: 1, secret_hash: 2 };
      // eslint-disable-next-line camelcase
      const deepStruct = { a_field: 1, a_bool: true, a_note: dummyNote, many_notes: [dummyNote, dummyNote, dummyNote] };
      args = [1, true, 1, [1, 2], dummyNote, deepStruct];
      testCodeGenArtifact = getFunctionArtifact(TestContractArtifact, 'test_code_gen');
      const serializedArgs = encodeArguments(testCodeGenArtifact, args);
      argsHash = computeVarArgsHash(serializedArgs);
    });

    it('test function should be directly callable', async () => {
      logger.info(`Calling testCodeGen function`);
      const result = await runSimulator({ args, artifact: testCodeGenArtifact });

      expect(result.returnValues).toEqual([argsHash]);
    });

    it('test function should be callable through autogenerated interface', async () => {
      const testAddress = AztecAddress.random();
      const parentArtifact = getFunctionArtifact(ImportTestContractArtifact, 'main_contract');
      const testCodeGenSelector = FunctionSelector.fromNameAndParameters(
        testCodeGenArtifact.name,
        testCodeGenArtifact.parameters,
      );

      oracle.getFunctionArtifact.mockResolvedValue(testCodeGenArtifact);

      logger.info(`Calling importer main function`);
      const args = [testAddress];
      const result = await runSimulator({ args, artifact: parentArtifact });

      expect(result.returnValues).toEqual([argsHash]);
      expect(oracle.getFunctionArtifact.mock.calls[0]).toEqual([testAddress, testCodeGenSelector]);
      expect(result.nestedExecutions).toHaveLength(1);
      expect(result.nestedExecutions[0].returnValues).toEqual([argsHash]);
    });
  });

  describe('consuming messages', () => {
    const contractAddress = defaultContractAddress;

    describe('L1 to L2', () => {
      const artifact = getFunctionArtifact(TestContractArtifact, 'consume_mint_private_message');
      let bridgedAmount = 100n;

      const secretHashForRedeemingNotes = new Fr(2n);
      let secretForL1ToL2MessageConsumption = new Fr(1n);

      let crossChainMsgRecipient: AztecAddress | undefined;
      let crossChainMsgSender: EthAddress | undefined;

      let preimage: L1ToL2Message;

      let args: Fr[];

      beforeEach(() => {
        bridgedAmount = 100n;
        secretForL1ToL2MessageConsumption = new Fr(2n);

        crossChainMsgRecipient = undefined;
        crossChainMsgSender = undefined;
      });

      const computePreimage = () =>
        buildL1ToL2Message(
          toFunctionSelector('mint_private(bytes32,uint256)').substring(2),
          [secretHashForRedeemingNotes, new Fr(bridgedAmount)],
          crossChainMsgRecipient ?? contractAddress,
          secretForL1ToL2MessageConsumption,
        );

      const computeArgs = () =>
        encodeArguments(artifact, [
          secretHashForRedeemingNotes,
          bridgedAmount,
          secretForL1ToL2MessageConsumption,
          crossChainMsgSender ?? preimage.sender.sender,
        ]);

      const mockOracles = async (updateHeader = true) => {
        const tree = await insertLeaves([preimage.hash()], 'l1ToL2Messages');
        oracle.getL1ToL2MembershipWitness.mockImplementation(async () => {
          return Promise.resolve(new MessageLoadOracleInputs(0n, await tree.getSiblingPath(0n, true)));
        });
        if (updateHeader) {
          oracle.getHeader.mockResolvedValue(header);
        }
      };

      it('Should be able to consume a dummy cross chain message', async () => {
        preimage = computePreimage();
        args = computeArgs();
        await mockOracles();

        const result = await runSimulator({
          contractAddress,
          artifact,
          args,
          txContext: { version: new Fr(1n), chainId: new Fr(1n) },
        });

        // Check a nullifier has been inserted
        const nullifiers = getNonEmptyItems(result.callStackItem.publicInputs.nullifiers);
        expect(nullifiers).toHaveLength(1);
      });

      it('Invalid membership proof', async () => {
        preimage = computePreimage();

        args = computeArgs();

        // Don't update the header so the message is not in state
        await mockOracles(false);

        await expect(
          runSimulator({
            contractAddress,
            artifact,
            args,
            txContext: { version: new Fr(1n), chainId: new Fr(1n) },
          }),
        ).rejects.toThrow('Message not in state');
      });

      it('Invalid recipient', async () => {
        crossChainMsgRecipient = AztecAddress.random();

        preimage = computePreimage();

        args = computeArgs();

        await mockOracles();
        // Update state
        oracle.getHeader.mockResolvedValue(header);

        await expect(
          runSimulator({
            contractAddress,
            artifact,
            args,
            txContext: { version: new Fr(1n), chainId: new Fr(1n) },
          }),
        ).rejects.toThrow('Message not in state');
      });

      it('Invalid sender', async () => {
        crossChainMsgSender = EthAddress.random();
        preimage = computePreimage();

        args = computeArgs();

        await mockOracles();
        // Update state
        oracle.getHeader.mockResolvedValue(header);

        await expect(
          runSimulator({
            contractAddress,
            artifact,
            args,
            txContext: { version: new Fr(1n), chainId: new Fr(1n) },
          }),
        ).rejects.toThrow('Message not in state');
      });

      it('Invalid chainid', async () => {
        preimage = computePreimage();

        args = computeArgs();

        await mockOracles();
        // Update state
        oracle.getHeader.mockResolvedValue(header);

        await expect(
          runSimulator({
            contractAddress,
            artifact,
            args,
            txContext: { version: new Fr(1n), chainId: new Fr(2n) },
          }),
        ).rejects.toThrow('Message not in state');
      });

      it('Invalid version', async () => {
        preimage = computePreimage();

        args = computeArgs();

        await mockOracles();
        // Update state
        oracle.getHeader.mockResolvedValue(header);

        await expect(
          runSimulator({
            contractAddress,
            artifact,
            args,
            txContext: { version: new Fr(2n), chainId: new Fr(1n) },
          }),
        ).rejects.toThrow('Message not in state');
      });

      it('Invalid content', async () => {
        preimage = computePreimage();

        bridgedAmount = bridgedAmount + 1n; // Invalid amount
        args = computeArgs();

        await mockOracles();
        // Update state
        oracle.getHeader.mockResolvedValue(header);

        await expect(
          runSimulator({
            contractAddress,
            artifact,
            args,
            txContext: { version: new Fr(1n), chainId: new Fr(1n) },
          }),
        ).rejects.toThrow('Message not in state');
      });

      it('Invalid Secret', async () => {
        preimage = computePreimage();

        secretForL1ToL2MessageConsumption = Fr.random();
        args = computeArgs();

        await mockOracles();
        // Update state
        oracle.getHeader.mockResolvedValue(header);

        await expect(
          runSimulator({
            contractAddress,
            artifact,
            args,
            txContext: { version: new Fr(1n), chainId: new Fr(1n) },
          }),
        ).rejects.toThrow('Message not in state');
      });
    });

    it('Should be able to consume a dummy public to private message', async () => {
      const artifact = getFunctionArtifact(TestContractArtifact, 'consume_note_from_secret');
      const secret = new Fr(1n);
      const secretHash = computeSecretHash(secret);
      const note = new Note([secretHash]);
      const storageSlot = TestContractArtifact.storageLayout['example_set'].slot;
      oracle.getNotes.mockResolvedValue([
        {
          contractAddress,
          storageSlot,
          nonce: Fr.ZERO,
          note,
          innerNoteHash: Fr.ZERO,
          siloedNullifier: Fr.random(),
          index: 1n,
        },
      ]);

      const result = await runSimulator({ artifact, args: [secret] });

      // Check a nullifier has been inserted.
      const nullifiers = getNonEmptyItems(result.callStackItem.publicInputs.nullifiers);
      expect(nullifiers).toHaveLength(1);

      // Check the commitment read request was created successfully.
      const readRequests = getNonEmptyItems(result.callStackItem.publicInputs.noteHashReadRequests);
      expect(readRequests).toHaveLength(1);
    });
  });

  describe('enqueued calls', () => {
    it.each([false, true])('parent should enqueue call to child (internal %p)', async isInternal => {
      const parentArtifact = getFunctionArtifact(ParentContractArtifact, 'enqueue_call_to_child');
      const childContractArtifact = ChildContractArtifact.functions.find(fn => fn.name === 'pub_set_value')!;
      expect(childContractArtifact).toBeDefined();
      const childAddress = AztecAddress.random();
      const childSelector = FunctionSelector.fromNameAndParameters(
        childContractArtifact.name,
        childContractArtifact.parameters,
      );
      const parentAddress = AztecAddress.random();

      oracle.getFunctionArtifact.mockImplementation(() => Promise.resolve({ ...childContractArtifact, isInternal }));

      const args = [childAddress, childSelector, 42n];
      const result = await runSimulator({
        msgSender: parentAddress,
        contractAddress: parentAddress,
        artifact: parentArtifact,
        args,
      });

      // Alter function data to match the manipulated oracle
      const functionSelector = FunctionSelector.fromNameAndParameters(
        childContractArtifact.name,
        childContractArtifact.parameters,
      );

      const publicCallRequest = PublicCallRequest.from({
        contractAddress: childAddress,
        functionSelector,
        args: [new Fr(42n)],
        callContext: CallContext.from({
          msgSender: parentAddress,
          storageContractAddress: childAddress,
          functionSelector: childSelector,
          isDelegateCall: false,
          isStaticCall: false,
        }),
        parentCallContext: CallContext.from({
          msgSender: parentAddress,
          storageContractAddress: parentAddress,
          functionSelector: FunctionSelector.fromNameAndParameters(parentArtifact.name, parentArtifact.parameters),
          isDelegateCall: false,
          isStaticCall: false,
        }),
        sideEffectCounter: 2,
      });

      const publicCallRequestHash = publicCallRequest.toPublicCallStackItem().getCompressed().hash();

      expect(result.enqueuedPublicFunctionCalls).toHaveLength(1);
      expect(result.enqueuedPublicFunctionCalls[0]).toEqual(publicCallRequest);
      expect(result.callStackItem.publicInputs.publicCallStackHashes[0]).toEqual(publicCallRequestHash);
    });
  });

  describe('setting teardown function', () => {
    it('should be able to set a teardown function', async () => {
      const entrypoint = getFunctionArtifact(TestContractArtifact, 'test_setting_teardown');
      const teardown = getFunctionArtifact(TestContractArtifact, 'dummy_public_call');
      oracle.getFunctionArtifact.mockImplementation(() => Promise.resolve({ ...teardown }));
      const result = await runSimulator({ artifact: entrypoint });
      expect(result.publicTeardownFunctionCall.isEmpty()).toBeFalsy();
      expect(result.publicTeardownFunctionCall.functionSelector).toEqual(
        FunctionSelector.fromNameAndParameters(teardown.name, teardown.parameters),
      );
    });
  });

  describe('setting fee payer', () => {
    it('should default to not being a fee payer', async () => {
      // arbitrary random function that doesn't set a fee payer
      const entrypoint = getFunctionArtifact(TestContractArtifact, 'get_this_address');
      const contractAddress = AztecAddress.random();
      const result = await runSimulator({ artifact: entrypoint, contractAddress });
      expect(result.callStackItem.publicInputs.isFeePayer).toBe(false);
    });

    it('should be able to set a fee payer', async () => {
      const entrypoint = getFunctionArtifact(TestContractArtifact, 'test_setting_fee_payer');
      const contractAddress = AztecAddress.random();
      const result = await runSimulator({ artifact: entrypoint, contractAddress });
      expect(result.callStackItem.publicInputs.isFeePayer).toBe(true);
    });
  });

  describe('pending note hashes contract', () => {
    const valueNoteTypeId = PendingNoteHashesContractArtifact.notes['ValueNote'].id;

    beforeEach(() => {
      oracle.getFunctionArtifact.mockImplementation((_, selector) =>
        Promise.resolve(getFunctionArtifact(PendingNoteHashesContractArtifact, selector)),
      );
      oracle.getFunctionArtifactByName.mockImplementation((_, functionName: string) =>
        Promise.resolve(getFunctionArtifact(PendingNoteHashesContractArtifact, functionName)),
      );
    });

    it('should be able to insert, read, and nullify pending note hashes in one call', async () => {
      oracle.getNotes.mockResolvedValue([]);

      const amountToTransfer = 100n;

      const contractAddress = AztecAddress.random();
      const artifact = getFunctionArtifact(PendingNoteHashesContractArtifact, 'test_insert_then_get_then_nullify_flat');

      const outgoingViewer = owner;
      const args = [amountToTransfer, owner, outgoingViewer];
      const result = await runSimulator({
        args: args,
        artifact: artifact,
        contractAddress,
      });

      expect(result.newNotes).toHaveLength(1);
      const noteAndSlot = result.newNotes[0];
      expect(noteAndSlot.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner));

      expect(noteAndSlot.note.items[0]).toEqual(new Fr(amountToTransfer));

      const noteHashes = getNonEmptyItems(result.callStackItem.publicInputs.noteHashes);
      expect(noteHashes).toHaveLength(1);

      const noteHash = noteHashes[0].value;
      const storageSlot = computeSlotForMapping(
        PendingNoteHashesContractArtifact.storageLayout['balances'].slot,
        owner,
      );

      const innerNoteHash = await acirSimulator.computeInnerNoteHash(
        contractAddress,
        storageSlot,
        valueNoteTypeId,
        noteAndSlot.note,
      );
      expect(noteHash).toEqual(innerNoteHash);

      const newEncryptedLogs = getNonEmptyItems(result.callStackItem.publicInputs.noteEncryptedLogsHashes);
      expect(newEncryptedLogs).toHaveLength(1);

      const [encryptedLog] = newEncryptedLogs;
      expect(encryptedLog.noteHashCounter).toEqual(noteHashes[0].counter);
      expect(encryptedLog.noteHashCounter).toEqual(result.noteEncryptedLogs[0].noteHashCounter);
      expect(encryptedLog.value).toEqual(Fr.fromBuffer(result.noteEncryptedLogs[0].log.hash()));

      // read request should match innerNoteHash for pending notes (there is no nonce, so can't compute "unique" hash)
      const readRequest = getNonEmptyItems(result.callStackItem.publicInputs.noteHashReadRequests)[0];
      expect(readRequest.value).toEqual(innerNoteHash);

      expect(result.returnValues).toEqual([new Fr(amountToTransfer)]);

      const nullifier = result.callStackItem.publicInputs.nullifiers[0];
      const expectedNullifier = poseidon2Hash([
        innerNoteHash,
        computeAppNullifierSecretKey(ownerNskM, contractAddress),
        GeneratorIndex.NOTE_NULLIFIER,
      ]);
      expect(nullifier.value).toEqual(expectedNullifier);
    });

    it('should be able to insert, read, and nullify pending note hashes in nested calls', async () => {
      oracle.getNotes.mockResolvedValue([]);

      const amountToTransfer = 100n;

      const contractAddress = AztecAddress.random();
      const artifact = getFunctionArtifact(
        PendingNoteHashesContractArtifact,
        'test_insert_then_get_then_nullify_all_in_nested_calls',
      );
      const insertArtifact = getFunctionArtifact(PendingNoteHashesContractArtifact, 'insert_note');

      const getThenNullifyArtifact = getFunctionArtifact(PendingNoteHashesContractArtifact, 'get_then_nullify_note');

      const insertFnSelector = FunctionSelector.fromNameAndParameters(insertArtifact.name, insertArtifact.parameters);
      const getThenNullifyFnSelector = FunctionSelector.fromNameAndParameters(
        getThenNullifyArtifact.name,
        getThenNullifyArtifact.parameters,
      );

      const outgoingViewer = owner;
      const args = [
        amountToTransfer,
        owner,
        outgoingViewer,
        insertFnSelector.toField(),
        getThenNullifyFnSelector.toField(),
      ];
      const result = await runSimulator({
        args: args,
        artifact: artifact,
        contractAddress: contractAddress,
      });

      const execInsert = result.nestedExecutions[0];
      const execGetThenNullify = result.nestedExecutions[1];

      const storageSlot = computeSlotForMapping(
        PendingNoteHashesContractArtifact.storageLayout['balances'].slot,
        owner,
      );

      expect(execInsert.newNotes).toHaveLength(1);
      const noteAndSlot = execInsert.newNotes[0];
      expect(noteAndSlot.storageSlot).toEqual(storageSlot);
      expect(noteAndSlot.noteTypeId).toEqual(valueNoteTypeId);

      expect(noteAndSlot.note.items[0]).toEqual(new Fr(amountToTransfer));

      const noteHashes = getNonEmptyItems(execInsert.callStackItem.publicInputs.noteHashes);
      expect(noteHashes).toHaveLength(1);

      const noteHash = noteHashes[0].value;
      const innerNoteHash = await acirSimulator.computeInnerNoteHash(
        contractAddress,
        noteAndSlot.storageSlot,
        noteAndSlot.noteTypeId,
        noteAndSlot.note,
      );
      expect(noteHash).toEqual(innerNoteHash);

      const newEncryptedLogs = getNonEmptyItems(execInsert.callStackItem.publicInputs.noteEncryptedLogsHashes);
      expect(newEncryptedLogs).toHaveLength(1);

      const [encryptedLog] = newEncryptedLogs;
      expect(encryptedLog.noteHashCounter).toEqual(noteHashes[0].counter);
      expect(encryptedLog.noteHashCounter).toEqual(execInsert.noteEncryptedLogs[0].noteHashCounter);
      expect(encryptedLog.value).toEqual(Fr.fromBuffer(execInsert.noteEncryptedLogs[0].log.hash()));

      // read request should match innerNoteHash for pending notes (there is no nonce, so can't compute "unique" hash)
      const readRequest = execGetThenNullify.callStackItem.publicInputs.noteHashReadRequests[0];
      expect(readRequest.value).toEqual(innerNoteHash);

      expect(execGetThenNullify.returnValues).toEqual([new Fr(amountToTransfer)]);

      const nullifier = execGetThenNullify.callStackItem.publicInputs.nullifiers[0];
      const expectedNullifier = poseidon2Hash([
        innerNoteHash,
        computeAppNullifierSecretKey(ownerNskM, contractAddress),
        GeneratorIndex.NOTE_NULLIFIER,
      ]);
      expect(nullifier.value).toEqual(expectedNullifier);
    });

    it('cant read a commitment that is inserted later in same call', async () => {
      oracle.getNotes.mockResolvedValue([]);

      const amountToTransfer = 100n;

      const contractAddress = AztecAddress.random();

      const artifact = getFunctionArtifact(PendingNoteHashesContractArtifact, 'test_bad_get_then_insert_flat');

      const args = [amountToTransfer, owner];
      await expect(() =>
        runSimulator({
          args: args,
          artifact: artifact,
          contractAddress,
        }),
      ).rejects.toThrow(`Assertion failed: Cannot return zero notes`);
    });
  });

  describe('get master incoming viewing public key', () => {
    it('gets the public key for an address', async () => {
      // Tweak the contract artifact so we can extract return values
      const artifact = getFunctionArtifact(TestContractArtifact, 'get_master_incoming_viewing_public_key');

      // Generate a partial address, pubkey, and resulting address
      const completeAddress = CompleteAddress.random();
      const args = [completeAddress.address];
      const pubKey = completeAddress.publicKeys.masterIncomingViewingPublicKey;

      oracle.getCompleteAddress.mockResolvedValue(completeAddress);
      const result = await runSimulator({ artifact, args });
      expect(result.returnValues).toEqual([pubKey.x, pubKey.y]);
    });
  });

  describe('Get notes', () => {
    it('fails if returning no notes', async () => {
      const artifact = getFunctionArtifact(TestContractArtifact, 'call_get_notes');

      const args = [2n, true];
      oracle.getNotes.mockResolvedValue([]);

      await expect(() => runSimulator({ artifact, args })).rejects.toThrow(
        `Assertion failed: Cannot return zero notes`,
      );
    });
  });

  describe('Context oracles', () => {
    it('this_address should return the current context address', async () => {
      const contractAddress = AztecAddress.random();

      // Tweak the contract artifact so we can extract return values
      const artifact = getFunctionArtifact(TestContractArtifact, 'get_this_address');

      // Overwrite the oracle return value
      const result = await runSimulator({ artifact, args: [], contractAddress });
      expect(result.returnValues).toEqual([contractAddress.toField()]);
    });
  });

  describe('Private global variables', () => {
    let chainId: Fr;
    let version: Fr;
    let args: any[];
    let artifact: FunctionArtifact;

    beforeEach(() => {
      chainId = Fr.random();
      version = Fr.random();
      args = [chainId, version];

      artifact = getFunctionArtifact(TestContractArtifact, 'assert_private_global_vars');
      oracle.getFunctionArtifact.mockImplementation(() => Promise.resolve(artifact));
    });

    it('Private global vars are correctly set', async () => {
      // Chain id and version set in tx context is the same as the ones we pass via args so this should not throw
      await runSimulator({ artifact, msgSender: owner, args, txContext: { chainId, version } });
    });

    it('Throws when chainId is incorrectly set', async () => {
      // We set the chainId in the tx context to a different value than the one we pass via args so the simulator should throw
      const unexpectedChainId = Fr.random();
      await expect(() =>
        runSimulator({ artifact, msgSender: owner, args, txContext: { chainId: unexpectedChainId, version } }),
      ).rejects.toThrow('Invalid chain id');
    });

    it('Throws when version is incorrectly set', async () => {
      // We set the version in the tx context to a different value than the one we pass via args so the simulator should throw
      const unexpectedVersion = Fr.random();
      await expect(() =>
        runSimulator({ artifact, msgSender: owner, args, txContext: { chainId, version: unexpectedVersion } }),
      ).rejects.toThrow('Invalid version');
    });
  });

  describe('Historical header in private context', () => {
    let artifact: FunctionArtifact;

    beforeEach(() => {
      artifact = getFunctionArtifact(TestContractArtifact, 'assert_header_private');
      oracle.getFunctionArtifact.mockImplementation(() => Promise.resolve(artifact));

      header = makeHeader();

      oracle.getHeader.mockClear();
      oracle.getHeader.mockResolvedValue(header);
    });

    it('Header is correctly set', async () => {
      const args = [header.hash()];

      await runSimulator({ artifact, msgSender: owner, args });
    });

    it('Throws when header is not as expected', async () => {
      const unexpectedHeaderHash = Fr.random();
      const args = [unexpectedHeaderHash];

      await expect(() => runSimulator({ artifact, msgSender: owner, args })).rejects.toThrow('Invalid header hash');
    });
  });
});
