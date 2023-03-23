import { Config } from '../config.js';
import { AztecEthereumjsTxSender } from './aztec-ethereumjs-tx-sender.js';
import { L2BlockPublisher } from './l2-block-publisher.js';

export { L2BlockPublisher } from './l2-block-publisher.js';

export function getL2BlockPublisher(config: Config): L2BlockPublisher {
  return new L2BlockPublisher(new AztecEthereumjsTxSender(config));
}
