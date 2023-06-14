import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import {
  ARGS_LENGTH,
  CallContext,
  CircuitsWasm,
  ContractDeploymentData,
  FunctionData,
  L1_TO_L2_MESSAGES_TREE_HEIGHT,
  NEW_COMMITMENTS_LENGTH,
  PRIVATE_DATA_TREE_HEIGHT,
  PrivateHistoricTreeRoots,
  PublicCallRequest,
  TxContext,
} from '@aztec/circuits.js';
import { computeSecretMessageHash } from '@aztec/circuits.js/abis';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { toBigIntBE, toBufferBE } from '@aztec/foundation/bigint-buffer';
import { padArrayEnd } from '@aztec/foundation/collection';
import { sha256 } from '@aztec/foundation/crypto';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { AppendOnlyTree, Pedersen, StandardTree, newTree } from '@aztec/merkle-tree';
import {
  ChildAbi,
  NonNativeTokenContractAbi,
  ParentAbi,
  TestContractAbi,
  ZkTokenContractAbi,
} from '@aztec/noir-contracts/examples';
import { L1Actor, L1ToL2Message, L2Actor, TxExecutionRequest } from '@aztec/types';
import { mock } from 'jest-mock-extended';
import { default as levelup } from 'levelup';
import { default as memdown, type MemDown } from 'memdown';
import { encodeArguments } from '../abi_coder/index.js';
import { NoirPoint, computeSlotForMapping, toPublicKey } from '../utils.js';
import { DBOracle } from './db_oracle.js';
import { AcirSimulator } from './simulator.js';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';

const createMemDown = () => (memdown as any)() as MemDown<any, any>;

describe('Private Execution test suite', () => {
  let bbWasm: CircuitsWasm;
  let oracle: ReturnType<typeof mock<DBOracle>>;
  let acirSimulator: AcirSimulator;
  let logger: DebugLogger;

  beforeAll(async () => {
    bbWasm = await CircuitsWasm.get();
    logger = createDebugLogger('aztec:test:private_execution');
  });

  beforeEach(() => {
    oracle = mock<DBOracle>();
    acirSimulator = new AcirSimulator(oracle);
  });

  describe('empty constructor', () => {
    const historicRoots = PrivateHistoricTreeRoots.empty();
    const contractDeploymentData = ContractDeploymentData.empty();
    const txContext = new TxContext(false, false, false, contractDeploymentData);

    it('should run the empty constructor', async () => {
      const txRequest = new TxExecutionRequest(
        AztecAddress.random(),
        AztecAddress.ZERO,
        new FunctionData(Buffer.alloc(4), true, true),
        new Array(ARGS_LENGTH).fill(Fr.ZERO),
        Fr.random(),
        txContext,
        Fr.ZERO,
      );
      const result = await acirSimulator.run(
        txRequest,
        TestContractAbi.functions[0],
        AztecAddress.ZERO,
        EthAddress.ZERO,
        historicRoots,
      );

      expect(result.callStackItem.publicInputs.newCommitments).toEqual(new Array(NEW_COMMITMENTS_LENGTH).fill(Fr.ZERO));
    });
  });

  describe('zk token contract', () => {
    let currentNonce = 0n;

    const contractDeploymentData = ContractDeploymentData.empty();
    const txContext = new TxContext(false, false, false, contractDeploymentData);

    let ownerPk: Buffer;
    let owner: NoirPoint;
    let recipientPk: Buffer;
    let recipient: NoirPoint;

    const buildNote = (amount: bigint, owner: NoirPoint) => {
      return [new Fr(1n), new Fr(currentNonce++), new Fr(owner.x), new Fr(owner.y), Fr.random(), new Fr(amount)];
    };

    beforeAll(() => {
      ownerPk = Buffer.from('5e30a2f886b4b6a11aea03bf4910fbd5b24e61aa27ea4d05c393b3ab592a8d33', 'hex');
      recipientPk = Buffer.from('0c9ed344548e8f9ba8aa3c9f8651eaa2853130f6c1e9c050ccf198f7ea18a7ec', 'hex');

      const grumpkin = new Grumpkin(bbWasm);
      owner = toPublicKey(ownerPk, grumpkin);
      recipient = toPublicKey(recipientPk, grumpkin);
    });

    it('should a constructor with arguments that creates notes', async () => {
      const historicRoots = PrivateHistoricTreeRoots.empty();
      const contractAddress = AztecAddress.random();
      const abi = ZkTokenContractAbi.functions.find(f => f.name === 'constructor')!;

      const txRequest = new TxExecutionRequest(
        AztecAddress.random(),
        AztecAddress.ZERO,
        new FunctionData(Buffer.alloc(4), true, true),
        encodeArguments(abi, [140, owner]),
        Fr.random(),
        txContext,
        Fr.ZERO,
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
      const historicRoots = PrivateHistoricTreeRoots.empty();
      const contractAddress = AztecAddress.random();
      const abi = ZkTokenContractAbi.functions.find(f => f.name === 'mint')!;

      const txRequest = new TxExecutionRequest(
        AztecAddress.random(),
        contractAddress,
        new FunctionData(Buffer.alloc(4), true, false),
        encodeArguments(abi, [140, owner]),
        Fr.random(),
        txContext,
        Fr.ZERO,
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
        Fr.ZERO,
        Fr.ZERO,
        Fr.ZERO,
        Fr.ZERO,
      );

      oracle.getNotes.mockImplementation(async () => {
        return {
          count: preimages.length,
          notes: await Promise.all(
            preimages.map((preimage, index) => ({
              preimage,
              index: BigInt(index),
            })),
          ),
        };
      });

      oracle.getSecretKey.mockReturnValue(Promise.resolve(ownerPk));

      const txRequest = new TxExecutionRequest(
        AztecAddress.random(),
        contractAddress,
        new FunctionData(Buffer.alloc(4), true, true),
        encodeArguments(abi, [amountToTransfer, owner, recipient]),
        Fr.random(),
        txContext,
        Fr.ZERO,
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
        Fr.ZERO,
        Fr.ZERO,
        Fr.ZERO,
        Fr.ZERO,
      );

      oracle.getNotes.mockImplementation(async () => {
        return {
          count: preimages.length,
          notes: await Promise.all(
            preimages.map((preimage, index) => ({
              preimage,
              index: BigInt(index),
            })),
          ),
        };
      });

      oracle.getSecretKey.mockReturnValue(Promise.resolve(ownerPk));

      const txRequest = new TxExecutionRequest(
        AztecAddress.random(),
        contractAddress,
        new FunctionData(Buffer.alloc(4), true, true),
        encodeArguments(abi, [amountToTransfer, owner, recipient]),
        Fr.random(),
        txContext,
        Fr.ZERO,
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
    const historicRoots = PrivateHistoricTreeRoots.empty();
    const contractDeploymentData = new ContractDeploymentData(
      Point.random(),
      Fr.random(),
      Fr.random(),
      Fr.random(),
      EthAddress.ZERO,
    );
    const txContext = new TxContext(false, false, true, contractDeploymentData);

    it('child function should be callable', async () => {
      const abi = ChildAbi.functions.find(f => f.name === 'value')!;

      const txRequest = new TxExecutionRequest(
        AztecAddress.random(),
        AztecAddress.ZERO,
        new FunctionData(Buffer.alloc(4), true, false),
        encodeArguments(abi, [100n]),
        Fr.random(),
        txContext,
        Fr.ZERO,
      );
      const result = await acirSimulator.run(txRequest, abi, AztecAddress.ZERO, EthAddress.ZERO, historicRoots);

      expect(result.callStackItem.publicInputs.returnValues[0]).toEqual(new Fr(142n));
    });

    it('parent should call child', async () => {
      const childAbi = ChildAbi.functions.find(f => f.name === 'value')!;
      const parentAbi = ParentAbi.functions.find(f => f.name === 'entryPoint')!;
      const parentAddress = AztecAddress.random();
      const childAddress = AztecAddress.random();
      const childSelector = Buffer.alloc(4, 1); // should match the call

      oracle.getFunctionABI.mockImplementation(() => Promise.resolve(childAbi));
      oracle.getPortalContractAddress.mockImplementation(() => Promise.resolve(EthAddress.ZERO));

      const txRequest = new TxExecutionRequest(
        AztecAddress.random(),
        parentAddress,
        new FunctionData(Buffer.alloc(4), true, false),
        encodeArguments(parentAbi, [Fr.fromBuffer(childAddress.toBuffer()), Fr.fromBuffer(childSelector)]),
        Fr.random(),
        txContext,
        Fr.ZERO,
      );

      logger(`Parent deployed at ${parentAddress.toShortString()}`);
      logger(`Calling child function ${childSelector.toString('hex')} at ${childAddress.toShortString()}`);
      const result = await acirSimulator.run(txRequest, parentAbi, parentAddress, EthAddress.ZERO, historicRoots);

      expect(result.callStackItem.publicInputs.returnValues[0]).toEqual(new Fr(42n));
      expect(oracle.getFunctionABI.mock.calls[0]).toEqual([childAddress, childSelector]);
      expect(oracle.getPortalContractAddress.mock.calls[0]).toEqual([childAddress]);
      expect(result.nestedExecutions).toHaveLength(1);
      expect(result.nestedExecutions[0].callStackItem.publicInputs.returnValues[0]).toEqual(new Fr(42n));
    });
  });

  describe('Consuming Messages', () => {
    const contractDeploymentData = ContractDeploymentData.empty();
    const txContext = new TxContext(false, false, false, contractDeploymentData);

    let recipientPk: Buffer;
    let recipient: NoirPoint;

    const buildL1ToL2Message = async (contentPreimage: Fr[], targetContract: AztecAddress, secret: Fr) => {
      const wasm = await CircuitsWasm.get();

      // Function selector: 0xeeb73071 keccak256('mint(uint256,bytes32,address)')
      const contentBuf = Buffer.concat([
        Buffer.from([0xee, 0xb7, 0x30, 0x71]),
        ...contentPreimage.map(field => field.toBuffer()),
      ]);
      const temp = toBigIntBE(sha256(contentBuf));
      const content = Fr.fromBuffer(toBufferBE(temp % Fr.MODULUS, 32));

      const secretHash = computeSecretMessageHash(wasm, secret);

      // Eventually the kernel will need to prove the kernel portal pair exists within the contract tree,
      // EthAddress.random() will need to be replaced when this happens
      return new L1ToL2Message(
        new L1Actor(EthAddress.random(), 1),
        new L2Actor(targetContract, 1),
        content,
        secretHash,
        0,
        0,
      );
    };

    beforeAll(() => {
      recipientPk = Buffer.from('0c9ed344548e8f9ba8aa3c9f8651eaa2853130f6c1e9c050ccf198f7ea18a7ec', 'hex');

      const grumpkin = new Grumpkin(bbWasm);
      recipient = toPublicKey(recipientPk, grumpkin);
    });

    it('Should be able to consume a dummy cross chain message', async () => {
      const db = levelup(createMemDown());
      const pedersen = new Pedersen(bbWasm);

      const contractAddress = AztecAddress.random();
      const bridgedAmount = 100n;
      const abi = NonNativeTokenContractAbi.functions.find(f => f.name === 'mint')!;

      const secret = new Fr(1n);
      const canceller = EthAddress.random();
      const preimage = await buildL1ToL2Message(
        [new Fr(bridgedAmount), new Fr(recipient.x), canceller.toField()],
        contractAddress,
        secret,
      );

      // stub message key
      const messageKey = Fr.random();

      const tree: AppendOnlyTree = await newTree(
        StandardTree,
        db,
        pedersen,
        'l1ToL2Messages',
        L1_TO_L2_MESSAGES_TREE_HEIGHT,
      );

      await tree.appendLeaves([messageKey.toBuffer()]);

      const l1ToL2Root = Fr.fromBuffer(tree.getRoot(false));
      const historicRoots = new PrivateHistoricTreeRoots(Fr.ZERO, Fr.ZERO, Fr.ZERO, l1ToL2Root, Fr.ZERO);

      oracle.getL1ToL2Message.mockImplementation(async () => {
        return Promise.resolve({
          message: preimage.toFieldArray(),
          index: 0n,
          siblingPath: (await tree.getSiblingPath(0n, false)).toFieldArray(),
        });
      });

      const txRequest = new TxExecutionRequest(
        AztecAddress.random(),
        contractAddress,
        new FunctionData(Buffer.alloc(4), true, true),
        encodeArguments(abi, [bridgedAmount, recipient, messageKey, secret, canceller.toField()]),
        Fr.random(),
        txContext,
        Fr.ZERO,
      );

      const result = await acirSimulator.run(txRequest, abi, contractAddress, EthAddress.ZERO, historicRoots);

      // Check a nullifier has been created
      const newNullifiers = result.callStackItem.publicInputs.newNullifiers.filter(field => !field.equals(Fr.ZERO));
      expect(newNullifiers).toHaveLength(1);
    }, 30_000);
  });

  describe('enqueued calls', () => {
    const historicRoots = PrivateHistoricTreeRoots.empty();
    const txContext = new TxContext(false, false, true, ContractDeploymentData.empty());

    it('parent should enqueue call to child', async () => {
      const parentAbi = ParentAbi.functions.find(f => f.name === 'enqueueCallToChild')!;
      const childAddress = AztecAddress.random();
      const childPortalContractAddress = EthAddress.random();
      const childSelector = Buffer.alloc(4, 1); // should match the call
      const parentAddress = AztecAddress.random();

      oracle.getPortalContractAddress.mockImplementation(() => Promise.resolve(childPortalContractAddress));

      const txRequest = new TxExecutionRequest(
        AztecAddress.random(),
        parentAddress,
        new FunctionData(Buffer.alloc(4), true, false),
        encodeArguments(parentAbi, [Fr.fromBuffer(childAddress.toBuffer()), Fr.fromBuffer(childSelector), 42n]),
        Fr.random(),
        txContext,
        Fr.ZERO,
      );

      const result = await acirSimulator.run(txRequest, parentAbi, parentAddress, EthAddress.ZERO, historicRoots);

      expect(result.enqueuedPublicFunctionCalls).toHaveLength(1);
      expect(result.enqueuedPublicFunctionCalls[0]).toEqual(
        PublicCallRequest.from({
          contractAddress: childAddress,
          functionData: new FunctionData(childSelector, false, false),
          args: padArrayEnd([new Fr(42n)], Fr.ZERO, ARGS_LENGTH),
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
});
