import { AztecAddress, EthAddress, Fr, TxStatus, Wallet } from '@aztec/aztec.js';
import { PortalERC20Abi, PortalERC20Bytecode, TokenPortalAbi, TokenPortalBytecode } from '@aztec/l1-artifacts';
import { TokenBridgeContract, TokenContract } from '@aztec/noir-contracts/types';

import type { Abi, Narrow } from 'abitype';
import { Account, Chain, Hex, HttpTransport, PublicClient, WalletClient, getContract } from 'viem';

/**
 * Deploy L1 token and portal, initialize portal, deploy a non native l2 token contract, its L2 bridge contract and attach is to the portal.
 * @param wallet - the wallet instance
 * @param walletClient - A viem WalletClient.
 * @param publicClient - A viem PublicClient.
 * @param rollupRegistryAddress - address of rollup registry to pass to initialize the token portal
 * @param owner - owner of the L2 contract
 * @param underlyingERC20Address - address of the underlying ERC20 contract to use (if none supplied, it deploys one)
 * @returns l2 contract instance, bridge contract instance, token portal instance, token portal address and the underlying ERC20 instance
 */
export async function deployAndInitializeTokenAndBridgeContracts(
  wallet: Wallet,
  walletClient: WalletClient<HttpTransport, Chain, Account>,
  publicClient: PublicClient<HttpTransport, Chain>,
  rollupRegistryAddress: EthAddress,
  owner: AztecAddress,
  underlyingERC20Address?: EthAddress,
): Promise<{
  /**
   * The L2 token contract instance.
   */
  token: TokenContract;
  /**
   * The L2 bridge contract instance.
   */
  bridge: TokenBridgeContract;
  /**
   * The token portal contract address.
   */
  tokenPortalAddress: EthAddress;
  /**
   * The token portal contract instance
   */
  tokenPortal: any;
  /**
   * The underlying ERC20 contract instance.
   */
  underlyingERC20: any;
}> {
  if (!underlyingERC20Address) {
    underlyingERC20Address = await deployL1Contract(walletClient, publicClient, PortalERC20Abi, PortalERC20Bytecode);
  }
  const underlyingERC20 = getContract({
    address: underlyingERC20Address.toString(),
    abi: PortalERC20Abi,
    walletClient,
    publicClient,
  });

  // deploy the token portal
  const tokenPortalAddress = await deployL1Contract(walletClient, publicClient, TokenPortalAbi, TokenPortalBytecode);
  const tokenPortal = getContract({
    address: tokenPortalAddress.toString(),
    abi: TokenPortalAbi,
    walletClient,
    publicClient,
  });

  // deploy l2 token
  const deployTx = TokenContract.deploy(wallet, owner).send();

  // now wait for the deploy txs to be mined. This way we send all tx in the same rollup.
  const deployReceipt = await deployTx.wait();
  if (deployReceipt.status !== TxStatus.MINED) throw new Error(`Deploy token tx status is ${deployReceipt.status}`);
  const token = await TokenContract.at(deployReceipt.contractAddress!, wallet);

  // deploy l2 token bridge and attach to the portal
  const bridgeTx = TokenBridgeContract.deploy(wallet, token.address).send({
    portalContract: tokenPortalAddress,
    contractAddressSalt: Fr.random(),
  });

  const bridgeReceipt = await bridgeTx.wait();
  const bridge = await TokenBridgeContract.at(bridgeReceipt.contractAddress!, wallet);
  await bridge.attach(tokenPortalAddress);
  const bridgeAddress = bridge.address.toString() as `0x${string}`;

  // now we wait for the txs to be mined. This way we send all tx in the same rollup.
  if ((await token.methods.admin().view()) !== owner.toBigInt()) throw new Error(`Token admin is not ${owner}`);

  if ((await bridge.methods.token().view()) !== token.address.toBigInt())
    throw new Error(`Bridge token is not ${token.address}`);

  // make the bridge a minter on the token:
  const makeMinterTx = token.methods.set_minter(bridge.address, true).send();
  const makeMinterReceipt = await makeMinterTx.wait();
  if (makeMinterReceipt.status !== TxStatus.MINED)
    throw new Error(`Make bridge a minter tx status is ${makeMinterReceipt.status}`);
  if ((await token.methods.is_minter(bridge.address).view()) === 1n) throw new Error(`Bridge is not a minter`);

  // initialize portal
  await tokenPortal.write.initialize(
    [rollupRegistryAddress.toString(), underlyingERC20Address.toString(), bridgeAddress],
    {} as any,
  );

  return { token, bridge, tokenPortalAddress, tokenPortal, underlyingERC20 };
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
  } as any);

  const receipt = await publicClient.waitForTransactionReceipt({ hash });
  const contractAddress = receipt.contractAddress;
  if (!contractAddress) {
    throw new Error(`No contract address found in receipt: ${JSON.stringify(receipt)}`);
  }

  return EthAddress.fromString(receipt.contractAddress!);
}
