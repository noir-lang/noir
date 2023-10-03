import { PublisherConfig, TxSenderConfig } from './config.js';
import { L1Publisher } from './l1-publisher.js';
import { ViemTxSender } from './viem-tx-sender.js';

export { L1Publisher } from './l1-publisher.js';
export { PublisherConfig } from './config.js';

/** Stats logged for each L1 rollup publish tx.*/
export type L1PublishStats = {
  /** Name of the event for metrics purposes */
  eventName: 'rollup-published-to-l1';
  /** Effective gas price of the tx. */
  gasPrice: bigint;
  /** Effective gas used in the tx. */
  gasUsed: bigint;
  /** Hash of the L1 tx. */
  transactionHash: string;
  /** Gas cost of the calldata. */
  calldataGas: number;
  /** Size in bytes of the calldata. */
  calldataSize: number;
  /** Number of txs in the L2 block. */
  txCount: number;
  /** Number of the L2 block. */
  blockNumber: number;
  /** Number of encrypted logs. */
  encryptedLogCount: number;
  /** Number of unencrypted logs. */
  unencryptedLogCount: number;
  /** Serialised size of encrypted logs. */
  encryptedLogSize: number;
  /** Serialised size of unencrypted logs. */
  unencryptedLogSize: number;
};

/**
 * Returns a new instance of the L1Publisher.
 * @param config - Configuration to initialize the new instance.
 */
export function getL1Publisher(config: PublisherConfig & TxSenderConfig): L1Publisher {
  return new L1Publisher(new ViemTxSender(config), config);
}
