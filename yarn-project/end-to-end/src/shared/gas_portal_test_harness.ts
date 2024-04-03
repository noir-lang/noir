import {
  type AztecAddress,
  type DebugLogger,
  EthAddress,
  Fr,
  type PXE,
  type Wallet,
  computeMessageSecretHash,
} from '@aztec/aztec.js';
import { GasPortalAbi, OutboxAbi, PortalERC20Abi } from '@aztec/l1-artifacts';
import { GasTokenContract } from '@aztec/noir-contracts.js';
import { getCanonicalGasToken, getCanonicalGasTokenAddress } from '@aztec/protocol-contracts/gas-token';

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
  bridgeFromL1ToL2(l1TokenBalance: bigint, bridgeAmount: bigint, owner: AztecAddress): Promise<void>;
  l2Token: GasTokenContract;
}

export interface GasPortalTestingHarnessFactoryConfig {
  pxeService: PXE;
  publicClient: PublicClient<HttpTransport, Chain>;
  walletClient: WalletClient<HttpTransport, Chain, Account>;
  wallet: Wallet;
  logger: DebugLogger;
  mockL1?: boolean;
}
export class GasPortalTestingHarnessFactory {
  private constructor(private config: GasPortalTestingHarnessFactoryConfig) {}

  private async createMock() {
    const wallet = this.config.wallet;

    const gasL2 = await GasTokenContract.deploy(wallet)
      .send({
        contractAddressSalt: getCanonicalGasToken(EthAddress.ZERO).instance.salt,
      })
      .deployed();
    return Promise.resolve(new MockGasBridgingTestHarness(gasL2));
  }

  private async createReal() {
    const { pxeService, publicClient, walletClient, wallet, logger } = this.config;

    const ethAccount = EthAddress.fromString((await walletClient.getAddresses())[0]);
    const l1ContractAddresses = (await pxeService.getNodeInfo()).l1ContractAddresses;

    const gasTokenAddress = l1ContractAddresses.gasTokenAddress;
    const gasPortalAddress = l1ContractAddresses.gasPortalAddress;

    if (gasTokenAddress.isZero() || gasPortalAddress.isZero()) {
      throw new Error('Gas portal not deployed on L1');
    }

    const outbox = getContract({
      address: l1ContractAddresses.outboxAddress.toString(),
      abi: OutboxAbi,
      client: walletClient,
    });

    const gasL1 = getContract({
      address: gasTokenAddress.toString(),
      abi: PortalERC20Abi,
      client: walletClient,
    });

    const gasPortal = getContract({
      address: gasPortalAddress.toString(),
      abi: GasPortalAbi,
      client: walletClient,
    });

    const gasL2 = await GasTokenContract.at(getCanonicalGasTokenAddress(gasPortalAddress), wallet);

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

  static create(config: GasPortalTestingHarnessFactoryConfig): Promise<IGasBridgingTestHarness> {
    const factory = new GasPortalTestingHarnessFactory(config);
    if (config.mockL1) {
      return factory.createMock();
    } else {
      return factory.createReal();
    }
  }
}

class MockGasBridgingTestHarness implements IGasBridgingTestHarness {
  constructor(public l2Token: GasTokenContract) {}
  async bridgeFromL1ToL2(_l1TokenBalance: bigint, bridgeAmount: bigint, owner: AztecAddress): Promise<void> {
    await this.l2Token.methods.mint_public(owner, bridgeAmount).send().wait();
  }
}

/**
 * A Class for testing cross chain interactions, contains common interactions
 * shared between cross chain tests.
 */
class GasBridgingTestHarness implements IGasBridgingTestHarness {
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
    public outbox: GetContractReturnType<typeof OutboxAbi, PublicClient<HttpTransport, Chain>>,
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
    this.logger('Sending messages to L1 portal to be consumed publicly');
    const args = [l2Address.toString(), bridgeAmount, secretHash.toString()] as const;
    const { result: messageHash } = await this.tokenPortal.simulate.depositToAztecPublic(args, {
      account: this.ethAccount.toString(),
    } as any);
    await this.tokenPortal.write.depositToAztecPublic(args, {} as any);

    return Fr.fromString(messageHash);
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
    const { result: messageHash } = await this.tokenPortal.simulate.depositToAztecPrivate(args, {
      account: this.ethAccount.toString(),
    } as any);
    await this.tokenPortal.write.depositToAztecPrivate(args, {} as any);

    return Fr.fromString(messageHash);
  }

  async consumeMessageOnAztecAndMintPublicly(bridgeAmount: bigint, owner: AztecAddress, secret: Fr) {
    this.logger('Consuming messages on L2 Publicly');
    // Call the mint tokens function on the Aztec.nr contract
    await this.l2Token.methods.claim_public(owner, bridgeAmount, secret).send().wait();
  }

  async getL2PublicBalanceOf(owner: AztecAddress) {
    return await this.l2Token.methods.balance_of_public(owner).simulate();
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
    await this.sendTokensToPortalPublic(bridgeAmount, owner, secretHash);
    expect(await this.getL1BalanceOf(this.ethAccount)).toBe(l1TokenBalance - bridgeAmount);

    // Perform an unrelated transactions on L2 to progress the rollup by 2 blocks.
    await this.l2Token.methods.check_balance(0).send().wait();
    await this.l2Token.methods.check_balance(0).send().wait();

    // 3. Consume L1-> L2 message and mint public tokens on L2
    await this.consumeMessageOnAztecAndMintPublicly(bridgeAmount, owner, secret);
    await this.expectPublicBalanceOnL2(owner, bridgeAmount);
  }
}
// docs:end:cross_chain_test_harness
