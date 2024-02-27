// docs:start:cross_chain_test_harness
import {
  AztecAddress,
  DebugLogger,
  EthAddress,
  Fr,
  PXE,
  TxStatus,
  Wallet,
  computeMessageSecretHash,
  deployL1Contract,
  sleep,
} from '@aztec/aztec.js';
import { GasPortalAbi, GasPortalBytecode, OutboxAbi, PortalERC20Abi, PortalERC20Bytecode } from '@aztec/l1-artifacts';
import { GasTokenContract } from '@aztec/noir-contracts.js';
import { getCanonicalGasToken } from '@aztec/protocol-contracts/gas-token';

import { Account, Chain, HttpTransport, PublicClient, WalletClient, getContract } from 'viem';

// docs:start:deployAndInitializeTokenAndBridgeContracts
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
  gasL2: GasTokenContract;
  /**
   * The token portal contract address.
   */
  gasPortalAddress: EthAddress;
  /**
   * The token portal contract instance
   */
  gasPortal: any;
  /**
   * The underlying ERC20 contract instance.
   */
  gasL1: any;
}> {
  if (!underlyingERC20Address) {
    underlyingERC20Address = await deployL1Contract(walletClient, publicClient, PortalERC20Abi, PortalERC20Bytecode);
  }
  const gasL1 = getContract({
    address: underlyingERC20Address.toString(),
    abi: PortalERC20Abi,
    client: walletClient,
  });

  // deploy the gas portal
  const gasPortalAddress = await deployL1Contract(walletClient, publicClient, GasPortalAbi, GasPortalBytecode);
  const gasPortal = getContract({
    address: gasPortalAddress.toString(),
    abi: GasPortalAbi,
    client: walletClient,
  });

  // deploy l2 token
  const gasL2 = await GasTokenContract.deploy(wallet)
    .send({
      portalContract: gasPortalAddress,
      contractAddressSalt: getCanonicalGasToken().instance.salt,
    })
    .deployed();

  // initialize portal
  await gasPortal.write.initialize(
    [rollupRegistryAddress.toString(), underlyingERC20Address.toString(), gasL2.address.toString()],
    {} as any,
  );

  return { gasL2, gasPortalAddress, gasPortal, gasL1 };
}
// docs:end:deployAndInitializeTokenAndBridgeContracts

/**
 * A Class for testing cross chain interactions, contains common interactions
 * shared between cross chain tests.
 */
export class GasBridgingTestHarness {
  static async new(
    pxeService: PXE,
    publicClient: PublicClient<HttpTransport, Chain>,
    walletClient: any,
    wallet: Wallet,
    logger: DebugLogger,
    underlyingERC20Address?: EthAddress,
  ): Promise<GasBridgingTestHarness> {
    const ethAccount = EthAddress.fromString((await walletClient.getAddresses())[0]);
    const owner = wallet.getCompleteAddress();
    const l1ContractAddresses = (await pxeService.getNodeInfo()).l1ContractAddresses;

    const outbox = getContract({
      address: l1ContractAddresses.outboxAddress.toString(),
      abi: OutboxAbi,
      client: walletClient,
    });

    // Deploy and initialize all required contracts
    logger('Deploying and initializing token, portal and its bridge...');
    const { gasPortalAddress, gasL1, gasL2, gasPortal } = await deployAndInitializeTokenAndBridgeContracts(
      wallet,
      walletClient,
      publicClient,
      l1ContractAddresses.registryAddress,
      owner.address,
      underlyingERC20Address,
    );
    logger('Deployed and initialized token, portal and its bridge.');

    return new GasBridgingTestHarness(
      pxeService,
      logger,
      gasL2,
      ethAccount,
      gasPortalAddress,
      gasPortal,
      gasL1,
      outbox,
      publicClient,
      walletClient,
    );
  }

  constructor(
    /** Private eXecution Environment (PXE). */
    public pxeService: PXE,
    /** Logger. */
    public logger: DebugLogger,

    /** L2 Token/Bridge contract. */
    public l2Token: GasTokenContract,

    /** Eth account to interact with. */
    public ethAccount: EthAddress,

    /** Portal address. */
    public tokenPortalAddress: EthAddress,
    /** Token portal instance. */
    public tokenPortal: any,
    /** Underlying token for portal tests. */
    public underlyingERC20: any,
    /** Message Bridge Outbox. */
    public outbox: any,
    /** Viem Public client instance. */
    public publicClient: PublicClient<HttpTransport, Chain>,
    /** Viem Wallet Client instance. */
    public walletClient: any,
  ) {}

  generateClaimSecret(): [Fr, Fr] {
    this.logger("Generating a claim secret using pedersen's hash function");
    const secret = Fr.random();
    const secretHash = computeMessageSecretHash(secret);
    this.logger('Generated claim secret: ' + secretHash.toString());
    return [secret, secretHash];
  }

  async mintTokensOnL1(amount: bigint) {
    this.logger('Minting tokens on L1');
    await this.underlyingERC20.write.mint([this.ethAccount.toString(), amount], {} as any);
    expect(await this.underlyingERC20.read.balanceOf([this.ethAccount.toString()])).toBe(amount);
  }

  async getL1BalanceOf(address: EthAddress) {
    return await this.underlyingERC20.read.balanceOf([address.toString()]);
  }

  async sendTokensToPortalPublic(bridgeAmount: bigint, l2Address: AztecAddress, secretHash: Fr) {
    await this.underlyingERC20.write.approve([this.tokenPortalAddress.toString(), bridgeAmount], {} as any);

    // Deposit tokens to the TokenPortal
    const deadline = 2 ** 32 - 1; // max uint32

    this.logger('Sending messages to L1 portal to be consumed publicly');
    const args = [
      l2Address.toString(),
      bridgeAmount,
      this.ethAccount.toString(),
      deadline,
      secretHash.toString(),
    ] as const;
    const { result: messageKeyHex } = await this.tokenPortal.simulate.depositToAztecPublic(args, {
      account: this.ethAccount.toString(),
    } as any);
    await this.tokenPortal.write.depositToAztecPublic(args, {} as any);

    return Fr.fromString(messageKeyHex);
  }

  async sendTokensToPortalPrivate(
    secretHashForRedeemingMintedNotes: Fr,
    bridgeAmount: bigint,
    secretHashForL2MessageConsumption: Fr,
  ) {
    await this.underlyingERC20.write.approve([this.tokenPortalAddress.toString(), bridgeAmount], {} as any);

    // Deposit tokens to the TokenPortal
    const deadline = 2 ** 32 - 1; // max uint32

    this.logger('Sending messages to L1 portal to be consumed privately');
    const args = [
      secretHashForRedeemingMintedNotes.toString(),
      bridgeAmount,
      this.ethAccount.toString(),
      deadline,
      secretHashForL2MessageConsumption.toString(),
    ] as const;
    const { result: messageKeyHex } = await this.tokenPortal.simulate.depositToAztecPrivate(args, {
      account: this.ethAccount.toString(),
    } as any);
    await this.tokenPortal.write.depositToAztecPrivate(args, {} as any);

    return Fr.fromString(messageKeyHex);
  }

  async consumeMessageOnAztecAndMintPublicly(bridgeAmount: bigint, owner: AztecAddress, messageKey: Fr, secret: Fr) {
    this.logger('Consuming messages on L2 Publicly');
    // Call the mint tokens function on the Aztec.nr contract
    const tx = this.l2Token.methods.claim_public(owner, bridgeAmount, this.ethAccount, messageKey, secret).send();
    const receipt = await tx.wait();
    expect(receipt.status).toBe(TxStatus.MINED);
  }

  async getL2PublicBalanceOf(owner: AztecAddress) {
    return await this.l2Token.methods.balance_of_public(owner).view();
  }

  async expectPublicBalanceOnL2(owner: AztecAddress, expectedBalance: bigint) {
    const balance = await this.getL2PublicBalanceOf(owner);
    expect(balance).toBe(expectedBalance);
  }

  async bridgeFromL1ToL2(l1TokenBalance: bigint, bridgeAmount: bigint, owner: AztecAddress) {
    const [secret, secretHash] = this.generateClaimSecret();

    // 1. Mint tokens on L1
    await this.mintTokensOnL1(l1TokenBalance);

    // 2. Deposit tokens to the TokenPortal
    const messageKey = await this.sendTokensToPortalPublic(bridgeAmount, owner, secretHash);
    expect(await this.getL1BalanceOf(this.ethAccount)).toBe(l1TokenBalance - bridgeAmount);

    // Wait for the archiver to process the message
    await sleep(2500);

    // Perform an unrelated transaction on L2 to progress the rollup. Here we mint public tokens.
    await this.l2Token.methods.check_balance(0).send().wait();

    // 3. Consume L1-> L2 message and mint public tokens on L2
    await this.consumeMessageOnAztecAndMintPublicly(bridgeAmount, owner, messageKey, secret);
    await this.expectPublicBalanceOnL2(owner, bridgeAmount);
  }
}
// docs:end:cross_chain_test_harness
