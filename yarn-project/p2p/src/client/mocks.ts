import { L2Block, type L2BlockSource, type TxEffect, type TxHash, TxReceipt, TxStatus } from '@aztec/circuit-types';
import { EthAddress } from '@aztec/circuits.js';

/**
 * A mocked implementation of L2BlockSource to be used in p2p tests.
 */
export class MockBlockSource implements L2BlockSource {
  private l2Blocks: L2Block[] = [];
  private txEffects: TxEffect[] = [];

  constructor(private numBlocks = 100) {
    for (let i = 0; i < this.numBlocks; i++) {
      const block = L2Block.random(i);
      this.l2Blocks.push(block);
      this.txEffects.push(...block.body.txEffects);
    }
  }

  /**
   * Method to fetch the rollup contract address at the base-layer.
   * @returns The rollup address.
   */
  getRollupAddress(): Promise<EthAddress> {
    return Promise.resolve(EthAddress.random());
  }

  /**
   * Method to fetch the registry contract address at the base-layer.
   * @returns The registry address.
   */
  getRegistryAddress(): Promise<EthAddress> {
    return Promise.resolve(EthAddress.random());
  }

  /**
   * Gets the number of the latest L2 block processed by the block source implementation.
   * @returns In this mock instance, returns the number of L2 blocks that we've mocked.
   */
  public getBlockNumber() {
    return Promise.resolve(this.l2Blocks.length - 1);
  }

  public getProvenBlockNumber(): Promise<number> {
    return this.getBlockNumber();
  }

  /**
   * Gets an l2 block.
   * @param number - The block number to return (inclusive).
   * @returns The requested L2 block.
   */
  public getBlock(number: number) {
    return Promise.resolve(this.l2Blocks[number]);
  }

  /**
   * Gets up to `limit` amount of L2 blocks starting from `from`.
   * @param from - Number of the first block to return (inclusive).
   * @param limit - The maximum number of blocks to return.
   * @returns The requested mocked L2 blocks.
   */
  public getBlocks(from: number, limit: number) {
    return Promise.resolve(this.l2Blocks.slice(from, from + limit));
  }

  /**
   * Gets a tx effect.
   * @param txHash - The hash of a transaction which resulted in the returned tx effect.
   * @returns The requested tx effect.
   */
  public getTxEffect(txHash: TxHash) {
    const txEffect = this.txEffects.find(tx => tx.txHash.equals(txHash));
    return Promise.resolve(txEffect);
  }

  /**
   * Gets a receipt of a settled tx.
   * @param txHash - The hash of a tx we try to get the receipt for.
   * @returns The requested tx receipt (or undefined if not found).
   */
  public getSettledTxReceipt(txHash: TxHash): Promise<TxReceipt | undefined> {
    for (const block of this.l2Blocks) {
      for (const txEffect of block.body.txEffects) {
        if (txEffect.txHash.equals(txHash)) {
          return Promise.resolve(
            new TxReceipt(
              txHash,
              TxStatus.SUCCESS,
              '',
              txEffect.transactionFee.toBigInt(),
              block.hash().toBuffer(),
              block.number,
            ),
          );
        }
      }
    }
    return Promise.resolve(undefined);
  }

  /**
   * Starts the block source. In this mock implementation, this is a noop.
   * @returns A promise that signals the initialization of the l2 block source on completion.
   */
  public start(): Promise<void> {
    return Promise.resolve();
  }

  /**
   * Stops the block source. In this mock implementation, this is a noop.
   * @returns A promise that signals the l2 block source is now stopped.
   */
  public stop(): Promise<void> {
    return Promise.resolve();
  }
}
