import { EthAddress } from '@aztec/foundation';

/**
 * A protocol level, type 2, EIP-1559 transaction.
 * This is what's rlp encoded, hashed, and signed over.
 * It's distinct from ETH JSON RPC types, which are much less strict.
 */
export interface EthTransaction {
  /**
   * The unique identifier for the Ethereum network.
   */
  chainId: number;
  /**
   * The destination Ethereum address for the transaction.
   */
  to?: EthAddress;
  /**
   * The maximum amount of gas units allocated for the execution of the transaction.
   */
  gas: number;
  /**
   * The maximum fee per gas unit for the transaction, expressed in Gwei.
   */
  maxFeePerGas: bigint;
  /**
   * Maximum fee per gas unit to prioritize the transaction inclusion.
   */
  maxPriorityFeePerGas: bigint;
  /**
   * The amount of Ether to be transferred in the transaction.
   */
  value: bigint;
  /**
   * The input data for the transaction execution.
   */
  data?: Buffer;
  /**
   * A unique value representing the number of transactions sent from a specific address.
   */
  nonce: number;
}
