import { EthAddress } from '@aztec/foundation';

/**
 * A protocol level, type 2, EIP-1559 transaction.
 * This is what's rlp encoded, hashed, and signed over.
 * It's distinct from ETH JSON RPC types, which are much less strict.
 */
export interface EthTransaction {
  chainId: number;
  to?: EthAddress;
  gas: number;
  maxFeePerGas: bigint;
  maxPriorityFeePerGas: bigint;
  value: bigint;
  data?: Buffer;
  nonce: number;
}
