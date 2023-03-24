import { PublisherConfig, TxSenderConfig } from './config.js';
import { EthereumjsTxSender } from './ethereumjs-tx-sender.js';
import { L2BlockPublisher } from './l2-block-publisher.js';

export { L2BlockPublisher } from './l2-block-publisher.js';
export { PublisherConfig } from './config.js';

export function getL2BlockPublisher(config: PublisherConfig & TxSenderConfig): L2BlockPublisher {
  return new L2BlockPublisher(new EthereumjsTxSender(config), config);
}
