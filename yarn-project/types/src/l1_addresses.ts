import { EthAddress } from '@aztec/foundation/eth-address';

/**
 * Rollup contract addresses.
 */
export interface L1Addresses {
  /**
   * Rollup contract address.
   */
  rollupContract: EthAddress;

  /**
   * Inbox contract address.
   */
  inboxContract: EthAddress;

  /**
   * Registry contract address.
   */
  registryContract: EthAddress;

  /**
   * ContractDeploymentEmitter contract address.
   */
  contractDeploymentEmitterContract: EthAddress;
}
