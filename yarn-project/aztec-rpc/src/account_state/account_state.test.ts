import { AztecNode } from '@aztec/aztec-node';
import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { KERNEL_NEW_COMMITMENTS_LENGTH } from '@aztec/circuits.js';
import { Point } from '@aztec/foundation';
import { L2Block, UnverifiedData } from '@aztec/l2-block';
import { jest } from '@jest/globals';
import { mock } from 'jest-mock-extended';
import { TxAuxData } from '../aztec_rpc_server/tx_aux_data/index.js';
import { Database, MemoryDB } from '../database/index.js';
import { ConstantKeyPair, KeyPair } from '../key_store/index.js';
import { AccountState } from './account_state.js';

describe('Account State', () => {
  let grumpkin: Grumpkin;
  let aztecNode: ReturnType<typeof mock<AztecNode>>;
  let database: Database;
  let addTxAuxDataBatchSpy: any;
  let accountState: AccountState;
  let owner: KeyPair;
  let ownedTxAuxData: TxAuxData[] = [];
  let publishedUnverifiedData: UnverifiedData[] = [];
  let publishedBlocks: L2Block[] = [];

  const createUnverifiedData = (ownedDataIndices: number[] = []): UnverifiedData => {
    ownedDataIndices.forEach(index => {
      if (index >= KERNEL_NEW_COMMITMENTS_LENGTH) {
        throw new Error(`Data index should be less than ${KERNEL_NEW_COMMITMENTS_LENGTH}.`);
      }
    });

    const dataChunks: Buffer[] = [];
    for (let i = 0; i < KERNEL_NEW_COMMITMENTS_LENGTH; ++i) {
      const txAuxData = TxAuxData.random();
      const isOwner = ownedDataIndices.includes(i);
      const publicKey = isOwner ? owner.getPublicKey() : Point.random();
      dataChunks.push(txAuxData.toEncryptedBuffer(publicKey, grumpkin));
      if (isOwner) {
        ownedTxAuxData.push(txAuxData);
      }
    }
    return new UnverifiedData(dataChunks);
  };

  const publishBlocks = (ownedData: (number[] | undefined)[] = []) => {
    for (let i = 0; i < ownedData.length; ++i) {
      const blockNumber = publishedBlocks.length + 1;
      publishedBlocks.push(L2Block.random(blockNumber));
      publishedUnverifiedData.push(createUnverifiedData(ownedData[i]));
    }
  };

  const getUnverifiedData = (from: number, take: number) => publishedUnverifiedData.slice(from - 1, from - 1 + take);

  beforeAll(async () => {
    const wasm = await BarretenbergWasm.new();
    grumpkin = new Grumpkin(wasm);
    owner = ConstantKeyPair.random(grumpkin);
  });

  beforeEach(async () => {
    ownedTxAuxData = [];
    publishedUnverifiedData = [];
    publishedBlocks = [];

    database = new MemoryDB();
    addTxAuxDataBatchSpy = jest.spyOn(database, 'addTxAuxDataBatch');

    aztecNode = mock<AztecNode>();
    aztecNode.getBlocks.mockImplementation((from, take) =>
      Promise.resolve(publishedBlocks.slice(from - 1, from - 1 + take)),
    );

    const ownerPrivateKey = await owner.getPrivateKey();
    accountState = new AccountState(ownerPrivateKey, database, aztecNode, grumpkin);
  });

  afterEach(() => {
    addTxAuxDataBatchSpy.mockReset();
  });

  it('should store a tx that belong to us', async () => {
    publishBlocks([[2]]);

    const from = 1;
    const take = 10;
    const unverifiedData = getUnverifiedData(from, take);
    await accountState.processUnverifiedData(unverifiedData, from, take);

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
        ...ownedTxAuxData[0],
        index: 2,
      }),
    ]);
  });

  it('should store multiple txs that belong to us', async () => {
    publishBlocks([[], [1], [], [], [0, 2], []]);

    const from = 1;
    const take = 10;
    const unverifiedData = getUnverifiedData(from, take);
    await accountState.processUnverifiedData(unverifiedData, from, take);

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
        ...ownedTxAuxData[0],
        index: KERNEL_NEW_COMMITMENTS_LENGTH + 1,
      }),
      expect.objectContaining({
        ...ownedTxAuxData[1],
        index: KERNEL_NEW_COMMITMENTS_LENGTH * 4,
      }),
      expect.objectContaining({
        ...ownedTxAuxData[2],
        index: KERNEL_NEW_COMMITMENTS_LENGTH * 4 + 2,
      }),
    ]);
  });

  it('should not store txs that do not belong to us', async () => {
    publishBlocks([[], []]);

    const from = 1;
    const take = 10;
    const unverifiedData = getUnverifiedData(from, take);
    await accountState.processUnverifiedData(unverifiedData, from, take);

    const txs = await accountState.getTxs();
    expect(txs).toEqual([]);
    expect(addTxAuxDataBatchSpy).toHaveBeenCalledTimes(0);
  });

  it('should throw an error if invalid privKey is passed on input', () => {
    const ownerPrivateKey = Buffer.alloc(0);
    expect(() => new AccountState(ownerPrivateKey, database, aztecNode, grumpkin)).toThrowError();
  });
});
