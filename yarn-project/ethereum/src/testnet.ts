import { Chain } from 'viem';

import { EthereumChain } from './ethereum_chain.js';

export const createTestnetChain = (apiKey: string) => {
  const chain: Chain = {
    id: 677868,
    name: 'testnet',
    network: 'aztec',
    nativeCurrency: {
      name: 'Ether',
      symbol: 'ETH',
      decimals: 18,
    },
    rpcUrls: {
      default: {
        http: [`https://aztec-connect-testnet-eth-host.aztec.network:8545/${apiKey}`],
      },
      public: {
        http: [`https://aztec-connect-testnet-eth-host.aztec.network:8545/${apiKey}`],
      },
    },
  };
  return {
    chainInfo: chain,
    rpcUrl: chain.rpcUrls.default.http[0],
  } as EthereumChain;
};
