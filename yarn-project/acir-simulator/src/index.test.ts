import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import {
  ARGS_LENGTH,
  ContractDeploymentData,
  FunctionData,
  NEW_COMMITMENTS_LENGTH,
  PrivateHistoricTreeRoots,
  PRIVATE_DATA_TREE_HEIGHT,
  TxContext,
  TxRequest,
} from '@aztec/circuits.js';
import { AztecAddress, EthAddress, Fr } from '@aztec/foundation';
import { AppendOnlyTree, Pedersen, StandardTree, newTree } from '@aztec/merkle-tree';
import { FunctionAbi } from '@aztec/noir-contracts';
import { ChildAbi, ParentAbi, TestContractAbi, ZkTokenContractAbi } from '@aztec/noir-contracts/examples';
import { mock } from 'jest-mock-extended';
import { default as levelup } from 'levelup';
import { default as memdown, type MemDown } from 'memdown';
import { encodeArguments } from './abi_coder/index.js';
import { DBOracle } from './db_oracle.js';
import { AcirSimulator } from './simulator.js';
import { NoirPoint, computeSlotForMapping, toPublicKey } from './utils.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

describe('ACIR simulator', () => {
  let bbWasm: BarretenbergWasm;
  let oracle: ReturnType<typeof mock<DBOracle>>;
  let acirSimulator: AcirSimulator;

  beforeAll(async () => {
    bbWasm = await BarretenbergWasm.get();
  });

  beforeEach(() => {
    oracle = mock<DBOracle>();
    acirSimulator = new AcirSimulator(oracle);
  });

  describe('empty constructor', () => {
    const historicRoots = new PrivateHistoricTreeRoots(new Fr(0n), new Fr(0n), new Fr(0n), new Fr(0n));
    const contractDeploymentData = new ContractDeploymentData(Fr.random(), Fr.random(), Fr.random(), EthAddress.ZERO);
    const txContext = new TxContext(false, false, true, contractDeploymentData);

    it('should run the empty constructor', async () => {
      const txRequest = new TxRequest(
        AztecAddress.random(),
        AztecAddress.ZERO,
        new FunctionData(Buffer.alloc(4), true, true),
        new Array(ARGS_LENGTH).fill(new Fr(0n)),
        Fr.random(),
        txContext,
        new Fr(0n),
      );
      const result = await acirSimulator.run(
        txRequest,
        TestContractAbi.functions[0],
        AztecAddress.ZERO,
        EthAddress.ZERO,
        historicRoots,
      );

      expect(result.callStackItem.publicInputs.newCommitments).toEqual(
        new Array(NEW_COMMITMENTS_LENGTH).fill(new Fr(0n)),
      );
    });
  });

  describe('zk token contract', () => {
    let currentNonce = 0n;

    const contractDeploymentData = new ContractDeploymentData(Fr.ZERO, Fr.ZERO, Fr.ZERO, EthAddress.ZERO);
    const txContext = new TxContext(false, false, false, contractDeploymentData);

    let ownerPk: Buffer;
    let owner: NoirPoint;
    let recipientPk: Buffer;
    let recipient: NoirPoint;

    function buildNote(amount: bigint, owner: NoirPoint) {
      return [new Fr(1n), new Fr(currentNonce++), new Fr(owner.x), new Fr(owner.y), Fr.random(), new Fr(amount)];
    }

    beforeAll(() => {
      ownerPk = Buffer.from('5e30a2f886b4b6a11aea03bf4910fbd5b24e61aa27ea4d05c393b3ab592a8d33', 'hex');
      recipientPk = Buffer.from('0c9ed344548e8f9ba8aa3c9f8651eaa2853130f6c1e9c050ccf198f7ea18a7ec', 'hex');

      const grumpkin = new Grumpkin(bbWasm);
      owner = toPublicKey(ownerPk, grumpkin);
      recipient = toPublicKey(recipientPk, grumpkin);
    });

    it('should a constructor with arguments that creates notes', async () => {
      const historicRoots = new PrivateHistoricTreeRoots(new Fr(0n), new Fr(0n), new Fr(0n), new Fr(0n));
      const contractAddress = AztecAddress.random();
      const abi = ZkTokenContractAbi.functions.find(f => f.name === 'constructor')!;

      const txRequest = new TxRequest(
        AztecAddress.random(),
        AztecAddress.ZERO,
        new FunctionData(Buffer.alloc(4), true, true),
        encodeArguments(abi, [140, owner]),
        Fr.random(),
        txContext,
        new Fr(0n),
      );
      const result = await acirSimulator.run(txRequest, abi, contractAddress, EthAddress.ZERO, historicRoots);

      expect(result.preimages.newNotes).toHaveLength(1);
      const newNote = result.preimages.newNotes[0];
      expect(newNote.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner, bbWasm));

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));
      expect(newCommitments).toHaveLength(1);

      const [commitment] = newCommitments;
      expect(commitment).toEqual(Fr.fromBuffer(acirSimulator.computeNoteHash(newNote.preimage, bbWasm)));
    }, 30_000);

    it('should run the mint function', async () => {
      const historicRoots = new PrivateHistoricTreeRoots(new Fr(0n), new Fr(0n), new Fr(0n), new Fr(0n));
      const contractAddress = AztecAddress.random();
      const abi = ZkTokenContractAbi.functions.find(f => f.name === 'mint')!;

      const txRequest = new TxRequest(
        AztecAddress.random(),
        contractAddress,
        new FunctionData(Buffer.alloc(4), true, false),
        encodeArguments(abi, [140, owner]),
        Fr.random(),
        txContext,
        new Fr(0n),
      );
      const result = await acirSimulator.run(txRequest, abi, AztecAddress.ZERO, EthAddress.ZERO, historicRoots);

      expect(result.preimages.newNotes).toHaveLength(1);
      const newNote = result.preimages.newNotes[0];
      expect(newNote.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), owner, bbWasm));

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));
      expect(newCommitments).toHaveLength(1);

      const [commitment] = newCommitments;
      expect(commitment).toEqual(Fr.fromBuffer(acirSimulator.computeNoteHash(newNote.preimage, bbWasm)));
    });

    it('should run the transfer function', async () => {
      const db = levelup(createMemDown());
      const pedersen = new Pedersen(bbWasm);

      const contractAddress = AztecAddress.random();
      const amountToTransfer = 100n;
      const abi = ZkTokenContractAbi.functions.find(f => f.name === 'transfer')!;

      const tree: AppendOnlyTree = await newTree(StandardTree, db, pedersen, 'privateData', PRIVATE_DATA_TREE_HEIGHT);
      const preimages = [buildNote(60n, owner), buildNote(80n, owner)];
      // TODO for this we need that noir siloes the commitment the same way as the kernel does, to do merkle membership
      await tree.appendLeaves(preimages.map(preimage => acirSimulator.computeNoteHash(preimage, bbWasm)));

      const historicRoots = new PrivateHistoricTreeRoots(
        Fr.fromBuffer(tree.getRoot(false)),
        new Fr(0n),
        new Fr(0n),
        new Fr(0n),
      );

      oracle.getNotes.mockImplementation(async () => {
        return {
          count: preimages.length,
          notes: await Promise.all(
            preimages.map(async (preimage, index) => ({
              preimage,
              siblingPath: (await tree.getSiblingPath(BigInt(index), false)).data.map(buf => Fr.fromBuffer(buf)),
              index: BigInt(index),
            })),
          ),
        };
      });

      oracle.getSecretKey.mockReturnValue(Promise.resolve(ownerPk));

      const txRequest = new TxRequest(
        AztecAddress.random(),
        contractAddress,
        new FunctionData(Buffer.alloc(4), true, true),
        encodeArguments(abi, [amountToTransfer, owner, recipient]),
        Fr.random(),
        txContext,
        new Fr(0n),
      );

      const result = await acirSimulator.run(txRequest, abi, AztecAddress.random(), EthAddress.ZERO, historicRoots);

      // The two notes were nullified
      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toHaveLength(2);

      expect(newNullifiers).toEqual(
        preimages.map(preimage => Fr.fromBuffer(acirSimulator.computeNullifier(preimage, ownerPk, bbWasm))),
      );

      expect(result.preimages.newNotes).toHaveLength(2);
      const [recipientNote, changeNote] = result.preimages.newNotes;
      expect(recipientNote.storageSlot).toEqual(computeSlotForMapping(new Fr(1n), recipient, bbWasm));

      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));

      expect(newCommitments).toHaveLength(2);

      const [recipientNoteCommitment, changeNoteCommitment] = newCommitments;
      expect(recipientNoteCommitment).toEqual(
        Fr.fromBuffer(acirSimulator.computeNoteHash(recipientNote.preimage, bbWasm)),
      );
      expect(changeNoteCommitment).toEqual(Fr.fromBuffer(acirSimulator.computeNoteHash(changeNote.preimage, bbWasm)));

      expect(recipientNote.preimage[5]).toEqual(new Fr(amountToTransfer));
      expect(changeNote.preimage[5]).toEqual(new Fr(40n));
    }, 30_000);

    it('should be able to transfer with dummy notes', async () => {
      const db = levelup(createMemDown());
      const pedersen = new Pedersen(bbWasm);

      const contractAddress = AztecAddress.random();
      const amountToTransfer = 100n;
      const balance = 160n;
      const abi = ZkTokenContractAbi.functions.find(f => f.name === 'transfer')!;

      const tree: AppendOnlyTree = await newTree(StandardTree, db, pedersen, 'privateData', PRIVATE_DATA_TREE_HEIGHT);
      const preimages = [buildNote(balance, owner)];
      // TODO for this we need that noir siloes the commitment the same way as the kernel does, to do merkle membership
      await tree.appendLeaves(preimages.map(preimage => acirSimulator.computeNoteHash(preimage, bbWasm)));

      const historicRoots = new PrivateHistoricTreeRoots(
        Fr.fromBuffer(tree.getRoot(false)),
        new Fr(0n),
        new Fr(0n),
        new Fr(0n),
      );

      oracle.getNotes.mockImplementation(async () => {
        return {
          count: preimages.length,
          notes: await Promise.all(
            preimages.map(async (preimage, index) => ({
              preimage,
              siblingPath: (await tree.getSiblingPath(BigInt(index), false)).data.map(buf => Fr.fromBuffer(buf)),
              index: BigInt(index),
            })),
          ),
        };
      });

      oracle.getSecretKey.mockReturnValue(Promise.resolve(ownerPk));

      const txRequest = new TxRequest(
        AztecAddress.random(),
        contractAddress,
        new FunctionData(Buffer.alloc(4), true, true),
        encodeArguments(abi, [amountToTransfer, owner, recipient]),
        Fr.random(),
        txContext,
        new Fr(0n),
      );

      const result = await acirSimulator.run(txRequest, abi, AztecAddress.random(), EthAddress.ZERO, historicRoots);

      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toHaveLength(2);

      expect(newNullifiers[0]).toEqual(Fr.fromBuffer(acirSimulator.computeNullifier(preimages[0], ownerPk, bbWasm)));

      expect(result.preimages.newNotes).toHaveLength(2);
      const [recipientNote, changeNote] = result.preimages.newNotes;
      expect(recipientNote.preimage[5]).toEqual(new Fr(amountToTransfer));
      expect(changeNote.preimage[5]).toEqual(new Fr(balance - amountToTransfer));
    }, 30_000);
  });

  describe('nested calls', () => {
    const historicRoots = new PrivateHistoricTreeRoots(new Fr(0n), new Fr(0n), new Fr(0n), new Fr(0n));
    const contractDeploymentData = new ContractDeploymentData(Fr.random(), Fr.random(), Fr.random(), EthAddress.ZERO);
    const txContext = new TxContext(false, false, true, contractDeploymentData);

    it('child function should be callable', async () => {
      const abi = ChildAbi.functions.find(f => f.name === 'value')!;

      const txRequest = new TxRequest(
        AztecAddress.random(),
        AztecAddress.ZERO,
        new FunctionData(Buffer.alloc(4), true, false),
        encodeArguments(abi, [100n]),
        Fr.random(),
        txContext,
        new Fr(0n),
      );
      const result = await acirSimulator.run(txRequest, abi, AztecAddress.ZERO, EthAddress.ZERO, historicRoots);

      expect(result.returnValues[0]).toEqual(142n);
    });

    it('parent should call child', async () => {
      const childAbi = ChildAbi.functions.find(f => f.name === 'value')!;
      const parentAbi = ParentAbi.functions.find(f => f.name === 'entryPoint')!;
      const childAddress = AztecAddress.random();
      const childSelector = Buffer.alloc(4, 1); // should match the call

      oracle.getFunctionABI.mockImplementation(() => Promise.resolve(childAbi));
      oracle.getPortalContractAddress.mockImplementation(() => Promise.resolve(EthAddress.ZERO));

      const txRequest = new TxRequest(
        AztecAddress.random(),
        AztecAddress.ZERO,
        new FunctionData(Buffer.alloc(4), true, false),
        encodeArguments(parentAbi, [Fr.fromBuffer(childAddress.toBuffer()).value, Fr.fromBuffer(childSelector).value]),
        Fr.random(),
        txContext,
        new Fr(0n),
      );
      const result = await acirSimulator.run(
        txRequest,
        parentAbi,
        AztecAddress.random(),
        EthAddress.ZERO,
        historicRoots,
      );

      expect(result.returnValues[0]).toEqual(42n);
      expect(oracle.getFunctionABI.mock.calls[0]).toEqual([childAddress, childSelector]);
      expect(oracle.getPortalContractAddress.mock.calls[0]).toEqual([childAddress]);
      expect(result.nestedExecutions).toHaveLength(1);
      expect(result.nestedExecutions[0].returnValues[0]).toEqual(42n);
    });
  });
});
