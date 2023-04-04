import {
  ARGS_LENGTH,
  ContractDeploymentData,
  FunctionData,
  NEW_COMMITMENTS_LENGTH,
  OldTreeRoots,
  TxContext,
  TxRequest,
} from '@aztec/circuits.js';
import { AztecAddress, EthAddress, Fr } from '@aztec/foundation';
import { Grumpkin, pedersenCompressInputs } from '@aztec/barretenberg.js/crypto';
import { FunctionAbi } from '@aztec/noir-contracts';
import { TestContractAbi, ZkTokenContractAbi } from '@aztec/noir-contracts/examples';
import { DBOracle } from './db_oracle.js';
import { AcirSimulator, MAPPING_SLOT_PEDERSEN_CONSTANT, NOTE_SLOT_PEDERSEN_CONSTANT } from './simulator.js';
import { jest } from '@jest/globals';
import { toBigIntBE } from '@aztec/foundation';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { default as levelup } from 'levelup';
import { default as memdown } from 'memdown';
import { Pedersen, StandardMerkleTree } from '@aztec/merkle-tree';
import { encodeArguments } from './arguments_encoder/index.js';

type NoirPoint = {
  x: bigint;
  y: bigint;
};

export const createMemDown = () => (memdown as any)();

describe('ACIR simulator', () => {
  let bbWasm: BarretenbergWasm;

  const oracle = {
    getNotes: jest.fn<DBOracle['getNotes']>(),
    getSecretKey: jest.fn<DBOracle['getSecretKey']>(),
    getBytecode: jest.fn<DBOracle['getBytecode']>(),
    getPortalContractAddress: jest.fn<DBOracle['getPortalContractAddress']>(),
  };
  const acirSimulator = new AcirSimulator(oracle as unknown as DBOracle);

  beforeAll(async () => {
    bbWasm = await BarretenbergWasm.new();
  });

  describe('empty constructor', () => {
    const oldRoots = new OldTreeRoots(new Fr(0n), new Fr(0n), new Fr(0n), new Fr(0n));
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
        TestContractAbi.functions[0] as FunctionAbi,
        AztecAddress.ZERO,
        EthAddress.ZERO,
        oldRoots,
      );

      expect(result.callStackItem.publicInputs.newCommitments).toEqual(
        new Array(NEW_COMMITMENTS_LENGTH).fill(new Fr(0n)),
      );
    });
  });

  describe('token contract', () => {
    let currentNonce = 0n;
    const SIBLING_PATH_SIZE = 5;

    const contractDeploymentData = new ContractDeploymentData(Fr.ZERO, Fr.ZERO, Fr.ZERO, EthAddress.ZERO);
    const txContext = new TxContext(false, false, false, contractDeploymentData);

    function computeSlot(mappingSlot: Fr, owner: NoirPoint, bbWasm: BarretenbergWasm) {
      return Fr.fromBuffer(
        pedersenCompressInputs(
          bbWasm,
          [MAPPING_SLOT_PEDERSEN_CONSTANT, mappingSlot, new Fr(owner.x)].map(f => f.toBuffer()),
        ),
      );
    }

    let ownerPk: Buffer;
    let owner: NoirPoint;
    let recipientPk: Buffer;
    let recipient: NoirPoint;

    function buildNote(amount: bigint, owner: NoirPoint, isDummy = false) {
      return [
        new Fr(amount),
        new Fr(owner.x),
        new Fr(owner.y),
        new Fr(4n),
        new Fr(currentNonce++),
        new Fr(isDummy ? 1n : 0n),
      ];
    }

    function computeCommitment(noteHash: Buffer, slot: Fr, contractAddress: AztecAddress, bbWasm: BarretenbergWasm) {
      return pedersenCompressInputs(bbWasm, [
        NOTE_SLOT_PEDERSEN_CONSTANT.toBuffer(),
        noteHash,
        slot.toBuffer(),
        contractAddress.toBuffer(),
      ]);
    }

    function toPublicKey(privateKey: Buffer, grumpkin: Grumpkin): NoirPoint {
      const publicKey = grumpkin.mul(Grumpkin.generator, privateKey);
      return {
        x: toBigIntBE(publicKey.slice(0, 32)),
        y: toBigIntBE(publicKey.slice(32, 64)),
      };
    }

    beforeAll(() => {
      ownerPk = Buffer.from('5e30a2f886b4b6a11aea03bf4910fbd5b24e61aa27ea4d05c393b3ab592a8d33', 'hex');
      recipientPk = Buffer.from('0c9ed344548e8f9ba8aa3c9f8651eaa2853130f6c1e9c050ccf198f7ea18a7ec', 'hex');

      const grumpkin = new Grumpkin(bbWasm);
      owner = toPublicKey(ownerPk, grumpkin);
      recipient = toPublicKey(recipientPk, grumpkin);
    });

    it('should a constructor with arguments that creates notes', async () => {
      const oldRoots = new OldTreeRoots(new Fr(0n), new Fr(0n), new Fr(0n), new Fr(0n));
      const contractAddress = AztecAddress.random();
      const abi = ZkTokenContractAbi.functions.find(f => f.name === 'constructor') as unknown as FunctionAbi;

      const txRequest = new TxRequest(
        AztecAddress.random(),
        AztecAddress.ZERO,
        new FunctionData(Buffer.alloc(4), true, true),
        encodeArguments(abi, [140, owner]),
        Fr.random(),
        txContext,
        new Fr(0n),
      );
      const result = await acirSimulator.run(txRequest, abi, contractAddress, EthAddress.ZERO, oldRoots);

      expect(result.preimages.newNotes).toHaveLength(1);
      const newCommitments = result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO));
      expect(newCommitments).toHaveLength(1);

      // TODO get a consistent commitment with noir
      // const [commitment] = newCommitments;
      // expect(commitment).toEqual(
      //   Fr.fromBuffer(
      //     computeCommitment(
      //       acirSimulator.computeNoteHash(result.preimages.newNotes[0].preimage, bbWasm),
      //       computeSlot(new Fr(1n), owner, bbWasm),
      //       contractAddress,
      //       bbWasm,
      //     ),
      //   ),
      // );
    });

    it('should run the mint function', async () => {
      const oldRoots = new OldTreeRoots(new Fr(0n), new Fr(0n), new Fr(0n), new Fr(0n));
      const contractAddress = AztecAddress.random();
      const abi = ZkTokenContractAbi.functions.find(f => f.name === 'mint') as unknown as FunctionAbi;

      const txRequest = new TxRequest(
        AztecAddress.random(),
        contractAddress,
        new FunctionData(Buffer.alloc(4), true, false),
        encodeArguments(abi, [140, owner]),
        Fr.random(),
        txContext,
        new Fr(0n),
      );
      const result = await acirSimulator.run(txRequest, abi, AztecAddress.ZERO, EthAddress.ZERO, oldRoots);

      expect(result.preimages.newNotes).toHaveLength(1);
      expect(result.callStackItem.publicInputs.newCommitments.filter(field => !field.equals(Fr.ZERO))).toHaveLength(1);
    });

    it.skip('should run the transfer function', async () => {
      const db = levelup(createMemDown());
      const pedersen = new Pedersen(bbWasm);

      const contractAddress = AztecAddress.random();
      const amountToTransfer = 100n;
      const abi = ZkTokenContractAbi.functions.find(f => f.name === 'transfer') as unknown as FunctionAbi;

      const tree = await StandardMerkleTree.new(db, pedersen, 'privateData', SIBLING_PATH_SIZE);
      const preimages = [buildNote(60n, owner), buildNote(80n, owner)];
      await tree.appendLeaves(
        preimages.map(preimage =>
          computeCommitment(
            acirSimulator.computeNoteHash(preimage, bbWasm),
            computeSlot(new Fr(1n), owner, bbWasm),
            contractAddress,
            bbWasm,
          ),
        ),
      );

      const oldRoots = new OldTreeRoots(Fr.fromBuffer(tree.getRoot()), new Fr(0n), new Fr(0n), new Fr(0n));

      oracle.getNotes.mockImplementation(() => {
        return Promise.all(
          preimages.map(async (preimage, index) => ({
            preimage,
            siblingPath: (await tree.getSiblingPath(BigInt(index))).data.map(buf => Fr.fromBuffer(buf)),
            index,
          })),
        );
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

      const result = await acirSimulator.run(txRequest, abi, AztecAddress.random(), EthAddress.ZERO, oldRoots);

      console.log(result);
    });
  });
});
