import { type EthAddress } from '@aztec/circuits.js';
import { createEthereumChain } from '@aztec/ethereum';
import { type LogFn } from '@aztec/foundation/log';
import { PortalERC20Abi } from '@aztec/l1-artifacts';

import { createPublicClient, getContract, http } from 'viem';

export async function getL1Balance(who: EthAddress, token: EthAddress, l1RpcUrl: string, chainId: number, log: LogFn) {
  const chain = createEthereumChain(l1RpcUrl, chainId);
  const publicClient = createPublicClient({ chain: chain.chainInfo, transport: http(chain.rpcUrl) });

  const gasL1 = getContract({
    address: token.toString(),
    abi: PortalERC20Abi,
    client: publicClient,
  });

  const balance = await gasL1.read.balanceOf([who.toString()]);

  log(`L1 gas token balance of ${who.toString()} is ${balance.toString()}`);
}
