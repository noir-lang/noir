/**
 * P2P client configuration values.
 */
export interface P2PConfig {
  /**
   * The frequency in which to check.
   */
  checkInterval: number;

  /**
   * Size of queue of L2 blocks to store.
   */
  l2QueueSize: number;
}

/**
 * Gets the config values for p2p client from environment variables.
 * @returns The config values for p2p client.
 */
export function getConfigEnvVars(): P2PConfig {
  const { P2P_CHECK_INTERVAL, P2P_L2_BLOCK_QUEUE_SIZE } = process.env;
  const envVars: P2PConfig = {
    checkInterval: P2P_CHECK_INTERVAL ? +P2P_CHECK_INTERVAL : 100,
    l2QueueSize: P2P_L2_BLOCK_QUEUE_SIZE ? +P2P_L2_BLOCK_QUEUE_SIZE : 1000,
  };
  return envVars;
}
