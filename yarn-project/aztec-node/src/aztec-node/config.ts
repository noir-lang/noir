import { ArchiverConfig, getConfigEnvVars as getArchiverVars } from '@aztec/archiver';
import { P2PConfig, getP2PConfigEnvVars } from '@aztec/p2p';
import { SequencerClientConfig, getConfigEnvVars as getSequencerVars } from '@aztec/sequencer-client';
import { getConfigEnvVars as getWorldStateVars } from '@aztec/world-state';

/**
 * The configuration the aztec node.
 */
export type AztecNodeConfig = ArchiverConfig &
  SequencerClientConfig &
  P2PConfig & {
    /** Whether the sequencer is disabled for this node. */
    disableSequencer: boolean;

    /** A URL for an archiver service that the node will use. */
    archiverUrl?: string;
  };

/**
 * Returns the config of the aztec node from environment variables with reasonable defaults.
 * @returns A valid aztec node config.
 */
export function getConfigEnvVars(): AztecNodeConfig {
  const { SEQ_DISABLED } = process.env;
  const allEnvVars: AztecNodeConfig = {
    ...getSequencerVars(),
    ...getArchiverVars(),
    ...getP2PConfigEnvVars(),
    ...getWorldStateVars(),
    disableSequencer: !!SEQ_DISABLED,
    archiverUrl: process.env.ARCHIVER_URL,
  };

  return allEnvVars;
}
