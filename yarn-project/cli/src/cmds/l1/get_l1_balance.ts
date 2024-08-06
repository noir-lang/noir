import { type EthAddress } from '@aztec/circuits.js';
import { createEthereumChain } from '@aztec/ethereum';
import { type LogFn } from '@aztec/foundation/log';
import { PortalERC20Abi } from '@aztec/l1-artifacts';

import { createPublicClient, getContract, http } from 'viem';

import { prettyPrintJSON } from '../../utils/commands.js';

export async function getL1Balance(
  who: EthAddress,
  token: EthAddress | undefined,
  l1RpcUrl: string,
  chainId: number,
  json: boolean,
  log: LogFn,
) {
  const chain = createEthereumChain(l1RpcUrl, chainId);
  const publicClient = createPublicClient({ chain: chain.chainInfo, transport: http(chain.rpcUrl) });

  let balance = 0n;
  if (token) {
    const gasL1 = getContract({
      address: token.toString(),
      abi: PortalERC20Abi,
      client: publicClient,
    });

    balance = await gasL1.read.balanceOf([who.toString()]);
  } else {
    balance = await publicClient.getBalance({
      address: who.toString(),
    });
  }

  if (json) {
    log(prettyPrintJSON({ balance }));
  } else {
    log(`L1 balance of ${who.toString()} is ${balance.toString()}`);
  }
}
