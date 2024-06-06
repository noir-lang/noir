// REFACTOR: This file has been shamelessly copied from yarn-project/end-to-end/src/shared/gas_portal_test_harness.ts
// We should make this a shared utility in the aztec.js package.
import { type AztecAddress, type DebugLogger, EthAddress, Fr, type PXE, computeSecretHash } from '@aztec/aztec.js';
import { GasPortalAbi, OutboxAbi, PortalERC20Abi } from '@aztec/l1-artifacts';

import {
  type Account,
  type Chain,
  type GetContractReturnType,
  type HttpTransport,
  type PublicClient,
  type WalletClient,
  getContract,
} from 'viem';

export interface GasPortalManagerFactoryConfig {
  pxeService: PXE;
  publicClient: PublicClient<HttpTransport, Chain>;
  walletClient: WalletClient<HttpTransport, Chain, Account>;
  logger: DebugLogger;
}

export class GasPortalManagerFactory {
  private constructor(private config: GasPortalManagerFactoryConfig) {}

  private async createReal() {
    const { pxeService, publicClient, walletClient, logger } = this.config;

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

    return new GasPortalManager(
      pxeService,
      logger,
      ethAccount,
      gasPortalAddress,
      gasPortal,
      gasL1,
      outbox,
      publicClient,
      walletClient,
    );
  }

  static create(config: GasPortalManagerFactoryConfig): Promise<GasPortalManager> {
    const factory = new GasPortalManagerFactory(config);
    return factory.createReal();
  }
}

/**
 * A Class for testing cross chain interactions, contains common interactions
 * shared between cross chain tests.
 */
class GasPortalManager {
  constructor(
    /** Private eXecution Environment (PXE). */
    public pxeService: PXE,
    /** Logger. */
    public logger: DebugLogger,
    /** Eth account to interact with. */
    public ethAccount: EthAddress,
    /** Portal address. */
    public tokenPortalAddress: EthAddress,
    /** Token portal instance. */
    public tokenPortal: GetContractReturnType<typeof GasPortalAbi, WalletClient<HttpTransport, Chain, Account>>,
    /** Underlying token for portal tests. */
    public underlyingERC20: GetContractReturnType<typeof PortalERC20Abi, WalletClient<HttpTransport, Chain, Account>>,
    /** Message Bridge Outbox. */
    public outbox: GetContractReturnType<typeof OutboxAbi, PublicClient<HttpTransport, Chain>>,
    /** Viem Public client instance. */
    public publicClient: PublicClient<HttpTransport, Chain>,
    /** Viem Wallet Client instance. */
    public walletClient: WalletClient,
  ) {}

  get l1GasTokenAddress() {
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
    this.logger.info(
      `Minting tokens on L1 for ${this.ethAccount.toString()} in contract ${this.underlyingERC20.address}`,
    );
    await this.publicClient.waitForTransactionReceipt({
      hash: await this.underlyingERC20.write.mint([this.ethAccount.toString(), amount]),
    });
  }

  async getL1GasTokenBalance(address: EthAddress) {
    return await this.underlyingERC20.read.balanceOf([address.toString()]);
  }

  async sendTokensToPortalPublic(bridgeAmount: bigint, l2Address: AztecAddress, secretHash: Fr) {
    this.logger.info(
      `Approving erc20 tokens for the TokenPortal at ${this.tokenPortalAddress.toString()} ${this.tokenPortal.address}`,
    );
    await this.publicClient.waitForTransactionReceipt({
      hash: await this.underlyingERC20.write.approve([this.tokenPortalAddress.toString(), bridgeAmount]),
    });

    // Deposit tokens to the TokenPortal
    this.logger.info(
      `Simulating token portal deposit configured for token ${await this.tokenPortal.read.l2TokenAddress()} with registry ${await this.tokenPortal.read.registry()} to retrieve message hash`,
    );
    const args = [l2Address.toString(), bridgeAmount, secretHash.toString()] as const;
    const { result: messageHash } = await this.tokenPortal.simulate.depositToAztecPublic(args);
    this.logger.info('Sending messages to L1 portal to be consumed publicly');
    await this.publicClient.waitForTransactionReceipt({
      hash: await this.tokenPortal.write.depositToAztecPublic(args),
    });

    return Fr.fromString(messageHash);
  }

  async prepareTokensOnL1(l1TokenBalance: bigint, bridgeAmount: bigint, owner: AztecAddress) {
    const [secret, secretHash] = this.generateClaimSecret();

    // Mint tokens on L1
    await this.mintTokensOnL1(l1TokenBalance);

    // Deposit tokens to the TokenPortal
    const msgHash = await this.sendTokensToPortalPublic(bridgeAmount, owner, secretHash);

    return { secret, msgHash, secretHash };
  }
}
