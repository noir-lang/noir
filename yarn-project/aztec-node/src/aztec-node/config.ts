import { type ArchiverConfig, archiverConfigMappings } from '@aztec/archiver';
import { type ConfigMappingsType, booleanConfigHelper, getConfigFromMappings } from '@aztec/foundation/config';
import { type P2PConfig, p2pConfigMappings } from '@aztec/p2p';
import { type ProverClientConfig, proverClientConfigMappings } from '@aztec/prover-client';
import { type SequencerClientConfig, sequencerClientConfigMappings } from '@aztec/sequencer-client';
import { type WorldStateConfig, worldStateConfigMappings } from '@aztec/world-state';

import { readFileSync } from 'fs';
import { dirname, resolve } from 'path';
import { fileURLToPath } from 'url';

export { sequencerClientConfigMappings, SequencerClientConfig } from '@aztec/sequencer-client';

/**
 * The configuration the aztec node.
 */
export type AztecNodeConfig = ArchiverConfig &
  SequencerClientConfig &
  ProverClientConfig &
  WorldStateConfig &
  Pick<ProverClientConfig, 'bbBinaryPath' | 'bbWorkingDirectory' | 'realProofs'> &
  P2PConfig & {
    /** Whether the sequencer is disabled for this node. */
    disableSequencer: boolean;
  };

export const aztecNodeConfigMappings: ConfigMappingsType<AztecNodeConfig> = {
  ...archiverConfigMappings,
  ...sequencerClientConfigMappings,
  ...proverClientConfigMappings,
  ...worldStateConfigMappings,
  ...p2pConfigMappings,
  disableSequencer: {
    env: 'SEQ_DISABLED',
    description: 'Whether the sequencer is disabled for this node.',
    ...booleanConfigHelper(),
  },
};

/**
 * Returns the config of the aztec node from environment variables with reasonable defaults.
 * @returns A valid aztec node config.
 */
export function getConfigEnvVars(): AztecNodeConfig {
  return getConfigFromMappings<AztecNodeConfig>(aztecNodeConfigMappings);
}

/**
 * Returns package name and version.
 */
export function getPackageInfo() {
  const packageJsonPath = resolve(dirname(fileURLToPath(import.meta.url)), '../../package.json');
  const { version, name } = JSON.parse(readFileSync(packageJsonPath).toString());
  return { version, name };
}
