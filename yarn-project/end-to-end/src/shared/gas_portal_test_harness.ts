import {
  type AztecAddress,
  type AztecNode,
  type DebugLogger,
  EthAddress,
  Fr,
  type PXE,
  type Wallet,
  computeSecretHash,
} from '@aztec/aztec.js';
import { FeeJuicePortalAbi, OutboxAbi, PortalERC20Abi } from '@aztec/l1-artifacts';
import { FeeJuiceContract } from '@aztec/noir-contracts.js';
import { FeeJuiceAddress, getCanonicalFeeJuice } from '@aztec/protocol-contracts/fee-juice';

import {
  type Account,
  type Chain,
  type GetContractReturnType,
  type HttpTransport,
  type PublicClient,
  type WalletClient,
  getContract,
} from 'viem';

export interface IGasBridgingTestHarness {
  getL1FeeJuiceBalance(address: EthAddress): Promise<bigint>;
  prepareTokensOnL1(
    l1TokenBalance: bigint,
    bridgeAmount: bigint,
    owner: AztecAddress,
  ): Promise<{ secret: Fr; secretHash: Fr; msgHash: Fr }>;
  bridgeFromL1ToL2(l1TokenBalance: bigint, bridgeAmount: bigint, owner: AztecAddress): Promise<void>;
  l2Token: FeeJuiceContract;
  l1FeeJuiceAddress: EthAddress;
}

export interface FeeJuicePortalTestingHarnessFactoryConfig {
  aztecNode: AztecNode;
  pxeService: PXE;
  publicClient: PublicClient<HttpTransport, Chain>;
  walletClient: WalletClient<HttpTransport, Chain, Account>;
  wallet: Wallet;
  logger: DebugLogger;
  mockL1?: boolean;
}

export class FeeJuicePortalTestingHarnessFactory {
  private constructor(private config: FeeJuicePortalTestingHarnessFactoryConfig) {}

  private async createMock() {
    const wallet = this.config.wallet;

    // In this case we are not using a portal we just yolo it.
    const gasL2 = await FeeJuiceContract.deploy(wallet)
      .send({ contractAddressSalt: getCanonicalFeeJuice().instance.salt })
      .deployed();
    return Promise.resolve(new MockGasBridgingTestHarness(gasL2, EthAddress.ZERO));
  }

  private async createReal() {
    const { aztecNode, pxeService, publicClient, walletClient, wallet, logger } = this.config;

    const ethAccount = EthAddress.fromString((await walletClient.getAddresses())[0]);
    const l1ContractAddresses = (await pxeService.getNodeInfo()).l1ContractAddresses;

    const feeJuiceAddress = l1ContractAddresses.feeJuiceAddress;
    const feeJuicePortalAddress = l1ContractAddresses.feeJuicePortalAddress;

    if (feeJuiceAddress.isZero() || feeJuicePortalAddress.isZero()) {
      throw new Error('Gas portal not deployed on L1');
    }

    const outbox = getContract({
      address: l1ContractAddresses.outboxAddress.toString(),
      abi: OutboxAbi,
      client: walletClient,
    });

    const gasL1 = getContract({
      address: feeJuiceAddress.toString(),
      abi: PortalERC20Abi,
      client: walletClient,
    });

    const feeJuicePortal = getContract({
      address: feeJuicePortalAddress.toString(),
      abi: FeeJuicePortalAbi,
      client: walletClient,
    });

    const gasL2 = await FeeJuiceContract.at(FeeJuiceAddress, wallet);

    return new GasBridgingTestHarness(
      aztecNode,
      pxeService,
      logger,
      gasL2,
      ethAccount,
      feeJuicePortalAddress,
      feeJuicePortal,
      gasL1,
      outbox,
      publicClient,
      walletClient,
    );
  }

  static create(config: FeeJuicePortalTestingHarnessFactoryConfig): Promise<IGasBridgingTestHarness> {
    const factory = new FeeJuicePortalTestingHarnessFactory(config);
    if (config.mockL1) {
      return factory.createMock();
    } else {
      return factory.createReal();
    }
  }
}

class MockGasBridgingTestHarness implements IGasBridgingTestHarness {
  constructor(public l2Token: FeeJuiceContract, public l1FeeJuiceAddress: EthAddress) {}
  prepareTokensOnL1(
    _l1TokenBalance: bigint,
    _bridgeAmount: bigint,
    _owner: AztecAddress,
  ): Promise<{ secret: Fr; secretHash: Fr; msgHash: Fr }> {
    throw new Error('Cannot prepare tokens on mocked L1.');
  }
  async bridgeFromL1ToL2(_l1TokenBalance: bigint, bridgeAmount: bigint, owner: AztecAddress): Promise<void> {
    await this.l2Token.methods.mint_public(owner, bridgeAmount).send().wait();
  }
  getL1FeeJuiceBalance(_address: EthAddress): Promise<bigint> {
    throw new Error('Cannot get Fee Juice balance on mocked L1.');
  }
}

/**
 * A Class for testing cross chain interactions, contains common interactions
 * shared between cross chain tests.
 */
class GasBridgingTestHarness implements IGasBridgingTestHarness {
  constructor(
    /** Aztec node */
    public aztecNode: AztecNode,
    /** Private eXecution Environment (PXE). */
    public pxeService: PXE,
    /** Logger. */
    public logger: DebugLogger,

    /** L2 Token/Bridge contract. */
    public l2Token: FeeJuiceContract,

    /** Eth account to interact with. */
    public ethAccount: EthAddress,

    /** Portal address. */
    public tokenPortalAddress: EthAddress,
    /** Token portal instance. */
    public tokenPortal: GetContractReturnType<typeof FeeJuicePortalAbi, WalletClient<HttpTransport, Chain, Account>>,
    /** Underlying token for portal tests. */
    public underlyingERC20: GetContractReturnType<typeof PortalERC20Abi, WalletClient<HttpTransport, Chain, Account>>,
    /** Message Bridge Outbox. */
    public outbox: GetContractReturnType<typeof OutboxAbi, PublicClient<HttpTransport, Chain>>,
    /** Viem Public client instance. */
    public publicClient: PublicClient<HttpTransport, Chain>,
    /** Viem Wallet Client instance. */
    public walletClient: WalletClient,
  ) {}

  get l1FeeJuiceAddress() {
    return EthAddress.fromString(this.underlyingERC20.address);
  }

  generateClaimSecret(): [Fr, Fr] {
    this.logger.debug("Generating a claim secret using pedersen's hash function");
    const secret = Fr.random();
    const secretHash = computeSecretHash(secret);
    this.logger.info('Generated claim secret: ' + secretHash.toString());
    return [secret, secretHash];
  }

  async mintTokensOnL1(amount: bigint) {
    this.logger.info('Minting tokens on L1');
    await this.publicClient.waitForTransactionReceipt({
      hash: await this.underlyingERC20.write.mint([this.ethAccount.toString(), amount]),
    });
    expect(await this.underlyingERC20.read.balanceOf([this.ethAccount.toString()])).toBe(amount);
  }

  async getL1FeeJuiceBalance(address: EthAddress) {
    return await this.underlyingERC20.read.balanceOf([address.toString()]);
  }

  async sendTokensToPortalPublic(bridgeAmount: bigint, l2Address: AztecAddress, secretHash: Fr) {
    await this.publicClient.waitForTransactionReceipt({
      hash: await this.underlyingERC20.write.approve([this.tokenPortalAddress.toString(), bridgeAmount]),
    });

    // Deposit tokens to the TokenPortal
    this.logger.info('Sending messages to L1 portal to be consumed publicly');
    const args = [l2Address.toString(), bridgeAmount, secretHash.toString()] as const;
    const { result: messageHash } = await this.tokenPortal.simulate.depositToAztecPublic(args, {
      account: this.ethAccount.toString(),
    } as any);
    await this.publicClient.waitForTransactionReceipt({
      hash: await this.tokenPortal.write.depositToAztecPublic(args),
    });

    return Fr.fromString(messageHash);
  }

  async consumeMessageOnAztecAndMintPublicly(bridgeAmount: bigint, owner: AztecAddress, secret: Fr, leafIndex: bigint) {
    this.logger.info('Consuming messages on L2 Publicly');
    // Call the mint tokens function on the Aztec.nr contract
    await this.l2Token.methods.claim_public(owner, bridgeAmount, secret, leafIndex).send().wait();
  }

  async getL2PublicBalanceOf(owner: AztecAddress) {
    return await this.l2Token.methods.balance_of_public(owner).simulate();
  }

  async expectPublicBalanceOnL2(owner: AztecAddress, expectedBalance: bigint) {
    const balance = await this.getL2PublicBalanceOf(owner);
    expect(balance).toBe(expectedBalance);
  }

  async prepareTokensOnL1(l1TokenBalance: bigint, bridgeAmount: bigint, owner: AztecAddress) {
    const [secret, secretHash] = this.generateClaimSecret();

    // Mint tokens on L1
    await this.mintTokensOnL1(l1TokenBalance);

    // Deposit tokens to the TokenPortal
    const msgHash = await this.sendTokensToPortalPublic(bridgeAmount, owner, secretHash);
    expect(await this.getL1FeeJuiceBalance(this.ethAccount)).toBe(l1TokenBalance - bridgeAmount);

    // Perform an unrelated transactions on L2 to progress the rollup by 2 blocks.
    await this.l2Token.methods.check_balance(0).send().wait();
    await this.l2Token.methods.check_balance(0).send().wait();

    return { secret, msgHash, secretHash };
  }

  async bridgeFromL1ToL2(l1TokenBalance: bigint, bridgeAmount: bigint, owner: AztecAddress) {
    // Prepare the tokens on the L1 side
    const { secret, msgHash } = await this.prepareTokensOnL1(l1TokenBalance, bridgeAmount, owner);

    // Get message leaf index, needed for claiming in public
    const maybeIndexAndPath = await this.aztecNode.getL1ToL2MessageMembershipWitness('latest', msgHash, 0n);
    expect(maybeIndexAndPath).toBeDefined();
    const messageLeafIndex = maybeIndexAndPath![0];

    // Consume L1-> L2 message and mint public tokens on L2
    await this.consumeMessageOnAztecAndMintPublicly(bridgeAmount, owner, secret, messageLeafIndex);
    await this.expectPublicBalanceOnL2(owner, bridgeAmount);
  }
}
// docs:end:cross_chain_test_harness
