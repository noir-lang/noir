import { foundry } from 'viem/chains';

import { type EthereumChain } from './ethereum_chain.js';

export * from './deploy_l1_contracts.js';
export * from './l1_contract_addresses.js';
export * from './constants.js';

/**
 * Helper function to create an instance of Aztec Chain from an rpc url and api key.
 * @param rpcUrl - The rpc url of the chain or a chain identifier (e.g. 'testnet')
 * @param apiKey - An optional API key for the chain client.
 */
export function createEthereumChain(rpcUrl: string, chainId?: number) {
  if (chainId) {
    return {
      chainInfo: {
        id: chainId,
        nativeCurrency: {
          decimals: 18,
          name: 'Ether',
          symbol: 'ETH',
        },
      },
      rpcUrl,
    } as EthereumChain;
  } else {
    return {
      chainInfo: foundry,
      rpcUrl,
    } as EthereumChain;
  }
}
