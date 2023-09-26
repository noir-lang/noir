import { foundry } from 'viem/chains';

import { EthereumChain } from './ethereum_chain.js';
import { createTestnetChain } from './testnet.js';

export * from './testnet.js';
export * from './deploy_l1_contracts.js';
export * from './l1_contract_addresses.js';

/**
 * Helper function to create an instance of Aztec Chain from an rpc url and api key.
 * @param rpcUrl - The rpc url of the chain or a chain identifier (e.g. 'testnet')
 * @param apiKey - An optional API key for the chain client.
 */
export function createEthereumChain(rpcUrl: string, apiKey?: string) {
  if (rpcUrl === 'testnet') {
    if (apiKey === undefined || apiKey === '') {
      throw new Error('API Key must be provided for aztec testnet');
    }
    return createTestnetChain(apiKey!);
  }
  return {
    chainInfo: foundry,
    rpcUrl,
  } as EthereumChain;
}
