import { type ArchiverConfig, getArchiverConfigFromEnv } from '@aztec/archiver';
import { type ProverClientConfig, getProverEnvVars } from '@aztec/prover-client';
import { type PublisherConfig, type TxSenderConfig, getTxSenderConfigFromEnv } from '@aztec/sequencer-client';
import { type WorldStateConfig, getWorldStateConfigFromEnv } from '@aztec/world-state';

import { type TxProviderConfig, getTxProviderConfigFromEnv } from './tx-provider/config.js';

export type ProverNodeConfig = ArchiverConfig &
  ProverClientConfig &
  WorldStateConfig &
  PublisherConfig &
  TxSenderConfig &
  TxProviderConfig;

export function getProverNodeConfigFromEnv(): ProverNodeConfig {
  const { PROOF_PUBLISH_RETRY_INTERVAL_MS } = process.env;
  return {
    ...getArchiverConfigFromEnv(),
    ...getProverEnvVars(),
    ...getWorldStateConfigFromEnv(),
    ...getTxSenderConfigFromEnv('PROVER'),
    ...getTxProviderConfigFromEnv(),
    l1PublishRetryIntervalMS: PROOF_PUBLISH_RETRY_INTERVAL_MS ? +PROOF_PUBLISH_RETRY_INTERVAL_MS : 1_000,
  };
}
