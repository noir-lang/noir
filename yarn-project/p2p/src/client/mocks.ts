import { randomBytes } from 'crypto';
import {
  L2BlockSource,
  L2Block,
  ContractData,
  randomContractData,
  randomAppendOnlyTreeSnapshot,
  L2BlockSourceSyncStatus,
} from '@aztec/archiver';
import { AccumulatedTxData, Tx } from './tx.js';

export const MockTx = () => {
  return new Tx(AccumulatedTxData.random());
};

export class MockBlockSource implements L2BlockSource {
  private l2Blocks: L2Block[];

  constructor(private numBlocks = 100) {
    this.l2Blocks = [];
    for (let i = 0; i < this.numBlocks; i++) {
      this.l2Blocks.push(new MockBlock(i));
    }
  }

  public getLatestBlockNum() {
    return Promise.resolve(this.l2Blocks.length - 1);
  }

  public getL2Blocks(from: number, take: number) {
    return Promise.resolve(this.l2Blocks.slice(from, from + take));
  }

  public getSyncStatus(): Promise<L2BlockSourceSyncStatus> {
    return Promise.resolve({
      syncedToBlock: this.numBlocks,
      latestBlock: this.numBlocks,
    } as L2BlockSourceSyncStatus);
  }

  public start(): Promise<void> {
    return Promise.resolve();
  }

  public stop(): Promise<void> {
    return Promise.resolve();
  }
}

export class MockBlock extends L2Block {
  constructor(private _id: number) {
    const newNullifiers = [randomBytes(32), randomBytes(32), randomBytes(32), randomBytes(32)];
    const newCommitments = [randomBytes(32), randomBytes(32), randomBytes(32), randomBytes(32)];
    const newContracts: Buffer[] = [randomBytes(32)];
    const newContractsData: ContractData[] = [randomContractData()];

    super(
      _id,
      randomAppendOnlyTreeSnapshot(0),
      randomAppendOnlyTreeSnapshot(0),
      randomAppendOnlyTreeSnapshot(0),
      randomAppendOnlyTreeSnapshot(0),
      randomAppendOnlyTreeSnapshot(0),
      randomAppendOnlyTreeSnapshot(newCommitments.length),
      randomAppendOnlyTreeSnapshot(newNullifiers.length),
      randomAppendOnlyTreeSnapshot(newContracts.length),
      randomAppendOnlyTreeSnapshot(1),
      randomAppendOnlyTreeSnapshot(1),
      newCommitments,
      newNullifiers,
      newContracts,
      newContractsData,
    );
  }

  get settlementTimestamp() {
    return Date.now();
  }
}
