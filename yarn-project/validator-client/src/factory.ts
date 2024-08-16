import { type P2P } from '@aztec/p2p';

import { type ValidatorClientConfig } from './config.js';
import { ValidatorClient } from './validator.js';

export function createValidatorClient(config: ValidatorClientConfig, p2pClient: P2P) {
  return config.disableValidator ? undefined : ValidatorClient.new(config, p2pClient);
}
