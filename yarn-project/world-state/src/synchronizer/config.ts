/** World State synchronizer configuration values. */
export interface WorldStateConfig {
  /** The frequency in which to check. */
  worldStateBlockCheckIntervalMS: number;

  /** Size of queue of L2 blocks to store. */
  l2QueueSize: number;

  /** Whether to follow only the proven chain. */
  worldStateProvenBlocksOnly: boolean;
}

/**
 * Returns the configuration values for the world state synchronizer.
 * @returns The configuration values for the world state synchronizer.
 */
export function getWorldStateConfigFromEnv(): WorldStateConfig {
  const { WS_BLOCK_CHECK_INTERVAL_MS, WS_L2_BLOCK_QUEUE_SIZE, WS_PROVEN_BLOCKS_ONLY } = process.env;
  const envVars: WorldStateConfig = {
    worldStateBlockCheckIntervalMS: WS_BLOCK_CHECK_INTERVAL_MS ? +WS_BLOCK_CHECK_INTERVAL_MS : 100,
    l2QueueSize: WS_L2_BLOCK_QUEUE_SIZE ? +WS_L2_BLOCK_QUEUE_SIZE : 1000,
    worldStateProvenBlocksOnly: ['1', 'true'].includes(WS_PROVEN_BLOCKS_ONLY!),
  };
  return envVars;
}
