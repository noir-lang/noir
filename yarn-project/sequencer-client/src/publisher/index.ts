import { PublisherConfig, TxSenderConfig } from './config.js';
// import { EthereumjsTxSender } from './ethereumjs-tx-sender.js';
import { L1Publisher } from './l1-publisher.js';
import { ViemTxSender } from './viem-tx-sender.js';

export { L1Publisher } from './l1-publisher.js';
export { PublisherConfig } from './config.js';

export function getL1Publisher(config: PublisherConfig & TxSenderConfig): L1Publisher {
  return new L1Publisher(new ViemTxSender(config), config);
}
