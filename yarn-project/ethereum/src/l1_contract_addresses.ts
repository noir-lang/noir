import { EthAddress } from '@aztec/foundation/eth-address';

/**
 * Provides the directory of current L1 contract addresses
 */
export interface L1ContractAddresses {
  /**
   * Availability Oracle Address.
   */
  availabilityOracleAddress: EthAddress;
  /**
   * Rollup Address.
   */
  rollupAddress: EthAddress;
  /**
   * Registry Address.
   */
  registryAddress: EthAddress;
  /**
   * Inbox Address.
   */
  inboxAddress: EthAddress;
  /**
   * Outbox Address.
   */
  outboxAddress: EthAddress;
  /**
   * Data Emitter Address.
   */
  contractDeploymentEmitterAddress: EthAddress;
}
