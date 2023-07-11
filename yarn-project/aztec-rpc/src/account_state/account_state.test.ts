import { AztecNode } from '@aztec/aztec-node';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { AztecAddress, CircuitsWasm, MAX_NEW_COMMITMENTS_PER_TX } from '@aztec/circuits.js';
import { Fr, Point } from '@aztec/foundation/fields';
import { ConstantKeyPair, KeyPair, KeyStore } from '@aztec/key-store';
import { FunctionL2Logs, L2Block, L2BlockContext, L2BlockL2Logs, NoteSpendingInfo, TxL2Logs } from '@aztec/types';
import { jest } from '@jest/globals';
import { MockProxy, mock } from 'jest-mock-extended';
import { Database, MemoryDB } from '../database/index.js';
import { AccountState } from './account_state.js';
import { SchnorrAccountContractAbi } from '@aztec/noir-contracts/examples';

describe('Account State', () => {
  let grumpkin: Grumpkin;
  let database: Database;
  let aztecNode: ReturnType<typeof mock<AztecNode>>;
  let addNoteSpendingInfoBatchSpy: any;
  let accountState: AccountState;
  let owner: KeyPair;
  let ownerAddress: AztecAddress;
  let keyStore: MockProxy<KeyStore>;

  const createEncryptedLogsAndOwnedNoteSpendingInfo = (ownedDataIndices: number[] = []) => {
    ownedDataIndices.forEach(index => {
      if (index >= MAX_NEW_COMMITMENTS_PER_TX) {
        throw new Error(`Data index should be less than ${MAX_NEW_COMMITMENTS_PER_TX}.`);
      }
    });

    const txLogs: TxL2Logs[] = [];
    const ownedNoteSpendingInfo: NoteSpendingInfo[] = [];
    for (let i = 0; i < MAX_NEW_COMMITMENTS_PER_TX; ++i) {
      const noteSpendingInfo = NoteSpendingInfo.random();
      const isOwner = ownedDataIndices.includes(i);
      const publicKey = isOwner ? owner.getPublicKey() : Point.random();
      const log = noteSpendingInfo.toEncryptedBuffer(publicKey, grumpkin);
      // 1 tx containing 1 function invocation containing 1 log
      txLogs.push(new TxL2Logs([new FunctionL2Logs([log])]));
      if (isOwner) {
        ownedNoteSpendingInfo.push(noteSpendingInfo);
      }
    }
    const encryptedLogs = new L2BlockL2Logs(txLogs);
    return { encryptedLogs, ownedNoteSpendingInfo };
  };

  const mockData = (firstBlockNum: number, ownedData: number[][]) => {
    const blockContexts: L2BlockContext[] = [];
    const encryptedLogsArr: L2BlockL2Logs[] = [];
    const ownedNoteSpendingInfos: NoteSpendingInfo[] = [];
    for (let i = 0; i < ownedData.length; ++i) {
      const randomBlockContext = new L2BlockContext(L2Block.random(firstBlockNum + i));
      blockContexts.push(randomBlockContext);
      const { encryptedLogs, ownedNoteSpendingInfo } = createEncryptedLogsAndOwnedNoteSpendingInfo(ownedData[i]);
      encryptedLogsArr.push(encryptedLogs);
      ownedNoteSpendingInfos.push(...ownedNoteSpendingInfo);
    }
    return { blockContexts, encryptedLogsArr, ownedNoteSpendingInfos };
  };

  beforeAll(async () => {
    const wasm = await CircuitsWasm.get();
    grumpkin = new Grumpkin(wasm);
    owner = ConstantKeyPair.random(grumpkin);
  });

  beforeEach(() => {
    database = new MemoryDB();
    addNoteSpendingInfoBatchSpy = jest.spyOn(database, 'addNoteSpendingInfoBatch');

    ownerAddress = AztecAddress.random();
    const partialAccountContractAddress = Fr.random();
    aztecNode = mock<AztecNode>();
    keyStore = mock<KeyStore>();
    keyStore.getAccountPrivateKey.mockResolvedValue(owner.getPrivateKey());
    accountState = new AccountState(
      owner.getPublicKey(),
      keyStore,
      ownerAddress,
      partialAccountContractAddress,
      database,
      aztecNode,
      SchnorrAccountContractAbi,
    );
  });

  afterEach(() => {
    addNoteSpendingInfoBatchSpy.mockReset();
  });

  it('should store a tx that belong to us', async () => {
    const firstBlockNum = 1;
    const { blockContexts, encryptedLogsArr, ownedNoteSpendingInfos } = mockData(firstBlockNum, [[2]]);
    await accountState.process(blockContexts, encryptedLogsArr);

    const txs = await accountState.getTxs();
    expect(txs).toEqual([
      expect.objectContaining({
        blockNumber: 1,
        from: ownerAddress,
      }),
    ]);
    expect(addNoteSpendingInfoBatchSpy).toHaveBeenCalledTimes(1);
    expect(addNoteSpendingInfoBatchSpy).toHaveBeenCalledWith([
      expect.objectContaining({
        ...ownedNoteSpendingInfos[0],
        index: 2n,
      }),
    ]);
  });

  it('should store multiple txs that belong to us', async () => {
    const firstBlockNum = 1;
    const { blockContexts, encryptedLogsArr, ownedNoteSpendingInfos } = mockData(firstBlockNum, [
      [],
      [1],
      [],
      [],
      [0, 2],
      [],
    ]);
    await accountState.process(blockContexts, encryptedLogsArr);

    const txs = await accountState.getTxs();
    expect(txs).toEqual([
      expect.objectContaining({
        blockNumber: 2,
        from: ownerAddress,
      }),
      expect.objectContaining({
        blockNumber: 5,
        from: ownerAddress,
      }),
      expect.objectContaining({
        blockNumber: 5,
        from: ownerAddress,
      }),
    ]);
    expect(addNoteSpendingInfoBatchSpy).toHaveBeenCalledTimes(1);
    expect(addNoteSpendingInfoBatchSpy).toHaveBeenCalledWith([
      expect.objectContaining({
        ...ownedNoteSpendingInfos[0],
        index: BigInt(MAX_NEW_COMMITMENTS_PER_TX + 1),
      }),
      expect.objectContaining({
        ...ownedNoteSpendingInfos[1],
        index: BigInt(MAX_NEW_COMMITMENTS_PER_TX * 4),
      }),
      expect.objectContaining({
        ...ownedNoteSpendingInfos[2],
        index: BigInt(MAX_NEW_COMMITMENTS_PER_TX * 4 + 2),
      }),
    ]);
  });

  it('should not store txs that do not belong to us', async () => {
    const firstBlockNum = 1;
    const { blockContexts, encryptedLogsArr } = mockData(firstBlockNum, [[], []]);
    await accountState.process(blockContexts, encryptedLogsArr);

    const txs = await accountState.getTxs();
    expect(txs).toEqual([]);
    expect(addNoteSpendingInfoBatchSpy).toHaveBeenCalledTimes(0);
  });
});
