import { type ArchiverConfig, archiverConfigMappings, getArchiverConfigFromEnv } from '@aztec/archiver';
import { type ConfigMappingsType } from '@aztec/foundation/config';
import { type ProverClientConfig, getProverEnvVars, proverClientConfigMappings } from '@aztec/prover-client';
import {
  type PublisherConfig,
  type TxSenderConfig,
  getPublisherConfigFromEnv,
  getPublisherConfigMappings,
  getTxSenderConfigFromEnv,
  getTxSenderConfigMappings,
} from '@aztec/sequencer-client';
import { type WorldStateConfig, getWorldStateConfigFromEnv, worldStateConfigMappings } from '@aztec/world-state';

import { type TxProviderConfig, getTxProviderConfigFromEnv, txProviderConfigMappings } from './tx-provider/config.js';

export type ProverNodeConfig = ArchiverConfig &
  ProverClientConfig &
  WorldStateConfig &
  PublisherConfig &
  TxSenderConfig &
  TxProviderConfig;

export const proverNodeConfigMappings: ConfigMappingsType<ProverNodeConfig> = {
  ...archiverConfigMappings,
  ...proverClientConfigMappings,
  ...worldStateConfigMappings,
  ...getPublisherConfigMappings('PROVER'),
  ...getTxSenderConfigMappings('PROVER'),
  ...txProviderConfigMappings,
};

export function getProverNodeConfigFromEnv(): ProverNodeConfig {
  return {
    ...getArchiverConfigFromEnv(),
    ...getProverEnvVars(),
    ...getWorldStateConfigFromEnv(),
    ...getPublisherConfigFromEnv('PROVER'),
    ...getTxSenderConfigFromEnv('PROVER'),
    ...getTxProviderConfigFromEnv(),
  };
}
