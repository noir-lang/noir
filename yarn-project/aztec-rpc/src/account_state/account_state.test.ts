import { AcirSimulator } from '@aztec/acir-simulator';
import { AztecNode } from '@aztec/aztec-node';
import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { KERNEL_NEW_COMMITMENTS_LENGTH } from '@aztec/circuits.js';
import { Point } from '@aztec/foundation';
import { L2Block, L2BlockContext } from '@aztec/l2-block';
import { UnverifiedData } from '@aztec/unverified-data';
import { jest } from '@jest/globals';
import { mock } from 'jest-mock-extended';
import { TxAuxData } from '../aztec_rpc_server/tx_aux_data/index.js';
import { Database, MemoryDB } from '../database/index.js';
import { ConstantKeyPair, KeyPair } from '../key_store/index.js';
import { AccountState } from './account_state.js';

describe('Account State', () => {
  let grumpkin: Grumpkin;
  let database: Database;
  let simulator: AcirSimulator;
  let aztecNode: ReturnType<typeof mock<AztecNode>>;
  let addTxAuxDataBatchSpy: any;
  let accountState: AccountState;
  let owner: KeyPair;

  const createUnverifiedDataAndOwnedTxAuxData = (ownedDataIndices: number[] = []) => {
    ownedDataIndices.forEach(index => {
      if (index >= KERNEL_NEW_COMMITMENTS_LENGTH) {
        throw new Error(`Data index should be less than ${KERNEL_NEW_COMMITMENTS_LENGTH}.`);
      }
    });

    const dataChunks: Buffer[] = [];
    const ownedTxAuxData: TxAuxData[] = [];
    for (let i = 0; i < KERNEL_NEW_COMMITMENTS_LENGTH; ++i) {
      const txAuxData = TxAuxData.random();
      const isOwner = ownedDataIndices.includes(i);
      const publicKey = isOwner ? owner.getPublicKey() : Point.random();
      dataChunks.push(txAuxData.toEncryptedBuffer(publicKey, grumpkin));
      if (isOwner) {
        ownedTxAuxData.push(txAuxData);
      }
    }
    const unverifiedData = new UnverifiedData(dataChunks);
    return { unverifiedData, ownedTxAuxData };
  };

  const mockData = (firstBlockNum: number, ownedData: number[][]) => {
    const blockContexts: L2BlockContext[] = [];
    const unverifiedDatas: UnverifiedData[] = [];
    const ownedTxAuxDatas: TxAuxData[] = [];
    for (let i = 0; i < ownedData.length; ++i) {
      const randomBlockContext = new L2BlockContext(L2Block.random(firstBlockNum + i));
      blockContexts.push(randomBlockContext);
      const { unverifiedData, ownedTxAuxData } = createUnverifiedDataAndOwnedTxAuxData(ownedData[i]);
      unverifiedDatas.push(unverifiedData);
      ownedTxAuxDatas.push(...ownedTxAuxData);
    }
    return { blockContexts, unverifiedDatas, ownedTxAuxDatas };
  };

  beforeAll(async () => {
    const wasm = await BarretenbergWasm.new();
    grumpkin = new Grumpkin(wasm);
    owner = ConstantKeyPair.random(grumpkin);
  });

  beforeEach(async () => {
    database = new MemoryDB();
    addTxAuxDataBatchSpy = jest.spyOn(database, 'addTxAuxDataBatch');

    const ownerPrivateKey = await owner.getPrivateKey();
    aztecNode = mock<AztecNode>();
    simulator = mock<AcirSimulator>();
    accountState = new AccountState(ownerPrivateKey, database, simulator, aztecNode, grumpkin);
  });

  afterEach(() => {
    addTxAuxDataBatchSpy.mockReset();
  });

  it('should store a tx that belong to us', async () => {
    const firstBlockNum = 1;
    const { blockContexts, unverifiedDatas, ownedTxAuxDatas } = mockData(firstBlockNum, [[2]]);
    await accountState.process(blockContexts, unverifiedDatas);

    const txs = await accountState.getTxs();
    expect(txs).toEqual([
      expect.objectContaining({
        blockNumber: 1,
        from: owner.getPublicKey().toAddress(),
      }),
    ]);
    expect(addTxAuxDataBatchSpy).toHaveBeenCalledTimes(1);
    expect(addTxAuxDataBatchSpy).toHaveBeenCalledWith([
      expect.objectContaining({
        ...ownedTxAuxDatas[0],
        index: 2,
      }),
    ]);
  });

  it('should store multiple txs that belong to us', async () => {
    const firstBlockNum = 1;
    const { blockContexts, unverifiedDatas, ownedTxAuxDatas } = mockData(firstBlockNum, [[], [1], [], [], [0, 2], []]);
    await accountState.process(blockContexts, unverifiedDatas);

    const txs = await accountState.getTxs();
    expect(txs).toEqual([
      expect.objectContaining({
        blockNumber: 2,
        from: owner.getPublicKey().toAddress(),
      }),
      expect.objectContaining({
        blockNumber: 5,
        from: owner.getPublicKey().toAddress(),
      }),
    ]);
    expect(addTxAuxDataBatchSpy).toHaveBeenCalledTimes(1);
    expect(addTxAuxDataBatchSpy).toHaveBeenCalledWith([
      expect.objectContaining({
        ...ownedTxAuxDatas[0],
        index: KERNEL_NEW_COMMITMENTS_LENGTH + 1,
      }),
      expect.objectContaining({
        ...ownedTxAuxDatas[1],
        index: KERNEL_NEW_COMMITMENTS_LENGTH * 4,
      }),
      expect.objectContaining({
        ...ownedTxAuxDatas[2],
        index: KERNEL_NEW_COMMITMENTS_LENGTH * 4 + 2,
      }),
    ]);
  });

  it('should not store txs that do not belong to us', async () => {
    const firstBlockNum = 1;
    const { blockContexts, unverifiedDatas } = mockData(firstBlockNum, [[], []]);
    await accountState.process(blockContexts, unverifiedDatas);

    const txs = await accountState.getTxs();
    expect(txs).toEqual([]);
    expect(addTxAuxDataBatchSpy).toHaveBeenCalledTimes(0);
  });

  it('should throw an error if invalid privKey is passed on input', () => {
    const ownerPrivateKey = Buffer.alloc(0);
    expect(() => new AccountState(ownerPrivateKey, database, simulator, aztecNode, grumpkin)).toThrowError();
  });
});
