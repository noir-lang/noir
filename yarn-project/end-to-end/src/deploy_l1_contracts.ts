import { EthAddress } from '@aztec/foundation/eth-address';
import { DebugLogger } from '@aztec/foundation/log';
import {
  RollupAbi,
  RollupBytecode,
  UnverifiedDataEmitterAbi,
  UnverifiedDataEmitterBytecode,
} from '@aztec/l1-artifacts';
import type { Abi, Narrow } from 'abitype';
import {
  Account,
  Chain,
  Hex,
  HttpTransport,
  PublicClient,
  WalletClient,
  createPublicClient,
  createWalletClient,
  http,
} from 'viem';
import { HDAccount } from 'viem/accounts';
import { foundry } from 'viem/chains';

export const deployL1Contracts = async (rpcUrl: string, account: HDAccount, logger: DebugLogger) => {
  logger('Deploying contracts... (rpcURL: %s)', rpcUrl);

  const walletClient = createWalletClient({
    account,
    chain: foundry,
    transport: http(rpcUrl),
  });
  const publicClient = createPublicClient({
    chain: foundry,
    transport: http(rpcUrl),
  });

  const rollupAddress = await deployL1Contract(walletClient, publicClient, RollupAbi, RollupBytecode);
  logger(`Deployed rollup contract at ${rollupAddress}`);

  const unverifiedDataEmitterAddress = await deployL1Contract(
    walletClient,
    publicClient,
    UnverifiedDataEmitterAbi,
    UnverifiedDataEmitterBytecode,
  );
  logger(`Deployed unverified data emitter at ${unverifiedDataEmitterAddress}`);

  return {
    rollupAddress,
    unverifiedDataEmitterAddress,
  };
};

async function deployL1Contract(
  walletClient: WalletClient<HttpTransport, Chain, Account>,
  publicClient: PublicClient<HttpTransport, Chain>,
  abi: Narrow<Abi | readonly unknown[]>,
  bytecode: Hex,
) {
  const hash = await walletClient.deployContract({
    abi,
    bytecode,
  });

  const receipt = await publicClient.waitForTransactionReceipt({ hash });
  const contractAddress = receipt.contractAddress;
  if (!contractAddress) {
    throw new Error(`No contract address found in receipt: ${JSON.stringify(receipt)}`);
  }

  return EthAddress.fromString(receipt.contractAddress!);
}
