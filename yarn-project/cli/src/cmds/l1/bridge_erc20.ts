import { type AztecAddress, type EthAddress, type Fr } from '@aztec/circuits.js';
import { createEthereumChain, createL1Clients } from '@aztec/ethereum';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { prettyPrintJSON } from '../../utils/commands.js';
import { L1PortalManager } from '../../utils/portal_manager.js';

export async function bridgeERC20(
  amount: bigint,
  recipient: AztecAddress,
  l1RpcUrl: string,
  chainId: number,
  privateKey: string | undefined,
  mnemonic: string,
  tokenAddress: EthAddress,
  portalAddress: EthAddress,
  privateTransfer: boolean,
  mint: boolean,
  json: boolean,
  log: LogFn,
  debugLogger: DebugLogger,
) {
  // Prepare L1 client
  const chain = createEthereumChain(l1RpcUrl, chainId);
  const { publicClient, walletClient } = createL1Clients(chain.rpcUrl, privateKey ?? mnemonic, chain.chainInfo);

  // Setup portal manager
  const manager = new L1PortalManager(portalAddress, tokenAddress, publicClient, walletClient, debugLogger);
  let claimSecret: Fr;
  let messageHash: `0x${string}`;
  if (privateTransfer) {
    ({ claimSecret, messageHash } = await manager.bridgeTokensPrivate(recipient, amount, mint));
  } else {
    ({ claimSecret, messageHash } = await manager.bridgeTokensPublic(recipient, amount, mint));
  }

  if (json) {
    log(
      prettyPrintJSON({
        claimAmount: amount,
        claimSecret: claimSecret,
      }),
    );
  } else {
    if (mint) {
      log(`Minted ${amount} tokens on L1 and pushed to L2 portal`);
    } else {
      log(`Bridged ${amount} tokens to L2 portal`);
    }
    log(`claimAmount=${amount},claimSecret=${claimSecret}\n,messageHash=${messageHash}`);
    log(`Note: You need to wait for two L2 blocks before pulling them from the L2 side`);
  }
}
