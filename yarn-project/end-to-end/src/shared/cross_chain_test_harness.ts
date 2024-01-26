// docs:start:cross_chain_test_harness
import {
  AztecAddress,
  DebugLogger,
  EthAddress,
  ExtendedNote,
  Fr,
  Note,
  PXE,
  TxHash,
  TxStatus,
  Wallet,
  computeMessageSecretHash,
  deployL1Contract,
  sha256,
} from '@aztec/aztec.js';
import {
  OutboxAbi,
  PortalERC20Abi,
  PortalERC20Bytecode,
  TokenPortalAbi,
  TokenPortalBytecode,
} from '@aztec/l1-artifacts';
import { TokenContract } from '@aztec/noir-contracts/Token';
import { TokenBridgeContract } from '@aztec/noir-contracts/TokenBridge';

import { Account, Chain, HttpTransport, PublicClient, WalletClient, getContract, getFunctionSelector } from 'viem';

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
  const deployTx = TokenContract.deploy(wallet, owner, 'TokenName', 'TokenSymbol', 18).send();

  // now wait for the deploy txs to be mined. This way we send all tx in the same rollup.
  const deployReceipt = await deployTx.wait();
  if (deployReceipt.status !== TxStatus.MINED) {
    throw new Error(`Deploy token tx status is ${deployReceipt.status}`);
  }
  const token = await TokenContract.at(deployReceipt.contractAddress!, wallet);

  // deploy l2 token bridge and attach to the portal
  const bridge = await TokenBridgeContract.deploy(wallet, token.address)
    .send({ portalContract: tokenPortalAddress })
    .deployed();

  if ((await token.methods.admin().view()) !== owner.toBigInt()) {
    throw new Error(`Token admin is not ${owner}`);
  }

  if (!(await bridge.methods.token().view()).equals(token.address)) {
    throw new Error(`Bridge token is not ${token.address}`);
  }

  // make the bridge a minter on the token:
  const makeMinterTx = token.methods.set_minter(bridge.address, true).send();
  const makeMinterReceipt = await makeMinterTx.wait();
  if (makeMinterReceipt.status !== TxStatus.MINED) {
    throw new Error(`Make bridge a minter tx status is ${makeMinterReceipt.status}`);
  }
  if ((await token.methods.is_minter(bridge.address).view()) === 1n) {
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
    pxeService: PXE,
    publicClient: PublicClient<HttpTransport, Chain>,
    walletClient: any,
    wallet: Wallet,
    logger: DebugLogger,
    underlyingERC20Address?: EthAddress,
  ): Promise<CrossChainTestHarness> {
    const ethAccount = EthAddress.fromString((await walletClient.getAddresses())[0]);
    const owner = wallet.getCompleteAddress();
    const l1ContractAddresses = (await pxeService.getNodeInfo()).l1ContractAddresses;

    const outbox = getContract({
      address: l1ContractAddresses.outboxAddress.toString(),
      abi: OutboxAbi,
      publicClient,
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
      pxeService,
      logger,
      token,
      bridge,
      ethAccount,
      tokenPortalAddress,
      tokenPortal,
      underlyingERC20,
      outbox,
      publicClient,
      walletClient,
      owner.address,
    );
  }

  constructor(
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
    /** Message Bridge Outbox. */
    public outbox: any,
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
    await this.underlyingERC20.write.mint([this.ethAccount.toString(), amount], {} as any);
    expect(await this.underlyingERC20.read.balanceOf([this.ethAccount.toString()])).toBe(amount);
  }

  async getL1BalanceOf(address: EthAddress) {
    return await this.underlyingERC20.read.balanceOf([address.toString()]);
  }

  async sendTokensToPortalPublic(bridgeAmount: bigint, secretHash: Fr) {
    await this.underlyingERC20.write.approve([this.tokenPortalAddress.toString(), bridgeAmount], {} as any);

    // Deposit tokens to the TokenPortal
    const deadline = 2 ** 32 - 1; // max uint32

    this.logger('Sending messages to L1 portal to be consumed publicly');
    const args = [
      this.ownerAddress.toString(),
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

  async mintTokensPublicOnL2(amount: bigint) {
    this.logger('Minting tokens on L2 publicly');
    const tx = this.l2Token.methods.mint_public(this.ownerAddress, amount).send();
    const receipt = await tx.wait();
    expect(receipt.status).toBe(TxStatus.MINED);
  }

  async mintTokensPrivateOnL2(amount: bigint, secretHash: Fr) {
    const tx = this.l2Token.methods.mint_private(amount, secretHash).send();
    const receipt = await tx.wait();
    expect(receipt.status).toBe(TxStatus.MINED);
    await this.addPendingShieldNoteToPXE(amount, secretHash, receipt.txHash);
  }

  async performL2Transfer(transferAmount: bigint, receiverAddress: AztecAddress) {
    // send a transfer tx to force through rollup with the message included
    const transferTx = this.l2Token.methods
      .transfer_public(this.ownerAddress, receiverAddress, transferAmount, 0)
      .send();
    const receipt = await transferTx.wait();
    expect(receipt.status).toBe(TxStatus.MINED);
  }

  async consumeMessageOnAztecAndMintSecretly(
    secretHashForRedeemingMintedNotes: Fr,
    bridgeAmount: bigint,
    messageKey: Fr,
    secretForL2MessageConsumption: Fr,
  ) {
    this.logger('Consuming messages on L2 secretively');
    // Call the mint tokens function on the Aztec.nr contract
    const consumptionTx = this.l2Bridge.methods
      .claim_private(
        secretHashForRedeemingMintedNotes,
        bridgeAmount,
        this.ethAccount,
        messageKey,
        secretForL2MessageConsumption,
      )
      .send();
    const consumptionReceipt = await consumptionTx.wait();
    expect(consumptionReceipt.status).toBe(TxStatus.MINED);

    await this.addPendingShieldNoteToPXE(bridgeAmount, secretHashForRedeemingMintedNotes, consumptionReceipt.txHash);
  }

  async consumeMessageOnAztecAndMintPublicly(bridgeAmount: bigint, messageKey: Fr, secret: Fr) {
    this.logger('Consuming messages on L2 Publicly');
    // Call the mint tokens function on the Aztec.nr contract
    const tx = this.l2Bridge.methods
      .claim_public(this.ownerAddress, bridgeAmount, this.ethAccount, messageKey, secret)
      .send();
    const receipt = await tx.wait();
    expect(receipt.status).toBe(TxStatus.MINED);
  }

  async withdrawPrivateFromAztecToL1(withdrawAmount: bigint, nonce: Fr = Fr.ZERO) {
    const withdrawTx = this.l2Bridge.methods
      .exit_to_l1_private(this.l2Token.address, this.ethAccount, withdrawAmount, EthAddress.ZERO, nonce)
      .send();
    const withdrawReceipt = await withdrawTx.wait();
    expect(withdrawReceipt.status).toBe(TxStatus.MINED);
  }

  async withdrawPublicFromAztecToL1(withdrawAmount: bigint, nonce: Fr = Fr.ZERO) {
    const withdrawTx = this.l2Bridge.methods
      .exit_to_l1_public(this.ethAccount, withdrawAmount, EthAddress.ZERO, nonce)
      .send();
    const withdrawReceipt = await withdrawTx.wait();
    expect(withdrawReceipt.status).toBe(TxStatus.MINED);
  }

  async getL2PrivateBalanceOf(owner: AztecAddress) {
    return await this.l2Token.methods.balance_of_private(owner).view({ from: owner });
  }

  async expectPrivateBalanceOnL2(owner: AztecAddress, expectedBalance: bigint) {
    const balance = await this.getL2PrivateBalanceOf(owner);
    this.logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  }

  async getL2PublicBalanceOf(owner: AztecAddress) {
    return await this.l2Token.methods.balance_of_public(owner).view();
  }

  async expectPublicBalanceOnL2(owner: AztecAddress, expectedBalance: bigint) {
    const balance = await this.getL2PublicBalanceOf(owner);
    expect(balance).toBe(expectedBalance);
  }

  async checkEntryIsNotInOutbox(withdrawAmount: bigint, callerOnL1: EthAddress = EthAddress.ZERO): Promise<Fr> {
    this.logger('Ensure that the entry is not in outbox yet');

    const content = Fr.fromBufferReduce(
      sha256(
        Buffer.concat([
          Buffer.from(getFunctionSelector('withdraw(address,uint256,address)').substring(2), 'hex'),
          this.ethAccount.toBuffer32(),
          new Fr(withdrawAmount).toBuffer(),
          callerOnL1.toBuffer32(),
        ]),
      ),
    );
    const entryKey = Fr.fromBufferReduce(
      sha256(
        Buffer.concat([
          this.l2Bridge.address.toBuffer(),
          new Fr(1).toBuffer(), // aztec version
          this.tokenPortalAddress.toBuffer32() ?? Buffer.alloc(32, 0),
          new Fr(this.publicClient.chain.id).toBuffer(), // chain id
          content.toBuffer(),
        ]),
      ),
    );
    expect(await this.outbox.read.contains([entryKey.toString()])).toBeFalsy();

    return entryKey;
  }

  async withdrawFundsFromBridgeOnL1(withdrawAmount: bigint, entryKey: Fr) {
    this.logger('Send L1 tx to consume entry and withdraw funds');
    // Call function on L1 contract to consume the message
    const { request: withdrawRequest, result: withdrawEntryKey } = await this.tokenPortal.simulate.withdraw([
      this.ethAccount.toString(),
      withdrawAmount,
      false,
    ]);

    expect(withdrawEntryKey).toBe(entryKey.toString());
    expect(await this.outbox.read.contains([withdrawEntryKey])).toBeTruthy();

    await this.walletClient.writeContract(withdrawRequest);
    return withdrawEntryKey;
  }

  async shieldFundsOnL2(shieldAmount: bigint, secretHash: Fr) {
    this.logger('Shielding funds on L2');
    const shieldTx = this.l2Token.methods.shield(this.ownerAddress, shieldAmount, secretHash, 0).send();
    const shieldReceipt = await shieldTx.wait();
    expect(shieldReceipt.status).toBe(TxStatus.MINED);

    await this.addPendingShieldNoteToPXE(shieldAmount, secretHash, shieldReceipt.txHash);
  }

  async addPendingShieldNoteToPXE(shieldAmount: bigint, secretHash: Fr, txHash: TxHash) {
    this.logger('Adding note to PXE');
    const storageSlot = new Fr(5);
    const note = new Note([new Fr(shieldAmount), secretHash]);
    const extendedNote = new ExtendedNote(note, this.ownerAddress, this.l2Token.address, storageSlot, txHash);
    await this.pxeService.addNote(extendedNote);
  }

  async redeemShieldPrivatelyOnL2(shieldAmount: bigint, secret: Fr) {
    this.logger('Spending commitment in private call');
    const privateTx = this.l2Token.methods.redeem_shield(this.ownerAddress, shieldAmount, secret).send();
    const privateReceipt = await privateTx.wait();
    expect(privateReceipt.status).toBe(TxStatus.MINED);
  }

  async unshieldTokensOnL2(unshieldAmount: bigint, nonce = Fr.ZERO) {
    this.logger('Unshielding tokens');
    const unshieldTx = this.l2Token.methods
      .unshield(this.ownerAddress, this.ownerAddress, unshieldAmount, nonce)
      .send();
    const unshieldReceipt = await unshieldTx.wait();
    expect(unshieldReceipt.status).toBe(TxStatus.MINED);
  }
}
// docs:end:cross_chain_test_harness
