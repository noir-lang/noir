// docs:start:cross_chain_test_harness
import {
  type AztecAddress,
  type AztecNode,
  type DebugLogger,
  EthAddress,
  ExtendedNote,
  type FieldsOf,
  Fr,
  Note,
  type PXE,
  type SiblingPath,
  type TxHash,
  type TxReceipt,
  type Wallet,
  computeMessageSecretHash,
  deployL1Contract,
  retryUntil,
} from '@aztec/aztec.js';
import { sha256ToField } from '@aztec/foundation/crypto';
import {
  InboxAbi,
  OutboxAbi,
  PortalERC20Abi,
  PortalERC20Bytecode,
  TokenPortalAbi,
  TokenPortalBytecode,
} from '@aztec/l1-artifacts';
import { TokenContract } from '@aztec/noir-contracts.js/Token';
import { TokenBridgeContract } from '@aztec/noir-contracts.js/TokenBridge';

import {
  type Account,
  type Chain,
  type GetContractReturnType,
  type HttpTransport,
  type PublicClient,
  type WalletClient,
  getContract,
  toFunctionSelector,
} from 'viem';

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
    client: walletClient,
  });

  // deploy the token portal
  const tokenPortalAddress = await deployL1Contract(walletClient, publicClient, TokenPortalAbi, TokenPortalBytecode);
  const tokenPortal = getContract({
    address: tokenPortalAddress.toString(),
    abi: TokenPortalAbi,
    client: walletClient,
  });

  // deploy l2 token
  const token = await TokenContract.deploy(wallet, owner, 'TokenName', 'TokenSymbol', 18).send().deployed();

  // deploy l2 token bridge and attach to the portal
  const bridge = await TokenBridgeContract.deploy(wallet, token.address)
    .send({ portalContract: tokenPortalAddress })
    .deployed();

  if ((await token.methods.admin().simulate()) !== owner.toBigInt()) {
    throw new Error(`Token admin is not ${owner}`);
  }

  if (!(await bridge.methods.token().simulate()).equals(token.address)) {
    throw new Error(`Bridge token is not ${token.address}`);
  }

  // make the bridge a minter on the token:
  await token.methods.set_minter(bridge.address, true).send().wait();
  if ((await token.methods.is_minter(bridge.address).simulate()) === 1n) {
    throw new Error(`Bridge is not a minter`);
  }

  // initialize portal
  await tokenPortal.write.initialize(
    [rollupRegistryAddress.toString(), underlyingERC20Address.toString(), bridge.address.toString()],
    {} as any,
  );

  return { token, bridge, tokenPortalAddress, tokenPortal, underlyingERC20 };
}
// docs:end:deployAndInitializeTokenAndBridgeContracts

/**
 * A Class for testing cross chain interactions, contains common interactions
 * shared between cross chain tests.
 */
export class CrossChainTestHarness {
  static async new(
    aztecNode: AztecNode,
    pxeService: PXE,
    publicClient: PublicClient<HttpTransport, Chain>,
    walletClient: WalletClient<HttpTransport, Chain, Account>,
    wallet: Wallet,
    logger: DebugLogger,
    underlyingERC20Address?: EthAddress,
  ): Promise<CrossChainTestHarness> {
    const ethAccount = EthAddress.fromString((await walletClient.getAddresses())[0]);
    const owner = wallet.getCompleteAddress();
    const l1ContractAddresses = (await pxeService.getNodeInfo()).l1ContractAddresses;

    const inbox = getContract({
      address: l1ContractAddresses.inboxAddress.toString(),
      abi: InboxAbi,
      client: walletClient,
    });

    const outbox = getContract({
      address: l1ContractAddresses.outboxAddress.toString(),
      abi: OutboxAbi,
      client: walletClient,
    });

    // Deploy and initialize all required contracts
    logger('Deploying and initializing token, portal and its bridge...');
    const { token, bridge, tokenPortalAddress, tokenPortal, underlyingERC20 } =
      await deployAndInitializeTokenAndBridgeContracts(
        wallet,
        walletClient,
        publicClient,
        l1ContractAddresses.registryAddress,
        owner.address,
        underlyingERC20Address,
      );
    logger('Deployed and initialized token, portal and its bridge.');

    return new CrossChainTestHarness(
      aztecNode,
      pxeService,
      logger,
      token,
      bridge,
      ethAccount,
      tokenPortalAddress,
      tokenPortal,
      underlyingERC20,
      inbox,
      outbox,
      publicClient,
      walletClient,
      owner.address,
    );
  }

  constructor(
    /** Aztec node instance. */
    public aztecNode: AztecNode,
    /** Private eXecution Environment (PXE). */
    public pxeService: PXE,
    /** Logger. */
    public logger: DebugLogger,

    /** L2 Token contract. */
    public l2Token: TokenContract,
    /** L2 Token bridge contract. */
    public l2Bridge: TokenBridgeContract,

    /** Eth account to interact with. */
    public ethAccount: EthAddress,

    /** Portal address. */
    public tokenPortalAddress: EthAddress,
    /** Token portal instance. */
    public tokenPortal: any,
    /** Underlying token for portal tests. */
    public underlyingERC20: any,
    /** Message Bridge Inbox. */
    public inbox: GetContractReturnType<typeof InboxAbi, WalletClient<HttpTransport, Chain, Account>>,
    /** Message Bridge Outbox. */
    public outbox: GetContractReturnType<typeof OutboxAbi, WalletClient<HttpTransport, Chain, Account>>,
    /** Viem Public client instance. */
    public publicClient: PublicClient<HttpTransport, Chain>,
    /** Viem Wallet Client instance. */
    public walletClient: any,

    /** Aztec address to use in tests. */
    public ownerAddress: AztecAddress,
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
    const txHash = await this.underlyingERC20.write.mint([this.ethAccount.toString(), amount], {} as any);
    await this.publicClient.waitForTransactionReceipt({ hash: txHash });
    expect(await this.underlyingERC20.read.balanceOf([this.ethAccount.toString()])).toBe(amount);
  }

  async getL1BalanceOf(address: EthAddress) {
    return await this.underlyingERC20.read.balanceOf([address.toString()]);
  }

  async sendTokensToPortalPublic(bridgeAmount: bigint, secretHash: Fr) {
    const txHash1 = await this.underlyingERC20.write.approve(
      [this.tokenPortalAddress.toString(), bridgeAmount],
      {} as any,
    );
    await this.publicClient.waitForTransactionReceipt({ hash: txHash1 });

    // Deposit tokens to the TokenPortal
    this.logger('Sending messages to L1 portal to be consumed publicly');
    const args = [this.ownerAddress.toString(), bridgeAmount, secretHash.toString()] as const;
    const { result: messageHash } = await this.tokenPortal.simulate.depositToAztecPublic(args, {
      account: this.ethAccount.toString(),
    } as any);
    const txHash2 = await this.tokenPortal.write.depositToAztecPublic(args, {} as any);
    await this.publicClient.waitForTransactionReceipt({ hash: txHash2 });

    return Fr.fromString(messageHash);
  }

  async sendTokensToPortalPrivate(
    secretHashForRedeemingMintedNotes: Fr,
    bridgeAmount: bigint,
    secretHashForL2MessageConsumption: Fr,
  ) {
    const txHash1 = await this.underlyingERC20.write.approve(
      [this.tokenPortalAddress.toString(), bridgeAmount],
      {} as any,
    );
    await this.publicClient.waitForTransactionReceipt({ hash: txHash1 });
    // Deposit tokens to the TokenPortal
    this.logger('Sending messages to L1 portal to be consumed privately');
    const args = [
      secretHashForRedeemingMintedNotes.toString(),
      bridgeAmount,
      secretHashForL2MessageConsumption.toString(),
    ] as const;
    const { result: messageHash } = await this.tokenPortal.simulate.depositToAztecPrivate(args, {
      account: this.ethAccount.toString(),
    } as any);
    const txHash2 = await this.tokenPortal.write.depositToAztecPrivate(args, {} as any);
    await this.publicClient.waitForTransactionReceipt({ hash: txHash2 });

    return Fr.fromString(messageHash);
  }

  async mintTokensPublicOnL2(amount: bigint) {
    this.logger('Minting tokens on L2 publicly');
    await this.l2Token.methods.mint_public(this.ownerAddress, amount).send().wait();
  }

  async mintTokensPrivateOnL2(amount: bigint, secretHash: Fr) {
    const receipt = await this.l2Token.methods.mint_private(amount, secretHash).send().wait();
    await this.addPendingShieldNoteToPXE(amount, secretHash, receipt.txHash);
  }

  async performL2Transfer(transferAmount: bigint, receiverAddress: AztecAddress) {
    // send a transfer tx to force through rollup with the message included
    await this.l2Token.methods.transfer_public(this.ownerAddress, receiverAddress, transferAmount, 0).send().wait();
  }

  async consumeMessageOnAztecAndMintPrivately(
    secretHashForRedeemingMintedNotes: Fr,
    bridgeAmount: bigint,
    secretForL2MessageConsumption: Fr,
  ) {
    this.logger('Consuming messages on L2 privately');
    // Call the mint tokens function on the Aztec.nr contract
    const consumptionReceipt = await this.l2Bridge.methods
      .claim_private(secretHashForRedeemingMintedNotes, bridgeAmount, secretForL2MessageConsumption)
      .send()
      .wait();

    await this.addPendingShieldNoteToPXE(bridgeAmount, secretHashForRedeemingMintedNotes, consumptionReceipt.txHash);
  }

  async consumeMessageOnAztecAndMintPublicly(bridgeAmount: bigint, secret: Fr) {
    this.logger('Consuming messages on L2 Publicly');
    // Call the mint tokens function on the Aztec.nr contract
    await this.l2Bridge.methods.claim_public(this.ownerAddress, bridgeAmount, secret).send().wait();
  }

  async withdrawPrivateFromAztecToL1(withdrawAmount: bigint, nonce: Fr = Fr.ZERO): Promise<FieldsOf<TxReceipt>> {
    const withdrawReceipt = await this.l2Bridge.methods
      .exit_to_l1_private(this.l2Token.address, this.ethAccount, withdrawAmount, EthAddress.ZERO, nonce)
      .send()
      .wait();

    return withdrawReceipt;
  }

  async withdrawPublicFromAztecToL1(withdrawAmount: bigint, nonce: Fr = Fr.ZERO): Promise<FieldsOf<TxReceipt>> {
    const withdrawReceipt = await this.l2Bridge.methods
      .exit_to_l1_public(this.ethAccount, withdrawAmount, EthAddress.ZERO, nonce)
      .send()
      .wait();

    return withdrawReceipt;
  }

  async getL2PrivateBalanceOf(owner: AztecAddress) {
    return await this.l2Token.methods.balance_of_private(owner).simulate({ from: owner });
  }

  async expectPrivateBalanceOnL2(owner: AztecAddress, expectedBalance: bigint) {
    const balance = await this.getL2PrivateBalanceOf(owner);
    this.logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  }

  async getL2PublicBalanceOf(owner: AztecAddress) {
    return await this.l2Token.methods.balance_of_public(owner).simulate();
  }

  async expectPublicBalanceOnL2(owner: AztecAddress, expectedBalance: bigint) {
    const balance = await this.getL2PublicBalanceOf(owner);
    expect(balance).toBe(expectedBalance);
  }

  getL2ToL1MessageLeaf(withdrawAmount: bigint, callerOnL1: EthAddress = EthAddress.ZERO): Fr {
    const content = sha256ToField([
      Buffer.from(toFunctionSelector('withdraw(address,uint256,address)').substring(2), 'hex'),
      this.ethAccount.toBuffer32(),
      new Fr(withdrawAmount).toBuffer(),
      callerOnL1.toBuffer32(),
    ]);
    const leaf = sha256ToField([
      this.l2Bridge.address.toBuffer(),
      new Fr(1).toBuffer(), // aztec version
      this.tokenPortalAddress.toBuffer32() ?? Buffer.alloc(32, 0),
      new Fr(this.publicClient.chain.id).toBuffer(), // chain id
      content.toBuffer(),
    ]);

    return leaf;
  }

  async withdrawFundsFromBridgeOnL1(
    withdrawAmount: bigint,
    blockNumber: number,
    messageIndex: bigint,
    siblingPath: SiblingPath<number>,
  ) {
    this.logger('Send L1 tx to consume message and withdraw funds');
    // Call function on L1 contract to consume the message
    const { request: withdrawRequest } = await this.tokenPortal.simulate.withdraw([
      this.ethAccount.toString(),
      withdrawAmount,
      false,
      BigInt(blockNumber),
      messageIndex,
      siblingPath.toBufferArray().map((buf: Buffer) => `0x${buf.toString('hex')}`) as readonly `0x${string}`[],
    ]);

    expect(
      await this.outbox.read.hasMessageBeenConsumedAtBlockAndIndex([BigInt(blockNumber), BigInt(messageIndex)], {}),
    ).toBe(false);

    await this.walletClient.writeContract(withdrawRequest);
    await expect(async () => {
      await this.walletClient.writeContract(withdrawRequest);
    }).rejects.toThrow();

    expect(
      await this.outbox.read.hasMessageBeenConsumedAtBlockAndIndex([BigInt(blockNumber), BigInt(messageIndex)], {}),
    ).toBe(true);
  }

  async shieldFundsOnL2(shieldAmount: bigint, secretHash: Fr) {
    this.logger('Shielding funds on L2');
    const shieldReceipt = await this.l2Token.methods
      .shield(this.ownerAddress, shieldAmount, secretHash, 0)
      .send()
      .wait();

    await this.addPendingShieldNoteToPXE(shieldAmount, secretHash, shieldReceipt.txHash);
  }

  async addPendingShieldNoteToPXE(shieldAmount: bigint, secretHash: Fr, txHash: TxHash) {
    this.logger('Adding note to PXE');
    const storageSlot = new Fr(5);
    const noteTypeId = new Fr(84114971101151129711410111011678111116101n); // TransparentNote
    const note = new Note([new Fr(shieldAmount), secretHash]);
    const extendedNote = new ExtendedNote(
      note,
      this.ownerAddress,
      this.l2Token.address,
      storageSlot,
      noteTypeId,
      txHash,
    );
    await this.pxeService.addNote(extendedNote);
  }

  async redeemShieldPrivatelyOnL2(shieldAmount: bigint, secret: Fr) {
    this.logger('Spending note in private call');
    await this.l2Token.methods.redeem_shield(this.ownerAddress, shieldAmount, secret).send().wait();
  }

  async unshieldTokensOnL2(unshieldAmount: bigint, nonce = Fr.ZERO) {
    this.logger('Unshielding tokens');
    await this.l2Token.methods.unshield(this.ownerAddress, this.ownerAddress, unshieldAmount, nonce).send().wait();
  }

  /**
   * Makes message available for consumption.
   * @dev Does that by performing 2 unrelated transactions on L2 to progress the rollup by 2 blocks and then waits for
   * message to be processed by archiver. We need to progress by 2 because there is a 1 block lag between when
   * the message is sent to Inbox and when the subtree containing the message is included in the block and then when
   * it's included it becomes available for consumption in the next block because the l1 to l2 message tree.
   */
  async makeMessageConsumable(msgHash: Fr) {
    const currentL2BlockNumber = await this.aztecNode.getBlockNumber();
    // We poll isL1ToL2MessageSynced endpoint until the message is available
    await retryUntil(
      async () => await this.aztecNode.isL1ToL2MessageSynced(msgHash, currentL2BlockNumber),
      'message sync',
      10,
    );

    await this.mintTokensPublicOnL2(0n);
    await this.mintTokensPublicOnL2(0n);
  }
}
// docs:end:cross_chain_test_harness
