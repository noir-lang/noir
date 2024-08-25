import { type P2P } from '@aztec/p2p';

import { generatePrivateKey } from 'viem/accounts';

import { type ValidatorClientConfig } from './config.js';
import { ValidatorClient } from './validator.js';

export function createValidatorClient(config: ValidatorClientConfig, p2pClient: P2P) {
  if (config.disableValidator) {
    return undefined;
  }
  // TODO: should this be exposed via a flag?
  if (config.validatorPrivateKey === undefined || config.validatorPrivateKey === '') {
    config.validatorPrivateKey = generatePrivateKey();
  }
  return ValidatorClient.new(config, p2pClient);
}
