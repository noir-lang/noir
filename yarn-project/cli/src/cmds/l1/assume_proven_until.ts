import { createPXEClient, makeFetch } from '@aztec/aztec.js';
import { createEthereumChain, createL1Clients } from '@aztec/ethereum';
import { type LogFn } from '@aztec/foundation/log';

import { setAssumeProvenUntil } from '../../utils/aztec.js';

export async function assumeProvenUntil(
  blockNumberOrLatest: number | undefined,
  l1RpcUrl: string,
  rpcUrl: string,
  chainId: number,
  privateKey: string | undefined,
  mnemonic: string,
  log: LogFn,
) {
  const chain = createEthereumChain(l1RpcUrl, chainId);
  const { walletClient } = createL1Clients(chain.rpcUrl, privateKey ?? mnemonic, chain.chainInfo);

  const pxe = createPXEClient(rpcUrl, makeFetch([], true));
  const rollupAddress = await pxe.getNodeInfo().then(i => i.l1ContractAddresses.rollupAddress);
  const blockNumber = blockNumberOrLatest ?? (await pxe.getBlockNumber());

  await setAssumeProvenUntil(blockNumber + 1, rollupAddress, walletClient);
  log(`Assumed proven until block ${blockNumber}`);
}
