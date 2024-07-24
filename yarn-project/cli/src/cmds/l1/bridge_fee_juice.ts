import { type AztecAddress } from '@aztec/circuits.js';
import { createEthereumChain, createL1Clients } from '@aztec/ethereum';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../../client.js';
import { FeeJuicePortalManager } from '../../portal_manager.js';

export async function bridgeL1Gas(
  amount: bigint,
  recipient: AztecAddress,
  rpcUrl: string,
  l1RpcUrl: string,
  chainId: number,
  mnemonic: string,
  mint: boolean,
  log: LogFn,
  debugLogger: DebugLogger,
) {
  // Prepare L1 client
  const chain = createEthereumChain(l1RpcUrl, chainId);
  const { publicClient, walletClient } = createL1Clients(chain.rpcUrl, mnemonic, chain.chainInfo);

  // Prepare L2 client
  const client = await createCompatibleClient(rpcUrl, debugLogger);

  // Setup portal manager
  const portal = await FeeJuicePortalManager.create(client, publicClient, walletClient, debugLogger);
  const { secret } = await portal.prepareTokensOnL1(amount, amount, recipient, mint);

  log(`Minted ${amount} gas tokens on L1 and pushed to L2 portal`);
  log(`claimAmount=${amount},claimSecret=${secret}\n`);
  log(`Note: You need to wait for two L2 blocks before pulling them from the L2 side`);
}
