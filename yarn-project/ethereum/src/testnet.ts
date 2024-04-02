import { type Chain } from 'viem';

import { type EthereumChain } from './ethereum_chain.js';

const { DEPLOY_TAG = 'aztec-dev', CHAIN_ID = 31337 } = process.env;

export const createTestnetChain = (apiKey: string) => {
  const chain: Chain = {
    id: +CHAIN_ID,
    name: 'testnet',
    testnet: true,
    nativeCurrency: {
      name: 'Ether',
      symbol: 'ETH',
      decimals: 18,
    },
    rpcUrls: {
      default: {
        http: [`https://${DEPLOY_TAG}-mainnet-fork.aztec.network:8545/${apiKey}`],
      },
      public: {
        http: [`https://${DEPLOY_TAG}-mainnet-fork.aztec.network:8545/${apiKey}`],
      },
    },
  };
  return {
    chainInfo: chain,
    rpcUrl: chain.rpcUrls.default.http[0],
  } as EthereumChain;
};
