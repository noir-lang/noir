// REFACTOR: This file has been shamelessly copied from yarn-project/end-to-end/src/shared/gas_portal_test_harness.ts
// We should make this a shared utility in the aztec.js package.
import { type AztecAddress, type DebugLogger, type EthAddress, Fr, type PXE, computeSecretHash } from '@aztec/aztec.js';
import { GasPortalAbi, PortalERC20Abi, TokenPortalAbi } from '@aztec/l1-artifacts';

import {
  type Account,
  type Chain,
  type GetContractReturnType,
  type Hex,
  type HttpTransport,
  type PublicClient,
  type WalletClient,
  getContract,
} from 'viem';

/**
 * A Class for testing cross chain interactions, contains common interactions
 * shared between cross chain tests.
 */
abstract class PortalManager {
  protected constructor(
    /** Underlying token for portal tests. */
    public underlyingERC20Address: EthAddress,
    /** Portal address. */
    public tokenPortalAddress: EthAddress,
    public publicClient: PublicClient<HttpTransport, Chain>,
    public walletClient: WalletClient<HttpTransport, Chain, Account>,
    /** Logger. */
    public logger: DebugLogger,
  ) {}

  generateClaimSecret(): [Fr, Fr] {
    this.logger.debug("Generating a claim secret using pedersen's hash function");
    const secret = Fr.random();
    const secretHash = computeSecretHash(secret);
    this.logger.info('Generated claim secret: ' + secretHash.toString());
    return [secret, secretHash];
  }

  getERC20Contract(): GetContractReturnType<typeof PortalERC20Abi, WalletClient<HttpTransport, Chain, Account>> {
    return getContract({
      address: this.underlyingERC20Address.toString(),
      abi: PortalERC20Abi,
      client: this.walletClient,
    });
  }

  async mintTokensOnL1(amount: bigint) {
    this.logger.info(
      `Minting tokens on L1 for ${this.walletClient.account.address} in contract ${this.underlyingERC20Address}`,
    );
    await this.publicClient.waitForTransactionReceipt({
      hash: await this.getERC20Contract().write.mint([this.walletClient.account.address, amount]),
    });
  }

  async getL1TokenBalance(address: EthAddress) {
    return await this.getERC20Contract().read.balanceOf([address.toString()]);
  }

  protected async sendTokensToPortalPublic(bridgeAmount: bigint, l2Address: AztecAddress, secretHash: Fr) {
    this.logger.info(`Approving erc20 tokens for the TokenPortal at ${this.tokenPortalAddress.toString()}`);
    await this.publicClient.waitForTransactionReceipt({
      hash: await this.getERC20Contract().write.approve([this.tokenPortalAddress.toString(), bridgeAmount]),
    });

    const messageHash = await this.bridgeTokens(l2Address, bridgeAmount, secretHash);
    return Fr.fromString(messageHash);
  }

  protected abstract bridgeTokens(to: AztecAddress, amount: bigint, secretHash: Fr): Promise<Hex>;

  async prepareTokensOnL1(l1TokenBalance: bigint, bridgeAmount: bigint, owner: AztecAddress, mint = true) {
    const [secret, secretHash] = this.generateClaimSecret();

    // Mint tokens on L1
    if (mint) {
      await this.mintTokensOnL1(l1TokenBalance);
    }

    // Deposit tokens to the TokenPortal
    const msgHash = await this.sendTokensToPortalPublic(bridgeAmount, owner, secretHash);

    return { secret, msgHash, secretHash };
  }
}

export class FeeJuicePortalManager extends PortalManager {
  async bridgeTokens(to: AztecAddress, amount: bigint, secretHash: Fr): Promise<Hex> {
    const portal = getContract({
      address: this.tokenPortalAddress.toString(),
      abi: GasPortalAbi,
      client: this.walletClient,
    });

    this.logger.info(
      `Simulating token portal deposit configured for token ${await portal.read.l2TokenAddress()} with registry ${await portal.read.registry()} to retrieve message hash`,
    );

    const args = [to.toString(), amount, secretHash.toString()] as const;
    const { result: messageHash } = await portal.simulate.depositToAztecPublic(args);
    this.logger.info('Sending messages to L1 portal to be consumed publicly');

    await this.publicClient.waitForTransactionReceipt({
      hash: await portal.write.depositToAztecPublic(args),
    });
    return messageHash;
  }

  public static async create(
    pxe: PXE,
    publicClient: PublicClient<HttpTransport, Chain>,
    walletClient: WalletClient<HttpTransport, Chain, Account>,
    logger: DebugLogger,
  ): Promise<PortalManager> {
    const {
      l1ContractAddresses: { gasTokenAddress, gasPortalAddress },
    } = await pxe.getNodeInfo();

    if (gasTokenAddress.isZero() || gasPortalAddress.isZero()) {
      throw new Error('Portal or token not deployed on L1');
    }

    return new FeeJuicePortalManager(gasTokenAddress, gasPortalAddress, publicClient, walletClient, logger);
  }
}

export class ERC20PortalManager extends PortalManager {
  async bridgeTokens(to: AztecAddress, amount: bigint, secretHash: Fr): Promise<Hex> {
    const portal = getContract({
      address: this.tokenPortalAddress.toString(),
      abi: TokenPortalAbi,
      client: this.walletClient,
    });

    this.logger.info(
      `Simulating token portal deposit configured for token ${await portal.read.l2Bridge()} with registry ${await portal.read.registry()} to retrieve message hash`,
    );

    const args = [to.toString(), amount, secretHash.toString()] as const;
    const { result: messageHash } = await portal.simulate.depositToAztecPublic(args);
    this.logger.info('Sending messages to L1 portal to be consumed publicly');

    await this.publicClient.waitForTransactionReceipt({
      hash: await portal.write.depositToAztecPublic(args),
    });
    return messageHash;
  }

  public static create(
    tokenAddress: EthAddress,
    portalAddress: EthAddress,
    publicClient: PublicClient<HttpTransport, Chain>,
    walletClient: WalletClient<HttpTransport, Chain, Account>,
    logger: DebugLogger,
  ): Promise<ERC20PortalManager> {
    return Promise.resolve(new ERC20PortalManager(tokenAddress, portalAddress, publicClient, walletClient, logger));
  }
}
