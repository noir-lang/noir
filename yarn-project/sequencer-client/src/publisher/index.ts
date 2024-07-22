import { type PublisherConfig, type TxSenderConfig } from './config.js';
import { L1Publisher } from './l1-publisher.js';
import { ViemTxSender } from './viem-tx-sender.js';

export { L1Publisher } from './l1-publisher.js';
export { PublisherConfig, TxSenderConfig, getTxSenderConfigFromEnv } from './config.js';

/**
 * Returns a new instance of the L1Publisher.
 * @param config - Configuration to initialize the new instance.
 */
export function getL1Publisher(config: PublisherConfig & TxSenderConfig): L1Publisher {
  return new L1Publisher(new ViemTxSender(config), config);
}
