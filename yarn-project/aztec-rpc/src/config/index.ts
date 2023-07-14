/**
 * Configuration settings for the RPC Server.
 */
export interface RpcServerConfig {
  /**
   * The interval to wait between polling for new blocks.
   */
  l2BlockPollingIntervalMS: number;
}

/**
 * Creates an instance of SequencerClientConfig out of environment variables using sensible defaults for integration testing if not set.
 */
export function getConfigEnvVars(): RpcServerConfig {
  const { RPC_SERVER_BLOCK_POLLING_INTERVAL_MS } = process.env;

  return {
    l2BlockPollingIntervalMS: RPC_SERVER_BLOCK_POLLING_INTERVAL_MS ? +RPC_SERVER_BLOCK_POLLING_INTERVAL_MS : 1000,
  };
}
