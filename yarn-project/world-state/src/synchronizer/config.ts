import { type ConfigMappingsType, getConfigFromMappings } from '@aztec/foundation/config';

/** World State synchronizer configuration values. */
export interface WorldStateConfig {
  /** The frequency in which to check. */
  worldStateBlockCheckIntervalMS: number;

  /** Size of queue of L2 blocks to store. */
  l2QueueSize: number;

  /** Whether to follow only the proven chain. */
  worldStateProvenBlocksOnly: boolean;
}

export const worldStateConfigMappings: ConfigMappingsType<WorldStateConfig> = {
  worldStateBlockCheckIntervalMS: {
    env: 'WS_BLOCK_CHECK_INTERVAL_MS',
    parseEnv: (val: string) => +val,
    default: 100,
    description: 'The frequency in which to check.',
  },
  l2QueueSize: {
    env: 'WS_L2_BLOCK_QUEUE_SIZE',
    parseEnv: (val: string) => +val,
    default: 1000,
    description: 'Size of queue of L2 blocks to store.',
  },
  worldStateProvenBlocksOnly: {
    env: 'WS_PROVEN_BLOCKS_ONLY',
    parseEnv: (val: string) => ['1', 'true'].includes(val),
    default: false,
    description: 'Whether to follow only the proven chain.',
  },
};

/**
 * Returns the configuration values for the world state synchronizer.
 * @returns The configuration values for the world state synchronizer.
 */
export function getWorldStateConfigFromEnv(): WorldStateConfig {
  return getConfigFromMappings<WorldStateConfig>(worldStateConfigMappings);
}
