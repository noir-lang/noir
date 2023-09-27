import { AztecAddress, EthAddress, Wallet } from '@aztec/aztec.js';
import { PortalERC20Abi, PortalERC20Bytecode, TokenPortalAbi, TokenPortalBytecode } from '@aztec/l1-artifacts';
import { NonNativeTokenContract } from '@aztec/noir-contracts/types';

import type { Abi, Narrow } from 'abitype';
import { Account, Chain, Hex, HttpTransport, PublicClient, WalletClient, getContract } from 'viem';

/**
 * Deploy L1 token and portal, initialize portal, deploy a non native l2 token contract and attach is to the portal.
 * @param wallet - A wallet instance.
 * @param walletClient - A viem WalletClient.
 * @param publicClient - A viem PublicClient.
 * @param rollupRegistryAddress - address of rollup registry to pass to initialize the token portal
 * @param initialBalance - initial balance of the owner of the L2 contract
 * @param owner - owner of the L2 contract
 * @param underlyingERC20Address - address of the underlying ERC20 contract to use (if none supplied, it deploys one)
 * @returns l2 contract instance, token portal instance, token portal address and the underlying ERC20 instance
 */
export async function deployAndInitializeNonNativeL2TokenContracts(
  wallet: Wallet,
  walletClient: WalletClient<HttpTransport, Chain, Account>,
  publicClient: PublicClient<HttpTransport, Chain>,
  rollupRegistryAddress: EthAddress,
  initialBalance = 0n,
  owner = AztecAddress.ZERO,
  underlyingERC20Address?: EthAddress,
) {
  // deploy underlying contract if no address supplied
  if (!underlyingERC20Address) {
    underlyingERC20Address = await deployL1Contract(walletClient, publicClient, PortalERC20Abi, PortalERC20Bytecode);
  }
  const underlyingERC20: any = getContract({
    address: underlyingERC20Address.toString(),
    abi: PortalERC20Abi,
    walletClient,
    publicClient,
  });

  // deploy the token portal
  const tokenPortalAddress = await deployL1Contract(walletClient, publicClient, TokenPortalAbi, TokenPortalBytecode);
  const tokenPortal: any = getContract({
    address: tokenPortalAddress.toString(),
    abi: TokenPortalAbi,
    walletClient,
    publicClient,
  });

  // deploy l2 contract and attach to portal
  const tx = NonNativeTokenContract.deploy(wallet, initialBalance, owner).send({
    portalContract: tokenPortalAddress,
  });
  await tx.isMined({ interval: 0.1 });
  const receipt = await tx.getReceipt();
  const l2Contract = await NonNativeTokenContract.at(receipt.contractAddress!, wallet);
  await l2Contract.attach(tokenPortalAddress);
  const l2TokenAddress = l2Contract.address.toString() as `0x${string}`;

  // initialize portal
  await tokenPortal.write.initialize(
    [rollupRegistryAddress.toString(), underlyingERC20Address.toString(), l2TokenAddress],
    {} as any,
  );
  return { l2Contract, tokenPortalAddress, tokenPortal, underlyingERC20 };
}

/**
 * Helper function to deploy ETH contracts.
 * @param walletClient - A viem WalletClient.
 * @param publicClient - A viem PublicClient.
 * @param abi - The ETH contract's ABI (as abitype's Abi).
 * @param bytecode  - The ETH contract's bytecode.
 * @param args - Constructor arguments for the contract.
 * @returns The ETH address the contract was deployed to.
 */
export async function deployL1Contract(
  walletClient: WalletClient<HttpTransport, Chain, Account>,
  publicClient: PublicClient<HttpTransport, Chain>,
  abi: Narrow<Abi | readonly unknown[]>,
  bytecode: Hex,
  args: readonly unknown[] = [],
): Promise<EthAddress> {
  const hash = await walletClient.deployContract({
    abi,
    bytecode,
    args,
  });

  const receipt = await publicClient.waitForTransactionReceipt({ hash });
  const contractAddress = receipt.contractAddress;
  if (!contractAddress) {
    throw new Error(`No contract address found in receipt: ${JSON.stringify(receipt)}`);
  }

  return EthAddress.fromString(receipt.contractAddress!);
}

/**
 * Sleep for a given number of milliseconds.
 * @param ms - the number of milliseconds to sleep for
 */
export function delay(ms: number): Promise<void> {
  return new Promise<void>(resolve => setTimeout(resolve, ms));
}
