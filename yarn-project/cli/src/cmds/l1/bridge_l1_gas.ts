import { type AztecAddress } from '@aztec/circuits.js';
import { createEthereumChain, createL1Clients } from '@aztec/ethereum';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../../client.js';
import { GasPortalManagerFactory } from '../../gas_portal.js';

export async function bridgeL1Gas(
  amount: bigint,
  recipient: AztecAddress,
  rpcUrl: string,
  l1RpcUrl: string,
  chainId: number,
  mnemonic: string,
  log: LogFn,
  debugLogger: DebugLogger,
) {
  // Prepare L1 client
  const chain = createEthereumChain(l1RpcUrl, chainId);
  const { publicClient, walletClient } = createL1Clients(chain.rpcUrl, mnemonic, chain.chainInfo);

  // Prepare L2 client
  const client = await createCompatibleClient(rpcUrl, debugLogger);

  // Setup portal manager
  const manager = await GasPortalManagerFactory.create({
    pxeService: client,
    publicClient: publicClient,
    walletClient: walletClient,
    logger: debugLogger,
  });

  const { secret } = await manager.prepareTokensOnL1(amount, amount, recipient);

  log(`Minted ${amount} gas tokens on L1 and pushed to L2 portal`);
  log(`claimAmount=${amount},claimSecret=${secret}\n`);
  log(`Note: You need to wait for two L2 blocks before pulling them from the L2 side`);
}
