import { createEthereumChain } from '@aztec/ethereum';

import { type AztecNodeConfig, AztecNodeService } from '../index.js';

describe('aztec node service', () => {
  it('fails to create Aztec Node if given incorrect chain id', async () => {
    const config: Partial<AztecNodeConfig> = {
      rpcUrl: 'testnet',
      apiKey: '12345',
      chainId: 12345, // not the testnet chain id
    };
    const ethereumChain = createEthereumChain(config.rpcUrl!, config.apiKey);
    await expect(() => AztecNodeService.createAndSync(config as AztecNodeConfig)).rejects.toThrow(
      `RPC URL configured for chain id ${ethereumChain.chainInfo.id} but expected id ${config.chainId}`,
    );
  });
});
