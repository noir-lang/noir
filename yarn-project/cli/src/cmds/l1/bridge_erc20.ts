import { type AztecAddress, type EthAddress } from '@aztec/circuits.js';
import { createEthereumChain, createL1Clients } from '@aztec/ethereum';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { ERC20PortalManager } from '../../portal_manager.js';

export async function bridgeERC20(
  amount: bigint,
  recipient: AztecAddress,
  l1RpcUrl: string,
  chainId: number,
  privateKey: string | undefined,
  mnemonic: string,
  tokenAddress: EthAddress,
  portalAddress: EthAddress,
  mint: boolean,
  log: LogFn,
  debugLogger: DebugLogger,
) {
  // Prepare L1 client
  const chain = createEthereumChain(l1RpcUrl, chainId);
  const { publicClient, walletClient } = createL1Clients(chain.rpcUrl, privateKey ?? mnemonic, chain.chainInfo);

  // Setup portal manager
  const portal = await ERC20PortalManager.create(tokenAddress, portalAddress, publicClient, walletClient, debugLogger);
  const { secret } = await portal.prepareTokensOnL1(amount, amount, recipient, mint);

  if (mint) {
    log(`Minted ${amount} tokens on L1 and pushed to L2 portal`);
  } else {
    log(`Bridged ${amount} tokens to L2 portal`);
  }
  log(`claimAmount=${amount},claimSecret=${secret}\n`);
  log(`Note: You need to wait for two L2 blocks before pulling them from the L2 side`);
}
