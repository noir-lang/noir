import { EthAddress } from '@aztec/circuits.js';
import { L2Block, L2BlockSource, L2Tx, TxHash } from '@aztec/types';

/**
 * A mocked implementation of L2BlockSource to be used in p2p tests.
 */
export class MockBlockSource implements L2BlockSource {
  private l2Blocks: L2Block[] = [];
  private l2Txs: L2Tx[] = [];

  constructor(private numBlocks = 100) {
    for (let i = 0; i < this.numBlocks; i++) {
      const block = L2Block.random(i);
      this.l2Blocks.push(block);
      this.l2Txs.push(...block.getTxs());
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

  /**
   * Gets an l2 block.
   * @param number - The block number to return (inclusive).
   * @returns The requested L2 block.
   */
  public getL2Block(number: number) {
    return Promise.resolve(this.l2Blocks[number]);
  }

  /**
   * Gets up to `limit` amount of L2 blocks starting from `from`.
   * @param from - Number of the first block to return (inclusive).
   * @param limit - The maximum number of blocks to return.
   * @returns The requested mocked L2 blocks.
   */
  public getL2Blocks(from: number, limit: number) {
    return Promise.resolve(this.l2Blocks.slice(from, from + limit));
  }

  /**
   * Gets an l2 tx.
   * @param txHash - The txHash of the l2 tx.
   * @returns The requested L2 tx.
   */
  getL2Tx(txHash: TxHash) {
    const l2Tx = this.l2Txs.find(tx => tx.txHash.equals(txHash));
    return Promise.resolve(l2Tx);
  }

  /**
   * Starts the block source. In this mock implementation, this is a noop.
   * @returns A promise that signals the initialization of the l2 block source on compmletion.
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
