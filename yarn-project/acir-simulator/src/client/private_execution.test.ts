import {
  CallContext,
  CircuitsWasm,
  CompleteAddress,
  ContractDeploymentData,
  FieldsOf,
  FunctionData,
  HistoricBlockData,
  L1_TO_L2_MSG_TREE_HEIGHT,
  MAX_NEW_COMMITMENTS_PER_CALL,
  PRIVATE_DATA_TREE_HEIGHT,
  PrivateKey,
  PublicCallRequest,
  TxContext,
} from '@aztec/circuits.js';
import {
  computeCallStackItemHash,
  computeCommitmentNonce,
  computeSecretMessageHash,
  computeUniqueCommitment,
  computeVarArgsHash,
  siloCommitment,
} from '@aztec/circuits.js/abis';
import { pedersenPlookupCommitInputs } from '@aztec/circuits.js/barretenberg';
import { makeContractDeploymentData } from '@aztec/circuits.js/factories';
import { FunctionAbi, encodeArguments, generateFunctionSelector } from '@aztec/foundation/abi';
import { asyncMap } from '@aztec/foundation/async-map';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { AppendOnlyTree, Pedersen, StandardTree, newTree } from '@aztec/merkle-tree';
import {
  ChildContractAbi,
  ImportTestContractAbi,
  NonNativeTokenContractAbi,
  ParentContractAbi,
  PendingCommitmentsContractAbi,
  PrivateTokenAirdropContractAbi,
  PrivateTokenContractAbi,
  TestContractAbi,
} from '@aztec/noir-contracts/artifacts';
import { PackedArguments, TxExecutionRequest } from '@aztec/types';

import { jest } from '@jest/globals';
import { MockProxy, mock } from 'jest-mock-extended';
import { default as levelup } from 'levelup';
import { type MemDown, default as memdown } from 'memdown';

import { buildL1ToL2Message } from '../test/utils.js';
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
  const ownerPk = PrivateKey.fromString('5e30a2f886b4b6a11aea03bf4910fbd5b24e61aa27ea4d05c393b3ab592a8d33');

  const treeHeights: { [name: string]: number } = {
    privateData: PRIVATE_DATA_TREE_HEIGHT,
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
    abi,
    args = [],
    origin = AztecAddress.random(),
    contractAddress = defaultContractAddress,
    portalContractAddress = EthAddress.ZERO,
    txContext = {},
  }: {
    abi: FunctionAbi;
    origin?: AztecAddress;
    contractAddress?: AztecAddress;
    portalContractAddress?: EthAddress;
    args?: any[];
    txContext?: Partial<FieldsOf<TxContext>>;
  }) => {
    const packedArguments = await PackedArguments.fromArgs(encodeArguments(abi, args), circuitsWasm);
    const functionData = FunctionData.fromAbi(abi);
    const txRequest = TxExecutionRequest.from({
      origin,
      argsHash: packedArguments.hash,
      functionData,
      txContext: TxContext.from({ ...txContextFields, ...txContext }),
      packedArguments: [packedArguments],
    });

    return acirSimulator.run(
      txRequest,
      abi,
      functionData.isConstructor ? AztecAddress.ZERO : contractAddress,
      portalContractAddress,
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
    const prevRoots = blockData.toBuffer();
    const rootIndex = name === 'privateData' ? 0 : 32 * 3;
    const newRoots = Buffer.concat([prevRoots.subarray(0, rootIndex), newRoot, prevRoots.subarray(rootIndex + 32)]);
    blockData = HistoricBlockData.fromBuffer(newRoots);

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
    oracle.getHistoricBlockData.mockResolvedValue(blockData);

    acirSimulator = new AcirSimulator(oracle);
  });

  describe('empty constructor', () => {
    it('should run the empty constructor', async () => {
      const abi = TestContractAbi.functions[0];
      const contractDeploymentData = makeContractDeploymentData(100);
      const txContext = { isContractDeploymentTx: true, contractDeploymentData };
      const result = await runSimulator({ abi, txContext });

      const emptyCommitments = new Array(MAX_NEW_COMMITMENTS_PER_CALL).fill(Fr.ZERO);
      expect(result.callStackItem.publicInputs.newCommitments).toEqual(emptyCommitments);
      expect(result.callStackItem.publicInputs.contractDeploymentData).toEqual(contractDeploymentData);
    });
  });

  describe('private token airdrop contract', () => {
    const contractAddress = defaultContractAddress;
    const recipientPk = PrivateKey.fromString('0c9ed344548e8f9ba8aa3c9f8651eaa2853130f6c1e9c050ccf198f7ea18a7ec');
    const mockFirstNullifier = new Fr(1111);
    let owner: AztecAddress;
    let recipient: AztecAddress;
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
      return {
        contractAddress,
        storageSlot,
        nonce,
        preimage,
        siloedNullifier: new Fr(0),
        index: currentNoteIndex++,
      };
    };

    beforeEach(async () => {
      const ownerCompleteAddress = await CompleteAddress.fromPrivateKey(ownerPk);
      const recipientCompleteAddress = await CompleteAddress.fromPrivateKey(recipientPk);

      owner = ownerCompleteAddress.address;
      recipient = recipientCompleteAddress.address;

      oracle.getCompleteAddress.mockImplementation((address: AztecAddress) => {
        if (address.equals(owner)) return Promise.resolve(ownerCompleteAddress);
        if (address.equals(recipient)) return Promise.resolve(recipientCompleteAddress);
        throw new Error(`Unknown address ${address}`);
      });

      oracle.getFunctionABI.mockImplementation((_, selector) =>
        Promise.resolve(
          PrivateTokenAirdropContractAbi.functions.find(f =>
            selector.equals(generateFunctionSelector(f.name, f.parameters)),
          )!,
        ),
      );
    });

    it('should have an abi for computing note hash and nullifier', async () => {
      const storageSlot = Fr.random();
      const note = buildNote(60n, owner, storageSlot);

      // Should be the same as how we compute the values for the ValueNote in the noir library.
      const valueNoteHash = pedersenPlookupCommitInputs(
        circuitsWasm,
        note.preimage.map(f => f.toBuffer()),
      );
      const innerNoteHash = Fr.fromBuffer(
        pedersenPlookupCommitInputs(circuitsWasm, [storageSlot.toBuffer(), valueNoteHash]),
      );
      const siloedNoteHash = siloCommitment(circuitsWasm, contractAddress, innerNoteHash);
      const uniqueSiloedNoteHash = computeUniqueCommitment(circuitsWasm, note.nonce, siloedNoteHash);
      const innerNullifier = Fr.fromBuffer(
        pedersenPlookupCommitInputs(circuitsWasm, [uniqueSiloedNoteHash.toBuffer(), ownerPk.value]),
      );

      const result = await acirSimulator.computeNoteHashAndNullifier(
        contractAddress,
        note.nonce,
        storageSlot,
        note.preimage,
      );

      expect(result).toEqual({
        innerNoteHash,
        siloedNoteHash,
        uniqueSiloedNoteHash,
        innerNullifier,
      });
    });

    it('should a constructor with arguments that inserts notes', async () => {
      const abi = PrivateTokenAirdropContractAbi.functions.find(f => f.name === 'constructor')!;

      const result = await runSimulator({ args: [140, owner], abi });

      expect(result.preimages.newNotes).toHaveLength(1);
      const newNote = result.preimages.newNotes[0];
      expect(newNote.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm));

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));
      expect(newCommitments).toHaveLength(1);

      const [commitment] = newCommitments;
      expect(commitment).toEqual(
        await acirSimulator.computeInnerNoteHash(contractAddress, newNote.storageSlot, newNote.preimage),
      );
    });

    it('should run the mint function', async () => {
      const abi = PrivateTokenAirdropContractAbi.functions.find(f => f.name === 'mint')!;

      const result = await runSimulator({ args: [140, owner], abi });

      expect(result.preimages.newNotes).toHaveLength(1);
      const newNote = result.preimages.newNotes[0];
      expect(newNote.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm));

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));
      expect(newCommitments).toHaveLength(1);

      const [commitment] = newCommitments;
      expect(commitment).toEqual(
        await acirSimulator.computeInnerNoteHash(contractAddress, newNote.storageSlot, newNote.preimage),
      );
    });

    it('should run the transfer function', async () => {
      const amountToTransfer = 100n;
      const abi = PrivateTokenAirdropContractAbi.functions.find(f => f.name === 'transfer')!;

      const storageSlot = computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm);
      const recipientStorageSlot = computeSlotForMapping(new Fr(1n), recipient.toField(), circuitsWasm);

      const notes = [buildNote(60n, owner, storageSlot), buildNote(80n, owner, storageSlot)];
      oracle.getNotes.mockResolvedValue(notes);

      const consumedNotes = await asyncMap(notes, ({ nonce, preimage }) =>
        acirSimulator.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, preimage),
      );
      await insertLeaves(consumedNotes.map(n => n.siloedNoteHash));

      const args = [amountToTransfer, owner, recipient];
      const result = await runSimulator({ args, abi });

      // The two notes were nullified
      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toEqual(consumedNotes.map(n => n.innerNullifier));

      expect(result.preimages.newNotes).toHaveLength(2);
      const [changeNote, recipientNote] = result.preimages.newNotes;
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
      expect(readRequests).toEqual(consumedNotes.map(n => n.uniqueSiloedNoteHash));
    });

    it('should be able to transfer with dummy notes', async () => {
      const amountToTransfer = 100n;
      const balance = 160n;
      const abi = PrivateTokenAirdropContractAbi.functions.find(f => f.name === 'transfer')!;

      const storageSlot = computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm);

      const notes = [buildNote(balance, owner, storageSlot)];
      oracle.getNotes.mockResolvedValue(notes);

      const consumedNotes = await asyncMap(notes, ({ nonce, preimage }) =>
        acirSimulator.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, preimage),
      );
      await insertLeaves(consumedNotes.map(n => n.siloedNoteHash));

      const args = [amountToTransfer, owner, recipient];
      const result = await runSimulator({ args, abi });

      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toEqual(consumedNotes.map(n => n.innerNullifier));

      expect(result.preimages.newNotes).toHaveLength(2);
      const [changeNote, recipientNote] = result.preimages.newNotes;
      expect(recipientNote.preimage[0]).toEqual(new Fr(amountToTransfer));
      expect(changeNote.preimage[0]).toEqual(new Fr(balance - amountToTransfer));
    });

    it('Should be able to claim a note by providing the correct secret', async () => {
      const amount = 100n;
      const secret = Fr.random();
      const abi = PrivateTokenAirdropContractAbi.functions.find(f => f.name === 'claim')!;
      const storageSlot = new Fr(2n);
      // choose nonzero nonce otherwise reads will be interpreted as transient (inner note hash instead of unique+siloed)
      const nonce = new Fr(1n);

      oracle.getNotes.mockResolvedValue([
        {
          contractAddress,
          storageSlot,
          nonce,
          preimage: [new Fr(amount), secret],
          siloedNullifier: new Fr(0),
          index: 1n,
        },
      ]);

      const result = await runSimulator({
        abi,
        args: [amount, secret, recipient],
      });

      // Check a nullifier has been inserted.
      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toHaveLength(1);

      // Check the read request was inserted successfully.
      const readRequests = result.callStackItem.publicInputs.readRequests.filter(field => !field.equals(Fr.ZERO));
      const customNoteHash = hash([toBufferBE(amount, 32), secret.toBuffer()]);
      const innerNoteHash = Fr.fromBuffer(hash([storageSlot.toBuffer(), customNoteHash]));
      const siloedNoteHash = siloCommitment(circuitsWasm, contractAddress, innerNoteHash);
      const uniqueSiloedNoteHash = computeUniqueCommitment(circuitsWasm, nonce, siloedNoteHash);
      expect(readRequests).toEqual([uniqueSiloedNoteHash]);
    });
  });

  describe('private token contract', () => {
    const contractAddress = defaultContractAddress;
    const recipientPk = PrivateKey.fromString('0c9ed344548e8f9ba8aa3c9f8651eaa2853130f6c1e9c050ccf198f7ea18a7ec');
    const mockFirstNullifier = new Fr(1111);
    let owner: AztecAddress;
    let recipient: AztecAddress;
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
      return {
        contractAddress,
        storageSlot,
        nonce,
        preimage,
        siloedNullifier: new Fr(0),
        index: currentNoteIndex++,
      };
    };

    beforeEach(async () => {
      const ownerCompleteAddress = await CompleteAddress.fromPrivateKey(ownerPk);
      const recipientCompleteAddress = await CompleteAddress.fromPrivateKey(recipientPk);

      owner = ownerCompleteAddress.address;
      recipient = recipientCompleteAddress.address;

      oracle.getCompleteAddress.mockImplementation((address: AztecAddress) => {
        if (address.equals(owner)) return Promise.resolve(ownerCompleteAddress);
        if (address.equals(recipient)) return Promise.resolve(recipientCompleteAddress);
        throw new Error(`Unknown address ${address}`);
      });

      oracle.getFunctionABI.mockImplementation((_, selector) =>
        Promise.resolve(
          PrivateTokenContractAbi.functions.find(f => selector.equals(generateFunctionSelector(f.name, f.parameters)))!,
        ),
      );
    });

    it('should have an abi for computing note hash and nullifier', async () => {
      const storageSlot = Fr.random();
      const note = buildNote(60n, owner, storageSlot);

      // Should be the same as how we compute the values for the ValueNote in the noir library.
      const valueNoteHash = pedersenPlookupCommitInputs(
        circuitsWasm,
        note.preimage.map(f => f.toBuffer()),
      );
      const innerNoteHash = Fr.fromBuffer(
        pedersenPlookupCommitInputs(circuitsWasm, [storageSlot.toBuffer(), valueNoteHash]),
      );
      const siloedNoteHash = siloCommitment(circuitsWasm, contractAddress, innerNoteHash);
      const uniqueSiloedNoteHash = computeUniqueCommitment(circuitsWasm, note.nonce, siloedNoteHash);
      const innerNullifier = Fr.fromBuffer(
        pedersenPlookupCommitInputs(circuitsWasm, [uniqueSiloedNoteHash.toBuffer(), ownerPk.value]),
      );

      const result = await acirSimulator.computeNoteHashAndNullifier(
        contractAddress,
        note.nonce,
        storageSlot,
        note.preimage,
      );

      expect(result).toEqual({
        innerNoteHash,
        siloedNoteHash,
        uniqueSiloedNoteHash,
        innerNullifier,
      });
    });

    it('should a constructor with arguments that inserts notes', async () => {
      const abi = PrivateTokenContractAbi.functions.find(f => f.name === 'constructor')!;

      const result = await runSimulator({ args: [140, owner], abi });

      expect(result.preimages.newNotes).toHaveLength(1);
      const newNote = result.preimages.newNotes[0];
      expect(newNote.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm));

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));
      expect(newCommitments).toHaveLength(1);

      const [commitment] = newCommitments;
      expect(commitment).toEqual(
        await acirSimulator.computeInnerNoteHash(contractAddress, newNote.storageSlot, newNote.preimage),
      );
    });

    it('should run the mint function', async () => {
      const abi = PrivateTokenContractAbi.functions.find(f => f.name === 'mint')!;

      const result = await runSimulator({ args: [140, owner], abi });

      expect(result.preimages.newNotes).toHaveLength(1);
      const newNote = result.preimages.newNotes[0];
      expect(newNote.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm));

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));
      expect(newCommitments).toHaveLength(1);

      const [commitment] = newCommitments;
      expect(commitment).toEqual(
        await acirSimulator.computeInnerNoteHash(contractAddress, newNote.storageSlot, newNote.preimage),
      );
    });

    it('should run the transfer function', async () => {
      const amountToTransfer = 100n;
      const abi = PrivateTokenContractAbi.functions.find(f => f.name === 'transfer')!;

      const storageSlot = computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm);
      const recipientStorageSlot = computeSlotForMapping(new Fr(1n), recipient.toField(), circuitsWasm);

      const notes = [buildNote(60n, owner, storageSlot), buildNote(80n, owner, storageSlot)];
      oracle.getNotes.mockResolvedValue(notes);

      const consumedNotes = await asyncMap(notes, ({ nonce, preimage }) =>
        acirSimulator.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, preimage),
      );
      await insertLeaves(consumedNotes.map(n => n.siloedNoteHash));

      const args = [amountToTransfer, owner, recipient];
      const result = await runSimulator({ args, abi });

      // The two notes were nullified
      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toEqual(consumedNotes.map(n => n.innerNullifier));

      expect(result.preimages.newNotes).toHaveLength(2);
      const [changeNote, recipientNote] = result.preimages.newNotes;
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
      expect(readRequests).toEqual(consumedNotes.map(n => n.uniqueSiloedNoteHash));
    });

    it('should be able to transfer with dummy notes', async () => {
      const amountToTransfer = 100n;
      const balance = 160n;
      const abi = PrivateTokenContractAbi.functions.find(f => f.name === 'transfer')!;

      const storageSlot = computeSlotForMapping(new Fr(1n), owner.toField(), circuitsWasm);

      const notes = [buildNote(balance, owner, storageSlot)];
      oracle.getNotes.mockResolvedValue(notes);

      const consumedNotes = await asyncMap(notes, ({ nonce, preimage }) =>
        acirSimulator.computeNoteHashAndNullifier(contractAddress, nonce, storageSlot, preimage),
      );
      await insertLeaves(consumedNotes.map(n => n.siloedNoteHash));

      const args = [amountToTransfer, owner, recipient];
      const result = await runSimulator({ args, abi });

      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toEqual(consumedNotes.map(n => n.innerNullifier));

      expect(result.preimages.newNotes).toHaveLength(2);
      const [changeNote, recipientNote] = result.preimages.newNotes;
      expect(recipientNote.preimage[0]).toEqual(new Fr(amountToTransfer));
      expect(changeNote.preimage[0]).toEqual(new Fr(balance - amountToTransfer));
    });
  });

  describe('nested calls', () => {
    const privateIncrement = txContextFields.chainId.value + txContextFields.version.value;

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
      const childSelector = generateFunctionSelector(childAbi.name, childAbi.parameters);

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

      // check that Noir calculated the call stack item hash like cpp does
      const wasm = await CircuitsWasm.get();
      const expectedCallStackItemHash = computeCallStackItemHash(wasm, result.nestedExecutions[0].callStackItem);
      expect(result.callStackItem.publicInputs.privateCallStack[0]).toEqual(expectedCallStackItemHash);
    });
  });

  describe('nested calls through autogenerated interface', () => {
    let args: any[];
    let argsHash: Fr;
    let testCodeGenAbi: FunctionAbi;

    beforeAll(async () => {
      // These args should match the ones hardcoded in importer contract
      const dummyNote = { amount: 1, secretHash: 2 };
      const deepStruct = { aField: 1, aBool: true, aNote: dummyNote, manyNotes: [dummyNote, dummyNote, dummyNote] };
      args = [1, true, 1, [1, 2], dummyNote, deepStruct];
      testCodeGenAbi = TestContractAbi.functions.find(f => f.name === 'testCodeGen')!;
      const serialisedArgs = encodeArguments(testCodeGenAbi, args);
      argsHash = await computeVarArgsHash(await CircuitsWasm.get(), serialisedArgs);
    });

    it('test function should be directly callable', async () => {
      logger(`Calling testCodeGen function`);
      const result = await runSimulator({ args, abi: testCodeGenAbi });

      expect(result.callStackItem.publicInputs.returnValues[0]).toEqual(argsHash);
    });

    it('test function should be callable through autogenerated interface', async () => {
      const importerAddress = AztecAddress.random();
      const testAddress = AztecAddress.random();
      const parentAbi = ImportTestContractAbi.functions.find(f => f.name === 'main')!;
      const testCodeGenSelector = generateFunctionSelector(testCodeGenAbi.name, testCodeGenAbi.parameters);

      oracle.getFunctionABI.mockResolvedValue(testCodeGenAbi);
      oracle.getPortalContractAddress.mockResolvedValue(EthAddress.ZERO);

      logger(`Calling importer main function`);
      const args = [testAddress];
      const result = await runSimulator({ args, abi: parentAbi, origin: importerAddress });

      expect(result.callStackItem.publicInputs.returnValues[0]).toEqual(argsHash);
      expect(oracle.getFunctionABI.mock.calls[0]).toEqual([testAddress, testCodeGenSelector]);
      expect(oracle.getPortalContractAddress.mock.calls[0]).toEqual([testAddress]);
      expect(result.nestedExecutions).toHaveLength(1);
      expect(result.nestedExecutions[0].callStackItem.publicInputs.returnValues[0]).toEqual(argsHash);
    });
  });

  describe('consuming messages', () => {
    const contractAddress = defaultContractAddress;
    const recipientPk = PrivateKey.fromString('0c9ed344548e8f9ba8aa3c9f8651eaa2853130f6c1e9c050ccf198f7ea18a7ec');

    let recipient: AztecAddress;

    beforeEach(async () => {
      const recipientCompleteAddress = await CompleteAddress.fromPrivateKey(recipientPk);
      recipient = recipientCompleteAddress.address;
      oracle.getCompleteAddress.mockImplementation((address: AztecAddress) => {
        if (address.equals(recipient)) return Promise.resolve(recipientCompleteAddress);
        throw new Error(`Unknown address ${address}`);
      });
    });

    it('Should be able to consume a dummy cross chain message', async () => {
      const bridgedAmount = 100n;
      const abi = NonNativeTokenContractAbi.functions.find(f => f.name === 'mint')!;

      const secret = new Fr(1n);
      const canceller = EthAddress.random();
      // Function selector: 0xeeb73071 keccak256('mint(uint256,bytes32,address)')
      const preimage = await buildL1ToL2Message(
        'eeb73071',
        [new Fr(bridgedAmount), recipient.toField(), canceller.toField()],
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

      const args = [bridgedAmount, recipient, messageKey, secret, canceller.toField()];
      const result = await runSimulator({ origin: contractAddress, contractAddress, abi, args });

      // Check a nullifier has been inserted
      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toHaveLength(1);
    });

    it('Should be able to consume a dummy public to private message', async () => {
      const amount = 100n;
      const abi = NonNativeTokenContractAbi.functions.find(f => f.name === 'redeemShield')!;

      const wasm = await CircuitsWasm.get();
      const secret = new Fr(1n);
      const secretHash = computeSecretMessageHash(wasm, secret);
      const preimage = [toBufferBE(amount, 32), secretHash.toBuffer()];
      const noteHash = Fr.fromBuffer(hash(preimage));
      const storageSlot = new Fr(2);
      const innerNoteHash = hash([storageSlot.toBuffer(), noteHash.toBuffer()]);
      const siloedNoteHash = siloCommitment(wasm, contractAddress, Fr.fromBuffer(innerNoteHash));
      // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386): should insert
      // uniqueSiloedNoteHash in tree and that should be what is expected in Noir
      //const uniqueSiloedNoteHash = computeUniqueCommitment(wasm, nonce, Fr.fromBuffer(innerNoteHash));

      const tree = await insertLeaves([siloedNoteHash]);

      oracle.getCommitmentOracle.mockImplementation(async () => {
        // Check the calculated commitment is correct
        return Promise.resolve({
          commitment: siloedNoteHash,
          siblingPath: (await tree.getSiblingPath(0n, false)).toFieldArray(),
          index: 0n,
        });
      });

      const result = await runSimulator({
        origin: contractAddress,
        abi,
        args: [amount, secret, recipient],
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
      const parentAbi = ParentContractAbi.functions.find(f => f.name === 'enqueueCallToChild')!;
      const childContractAbi = ParentContractAbi.functions[0];
      const childAddress = AztecAddress.random();
      const childPortalContractAddress = EthAddress.random();
      const childSelector = generateFunctionSelector(childContractAbi.name, childContractAbi.parameters);
      const parentAddress = AztecAddress.random();

      oracle.getPortalContractAddress.mockImplementation(() => Promise.resolve(childPortalContractAddress));
      oracle.getFunctionABI.mockImplementation(() => Promise.resolve({ ...childContractAbi, isInternal }));

      const args = [Fr.fromBuffer(childAddress.toBuffer()), Fr.fromBuffer(childSelector), 42n];
      const result = await runSimulator({
        origin: parentAddress,
        contractAddress: parentAddress,
        abi: parentAbi,
        args,
      });

      // Alter function data (abi) to match the manipulated oracle
      const functionData = FunctionData.fromAbi(childContractAbi);
      functionData.isInternal = isInternal;

      const publicCallRequest = PublicCallRequest.from({
        contractAddress: childAddress,
        functionData: functionData,
        args: [new Fr(42n)],
        callContext: CallContext.from({
          msgSender: parentAddress,
          storageContractAddress: childAddress,
          portalContractAddress: childPortalContractAddress,
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
    let owner: AztecAddress;

    beforeEach(async () => {
      const ownerCompleteAddress = await CompleteAddress.fromPrivateKey(ownerPk);

      owner = ownerCompleteAddress.address;

      oracle.getCompleteAddress.mockImplementation((address: AztecAddress) => {
        if (address.equals(owner)) return Promise.resolve(ownerCompleteAddress);
        throw new Error(`Unknown address ${address}`);
      });
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

    it('should be able to insert, read, and nullify pending commitments in one call', async () => {
      oracle.getNotes.mockResolvedValue([]);

      const amountToTransfer = 100n;

      const contractAddress = AztecAddress.random();
      const abi = PendingCommitmentsContractAbi.functions.find(
        f => f.name === 'test_insert_then_get_then_nullify_flat',
      )!;

      const args = [amountToTransfer, owner];
      const result = await runSimulator({
        args: args,
        abi: abi,
        origin: contractAddress,
        contractAddress: contractAddress,
      });

      expect(result.preimages.newNotes).toHaveLength(1);
      const note = result.preimages.newNotes[0];
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
      expect(nullifier).toEqual(
        await acirSimulator.computeInnerNullifier(contractAddress, Fr.ZERO, note.storageSlot, note.preimage),
      );
    });

    it('should be able to insert, read, and nullify pending commitments in nested calls', async () => {
      oracle.getNotes.mockResolvedValue([]);

      const amountToTransfer = 100n;

      const contractAddress = AztecAddress.random();
      const abi = PendingCommitmentsContractAbi.functions.find(
        f => f.name === 'test_insert_then_get_then_nullify_all_in_nested_calls',
      )!;
      const insertAbi = PendingCommitmentsContractAbi.functions.find(f => f.name === 'insert_note')!;
      const getThenNullifyAbi = PendingCommitmentsContractAbi.functions.find(f => f.name === 'get_then_nullify_note')!;
      const getZeroAbi = PendingCommitmentsContractAbi.functions.find(f => f.name === 'get_note_zero_balance')!;

      const insertFnSelector = generateFunctionSelector(insertAbi.name, insertAbi.parameters);
      const getThenNullifyFnSelector = generateFunctionSelector(getThenNullifyAbi.name, getThenNullifyAbi.parameters);
      const getZeroFnSelector = generateFunctionSelector(getZeroAbi.name, getZeroAbi.parameters);

      oracle.getPortalContractAddress.mockImplementation(() => Promise.resolve(EthAddress.ZERO));

      const args = [
        amountToTransfer,
        owner,
        Fr.fromBuffer(insertFnSelector),
        Fr.fromBuffer(getThenNullifyFnSelector),
        Fr.fromBuffer(getZeroFnSelector),
      ];
      const result = await runSimulator({
        args: args,
        abi: abi,
        origin: contractAddress,
        contractAddress: contractAddress,
      });

      const execInsert = result.nestedExecutions[0];
      const execGetThenNullify = result.nestedExecutions[1];
      const getNotesAfterNullify = result.nestedExecutions[2];

      expect(execInsert.preimages.newNotes).toHaveLength(1);
      const note = execInsert.preimages.newNotes[0];
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

      const nonce = Fr.ZERO;
      const nullifier = execGetThenNullify.callStackItem.publicInputs.newNullifiers[0];
      expect(nullifier).toEqual(
        await acirSimulator.computeInnerNullifier(contractAddress, nonce, note.storageSlot, note.preimage),
      );

      // check that the last get_notes call return no note
      const afterNullifyingNoteValue = getNotesAfterNullify.callStackItem.publicInputs.returnValues[0].value;
      expect(afterNullifyingNoteValue).toEqual(0n);
    });

    it('cant read a commitment that is inserted later in same call', async () => {
      oracle.getNotes.mockResolvedValue([]);

      const amountToTransfer = 100n;

      const contractAddress = AztecAddress.random();
      const abi = PendingCommitmentsContractAbi.functions.find(f => f.name === 'test_bad_get_then_insert_flat')!;

      const args = [amountToTransfer, owner];
      const result = await runSimulator({
        args: args,
        abi: abi,
        origin: contractAddress,
        contractAddress: contractAddress,
      });

      expect(result.preimages.newNotes).toHaveLength(1);
      const note = result.preimages.newNotes[0];
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
      // Tweak the contract ABI so we can extract return values
      const abi = TestContractAbi.functions.find(f => f.name === 'getPublicKey')!;
      abi.returnTypes = [{ kind: 'array', length: 2, type: { kind: 'field' } }];

      // Generate a partial address, pubkey, and resulting address
      const completeAddress = await CompleteAddress.random();
      const args = [completeAddress.address];
      const pubKey = completeAddress.publicKey;

      oracle.getCompleteAddress.mockResolvedValue(completeAddress);
      const result = await runSimulator({ origin: AztecAddress.random(), abi, args });
      expect(result.returnValues).toEqual([pubKey.x.value, pubKey.y.value]);
    });
  });

  describe('Context oracles', () => {
    it("Should be able to get and return the contract's portal contract address", async () => {
      const portalContractAddress = EthAddress.random();
      const aztecAddressToQuery = AztecAddress.random();

      // Tweak the contract ABI so we can extract return values
      const abi = TestContractAbi.functions.find(f => f.name === 'getPortalContractAddress')!;
      abi.returnTypes = [{ kind: 'field' }];

      const args = [aztecAddressToQuery.toField()];

      // Overwrite the oracle return value
      oracle.getPortalContractAddress.mockResolvedValue(portalContractAddress);
      const result = await runSimulator({ origin: AztecAddress.random(), abi, args });
      expect(result.returnValues).toEqual(portalContractAddress.toField().value);
    });

    it('this_address should return the current context address', async () => {
      const contractAddress = AztecAddress.random();

      // Tweak the contract ABI so we can extract return values
      const abi = TestContractAbi.functions.find(f => f.name === 'getThisAddress')!;
      abi.returnTypes = [{ kind: 'field' }];

      // Overwrite the oracle return value
      const result = await runSimulator({ origin: AztecAddress.random(), abi, args: [], contractAddress });
      expect(result.returnValues).toEqual(contractAddress.toField().value);
    });

    it("this_portal_address should return the current context's portal address", async () => {
      const portalContractAddress = EthAddress.random();

      // Tweak the contract ABI so we can extract return values
      const abi = TestContractAbi.functions.find(f => f.name === 'getThisPortalAddress')!;
      abi.returnTypes = [{ kind: 'field' }];

      // Overwrite the oracle return value
      const result = await runSimulator({ origin: AztecAddress.random(), abi, args: [], portalContractAddress });
      expect(result.returnValues).toEqual(portalContractAddress.toField().value);
    });
  });
});
