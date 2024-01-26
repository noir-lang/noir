import { INITIAL_L2_BLOCK_NUM, L2Block, L2Tx, TxHash } from '@aztec/circuit-types';
import { AztecAddress } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { AztecKVStore, AztecMap, Range } from '@aztec/kv-store';

type BlockIndexValue = [blockNumber: number, index: number];

type BlockContext = {
  blockNumber: number;
  l1BlockNumber: bigint;
  block: Buffer;
  blockHash: Buffer;
};

/**
 * LMDB implementation of the ArchiverDataStore interface.
 */
export class BlockStore {
  /** Map block number to block data */
  #blocks: AztecMap<number, BlockContext>;

  /** Index mapping transaction hash (as a string) to its location in a block */
  #txIndex: AztecMap<string, BlockIndexValue>;

  /** Index mapping a contract's address (as a string) to its location in a block */
  #contractIndex: AztecMap<string, BlockIndexValue>;

  #log = createDebugLogger('aztec:archiver:block_store');

  constructor(private db: AztecKVStore) {
    this.#blocks = db.openMap('archiver_blocks');

    this.#txIndex = db.openMap('archiver_tx_index');
    this.#contractIndex = db.openMap('archiver_contract_index');
  }

  /**
   * Append new blocks to the store's list.
   * @param blocks - The L2 blocks to be added to the store.
   * @returns True if the operation is successful.
   */
  addBlocks(blocks: L2Block[]): Promise<boolean> {
    return this.db.transaction(() => {
      for (const block of blocks) {
        void this.#blocks.set(block.number, {
          blockNumber: block.number,
          block: block.toBuffer(),
          l1BlockNumber: block.getL1BlockNumber(),
          blockHash: block.getBlockHash(),
        });

        for (const [i, tx] of block.getTxs().entries()) {
          if (tx.txHash.isZero()) {
            continue;
          }
          void this.#txIndex.set(tx.txHash.toString(), [block.number, i]);
        }

        for (const [i, contractData] of block.newContractData.entries()) {
          if (contractData.contractAddress.isZero()) {
            continue;
          }

          void this.#contractIndex.set(contractData.contractAddress.toString(), [block.number, i]);
        }
      }

      return true;
    });
  }

  /**
   * Gets up to `limit` amount of L2 blocks starting from `from`.
   * @param start - Number of the first block to return (inclusive).
   * @param limit - The number of blocks to return.
   * @returns The requested L2 blocks, without logs attached
   */
  *getBlocks(start: number, limit: number): IterableIterator<L2Block> {
    for (const blockCtx of this.#blocks.values(this.#computeBlockRange(start, limit))) {
      yield L2Block.fromBuffer(blockCtx.block, blockCtx.blockHash);
    }
  }

  /**
   * Gets an L2 block.
   * @param blockNumber - The number of the block to return.
   * @returns The requested L2 block, without logs attached
   */
  getBlock(blockNumber: number): L2Block | undefined {
    const blockCtx = this.#blocks.get(blockNumber);
    if (!blockCtx || !blockCtx.block) {
      return undefined;
    }

    const block = L2Block.fromBuffer(blockCtx.block, blockCtx.blockHash);

    return block;
  }

  /**
   * Gets an l2 tx.
   * @param txHash - The txHash of the l2 tx.
   * @returns The requested L2 tx.
   */
  getL2Tx(txHash: TxHash): L2Tx | undefined {
    const [blockNumber, txIndex] = this.getL2TxLocation(txHash) ?? [];
    if (typeof blockNumber !== 'number' || typeof txIndex !== 'number') {
      return undefined;
    }

    const block = this.getBlock(blockNumber);
    return block?.getTx(txIndex);
  }

  /**
   * Looks up which block included the requested L2 tx.
   * @param txHash - The txHash of the l2 tx.
   * @returns The block number and index of the tx.
   */
  getL2TxLocation(txHash: TxHash): [blockNumber: number, txIndex: number] | undefined {
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
  getBlockNumber(): number {
    const [lastBlockNumber] = this.#blocks.keys({ reverse: true, limit: 1 });
    return typeof lastBlockNumber === 'number' ? lastBlockNumber : INITIAL_L2_BLOCK_NUM - 1;
  }

  /**
   * Gets the most recent L1 block processed.
   * @returns The L1 block that published the latest L2 block
   */
  getL1BlockNumber(): bigint {
    const [lastBlock] = this.#blocks.values({ reverse: true, limit: 1 });
    if (!lastBlock) {
      return 0n;
    } else {
      return lastBlock.l1BlockNumber;
    }
  }

  #computeBlockRange(start: number, limit: number): Required<Pick<Range<number>, 'start' | 'end'>> {
    if (limit < 1) {
      throw new Error(`Invalid limit: ${limit}`);
    }

    if (start < INITIAL_L2_BLOCK_NUM) {
      this.#log(`Clamping start block ${start} to ${INITIAL_L2_BLOCK_NUM}`);
      start = INITIAL_L2_BLOCK_NUM;
    }

    const end = start + limit;
    return { start, end };
  }
}
