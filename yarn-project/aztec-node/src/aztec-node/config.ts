import { ArchiverConfig, getConfigEnvVars as getArchiverVars } from '@aztec/archiver';
import { SequencerClientConfig, getConfigEnvVars as getSequencerVars } from '@aztec/sequencer-client';

/**
 * The configuration the aztec node.
 */
export type AztecNodeConfig = ArchiverConfig & SequencerClientConfig;

/**
 * Returns the config of the aztec node from environment variables with reasonable defaults.
 * @returns A valid aztec node config.
 */
export function getConfigEnvVars(): AztecNodeConfig {
  const allEnvVars: AztecNodeConfig = {
    ...getSequencerVars(),
    ...getArchiverVars(),
  };

  return allEnvVars;
}
