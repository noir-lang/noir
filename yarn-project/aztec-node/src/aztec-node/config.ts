import { ArchiverConfig, getConfigEnvVars as getArchiverVars } from '@aztec/archiver';
import { SequencerClientConfig, getConfigEnvVars as getSequencerVars } from '@aztec/sequencer-client';
import { P2PConfig, getP2PConfigEnvVars } from '@aztec/p2p';

/**
 * The configuration the aztec node.
 */
export type AztecNodeConfig = ArchiverConfig & SequencerClientConfig & P2PConfig;

/**
 * Returns the config of the aztec node from environment variables with reasonable defaults.
 * @returns A valid aztec node config.
 */
export function getConfigEnvVars(): AztecNodeConfig {
  const allEnvVars: AztecNodeConfig = {
    ...getSequencerVars(),
    ...getArchiverVars(),
    ...getP2PConfigEnvVars(),
  };

  return allEnvVars;
}
