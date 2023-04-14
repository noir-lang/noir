import { EthAddress } from '@aztec/foundation';
import { L1Addresses } from '@aztec/l1-contracts';

/**
 * The archiver configuration.
 */
export interface ArchiverConfig extends L1Addresses {
  /**
   * The url of the Ethereum RPC node.
   */
  rpcUrl: string;

  /**
   * The polling interval in ms for retrieving new L2 blocks and unverified data.
   */
  archiverPollingInterval?: number;
}

export function getConfigEnvVars(): ArchiverConfig {
  const { ETHEREUM_HOST, ARCHIVER_POLLING_INTERVAL, ROLLUP_CONTRACT_ADDRESS, UNVERIFIED_DATA_EMITTER_ADDRESS } =
    process.env;
  return {
    rpcUrl: ETHEREUM_HOST || 'http://127.0.0.1:8545/',
    archiverPollingInterval: ARCHIVER_POLLING_INTERVAL ? +ARCHIVER_POLLING_INTERVAL : 1_000,
    rollupContract: ROLLUP_CONTRACT_ADDRESS ? EthAddress.fromString(ROLLUP_CONTRACT_ADDRESS) : EthAddress.ZERO,
    unverifiedDataEmitterContract: UNVERIFIED_DATA_EMITTER_ADDRESS
      ? EthAddress.fromString(UNVERIFIED_DATA_EMITTER_ADDRESS)
      : EthAddress.ZERO,
  };
}
