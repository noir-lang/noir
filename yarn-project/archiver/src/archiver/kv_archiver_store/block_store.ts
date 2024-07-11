import { L2Block, type TxEffect, type TxHash, TxReceipt } from '@aztec/circuit-types';
import { AppendOnlyTreeSnapshot, type AztecAddress, Header, INITIAL_L2_BLOCK_NUM } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { type AztecKVStore, type AztecMap, type AztecSingleton, type Range } from '@aztec/kv-store';

import { type DataRetrieval } from '../data_retrieval.js';
import { type BlockBodyStore } from './block_body_store.js';

type BlockIndexValue = [blockNumber: number, index: number];

type BlockStorage = {
  header: Buffer;
  archive: Buffer;
};

/**
 * LMDB implementation of the ArchiverDataStore interface.
 */
export class BlockStore {
  /** Map block number to block data */
  #blocks: AztecMap<number, BlockStorage>;
  /** Stores L1 block number in which the last processed L2 block was included */
  #lastSynchedL1Block: AztecSingleton<bigint>;

  /** Stores last proven L2 block number */
  #lastProvenL2Block: AztecSingleton<number>;

  /** Index mapping transaction hash (as a string) to its location in a block */
  #txIndex: AztecMap<string, BlockIndexValue>;

  /** Index mapping a contract's address (as a string) to its location in a block */
  #contractIndex: AztecMap<string, BlockIndexValue>;

  #log = createDebugLogger('aztec:archiver:block_store');

  #blockBodyStore: BlockBodyStore;

  constructor(private db: AztecKVStore, blockBodyStore: BlockBodyStore) {
    this.#blockBodyStore = blockBodyStore;

    this.#blocks = db.openMap('archiver_blocks');
    this.#txIndex = db.openMap('archiver_tx_index');
    this.#contractIndex = db.openMap('archiver_contract_index');
    this.#lastSynchedL1Block = db.openSingleton('archiver_last_synched_l1_block');
    this.#lastProvenL2Block = db.openSingleton('archiver_last_proven_l2_block');
  }

  /**
   * Append new blocks to the store's list.
   * @param blocks - The L2 blocks to be added to the store and the last processed L1 block.
   * @returns True if the operation is successful.
   */
  addBlocks(blocks: DataRetrieval<L2Block>): Promise<boolean> {
    return this.db.transaction(() => {
      for (const block of blocks.retrievedData) {
        void this.#blocks.set(block.number, {
          header: block.header.toBuffer(),
          archive: block.archive.toBuffer(),
        });

        block.body.txEffects.forEach((tx, i) => {
          void this.#txIndex.set(tx.txHash.toString(), [block.number, i]);
        });
      }

      void this.#lastSynchedL1Block.set(blocks.lastProcessedL1BlockNumber);

      return true;
    });
  }

  /**
   * Gets up to `limit` amount of L2 blocks starting from `from`.
   * @param start - Number of the first block to return (inclusive).
   * @param limit - The number of blocks to return.
   * @returns The requested L2 blocks
   */
  *getBlocks(start: number, limit: number): IterableIterator<L2Block> {
    for (const blockStorage of this.#blocks.values(this.#computeBlockRange(start, limit))) {
      yield this.getBlockFromBlockStorage(blockStorage);
    }
  }

  /**
   * Gets an L2 block.
   * @param blockNumber - The number of the block to return.
   * @returns The requested L2 block.
   */
  getBlock(blockNumber: number): L2Block | undefined {
    const blockStorage = this.#blocks.get(blockNumber);
    if (!blockStorage || !blockStorage.header) {
      return undefined;
    }

    return this.getBlockFromBlockStorage(blockStorage);
  }

  private getBlockFromBlockStorage(blockStorage: BlockStorage) {
    const header = Header.fromBuffer(blockStorage.header);
    const archive = AppendOnlyTreeSnapshot.fromBuffer(blockStorage.archive);
    const body = this.#blockBodyStore.getBlockBody(header.contentCommitment.txsEffectsHash);

    if (body === undefined) {
      throw new Error('Body is not able to be retrieved from BodyStore');
    }

    return L2Block.fromFields({
      header,
      archive,
      body,
    });
  }

  /**
   * Gets a tx effect.
   * @param txHash - The txHash of the tx corresponding to the tx effect.
   * @returns The requested tx effect (or undefined if not found).
   */
  getTxEffect(txHash: TxHash): TxEffect | undefined {
    const [blockNumber, txIndex] = this.getTxLocation(txHash) ?? [];
    if (typeof blockNumber !== 'number' || typeof txIndex !== 'number') {
      return undefined;
    }

    const block = this.getBlock(blockNumber);
    return block?.body.txEffects[txIndex];
  }

  /**
   * Gets a receipt of a settled tx.
   * @param txHash - The hash of a tx we try to get the receipt for.
   * @returns The requested tx receipt (or undefined if not found).
   */
  getSettledTxReceipt(txHash: TxHash): TxReceipt | undefined {
    const [blockNumber, txIndex] = this.getTxLocation(txHash) ?? [];
    if (typeof blockNumber !== 'number' || typeof txIndex !== 'number') {
      return undefined;
    }

    const block = this.getBlock(blockNumber)!;
    const tx = block.body.txEffects[txIndex];

    return new TxReceipt(
      txHash,
      TxReceipt.statusFromRevertCode(tx.revertCode),
      '',
      tx.transactionFee.toBigInt(),
      block.hash().toBuffer(),
      block.number,
    );
  }

  /**
   * Looks up which block included the requested tx effect.
   * @param txHash - The txHash of the tx.
   * @returns The block number and index of the tx.
   */
  getTxLocation(txHash: TxHash): [blockNumber: number, txIndex: number] | undefined {
    return this.#txIndex.get(txHash.toString());
  }

  /**
   * Looks up which block deployed a particular contract.
   * @param contractAddress - The address of the contract to look up.
   * @returns The block number and index of the contract.
   */
  getContractLocation(contractAddress: AztecAddress): [blockNumber: number, index: number] | undefined {
    return this.#contractIndex.get(contractAddress.toString());
  }

  /**
   * Gets the number of the latest L2 block processed.
   * @returns The number of the latest L2 block processed.
   */
  getSynchedL2BlockNumber(): number {
    const [lastBlockNumber] = this.#blocks.keys({ reverse: true, limit: 1 });
    return typeof lastBlockNumber === 'number' ? lastBlockNumber : INITIAL_L2_BLOCK_NUM - 1;
  }

  /**
   * Gets the most recent L1 block processed.
   * @returns The L1 block that published the latest L2 block
   */
  getSynchedL1BlockNumber(): bigint {
    return this.#lastSynchedL1Block.get() ?? 0n;
  }

  getProvenL2BlockNumber(): number {
    return this.#lastProvenL2Block.get() ?? 0;
  }

  async setProvenL2BlockNumber(blockNumber: number) {
    await this.#lastProvenL2Block.set(blockNumber);
  }

  #computeBlockRange(start: number, limit: number): Required<Pick<Range<number>, 'start' | 'end'>> {
    if (limit < 1) {
      throw new Error(`Invalid limit: ${limit}`);
    }

    if (start < INITIAL_L2_BLOCK_NUM) {
      start = INITIAL_L2_BLOCK_NUM;
    }

    const end = start + limit;
    return { start, end };
  }
}
