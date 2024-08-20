// REFACTOR: This file has been shamelessly copied from yarn-project/end-to-end/src/shared/gas_portal_test_harness.ts
// We should make this a shared utility in the aztec.js package.
import { type AztecAddress, type DebugLogger, type EthAddress, Fr, type PXE, computeSecretHash } from '@aztec/aztec.js';
import { FeeJuicePortalAbi, PortalERC20Abi, TokenPortalAbi } from '@aztec/l1-artifacts';

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

export interface L2Claim {
  claimSecret: Fr;
  claimAmount: Fr;
  messageHash: `0x${string}`;
}

function stringifyEthAddress(address: EthAddress | Hex, name?: string) {
  return name ? `${name} (${address.toString()})` : address.toString();
}

function generateClaimSecret(): [Fr, Fr] {
  const secret = Fr.random();
  const secretHash = computeSecretHash(secret);
  return [secret, secretHash];
}

class L1TokenManager {
  private contract: GetContractReturnType<typeof PortalERC20Abi, WalletClient<HttpTransport, Chain, Account>>;

  public constructor(
    public readonly address: EthAddress,
    private publicClient: PublicClient<HttpTransport, Chain>,
    private walletClient: WalletClient<HttpTransport, Chain, Account>,
    private logger: DebugLogger,
  ) {
    this.contract = getContract({
      address: this.address.toString(),
      abi: PortalERC20Abi,
      client: this.walletClient,
    });
  }

  public async getL1TokenBalance(address: Hex) {
    return await this.contract.read.balanceOf([address]);
  }

  public async mint(amount: bigint, address: Hex, addressName = '') {
    this.logger.info(`Minting ${amount} tokens for ${stringifyEthAddress(address, addressName)}`);
    await this.publicClient.waitForTransactionReceipt({
      hash: await this.contract.write.mint([address, amount]),
    });
  }

  public async approve(amount: bigint, address: Hex, addressName = '') {
    this.logger.info(`Approving ${amount} tokens for ${stringifyEthAddress(address, addressName)}`);
    await this.publicClient.waitForTransactionReceipt({
      hash: await this.contract.write.approve([address, amount]),
    });
  }
}

export class FeeJuicePortalManager {
  tokenManager: L1TokenManager;
  contract: GetContractReturnType<typeof FeeJuicePortalAbi, WalletClient<HttpTransport, Chain, Account>>;

  constructor(
    portalAddress: EthAddress,
    tokenAddress: EthAddress,
    private publicClient: PublicClient<HttpTransport, Chain>,
    private walletClient: WalletClient<HttpTransport, Chain, Account>,
    /** Logger. */
    private logger: DebugLogger,
  ) {
    this.tokenManager = new L1TokenManager(tokenAddress, publicClient, walletClient, logger);
    this.contract = getContract({
      address: portalAddress.toString(),
      abi: FeeJuicePortalAbi,
      client: this.walletClient,
    });
  }

  public async bridgeTokensPublic(to: AztecAddress, amount: bigint, mint = false): Promise<L2Claim> {
    const [claimSecret, claimSecretHash] = generateClaimSecret();
    if (mint) {
      await this.tokenManager.mint(amount, this.walletClient.account.address);
    }

    await this.tokenManager.approve(amount, this.contract.address, 'FeeJuice Portal');

    this.logger.info('Sending L1 Fee Juice to L2 to be claimed publicly');
    const args = [to.toString(), amount, claimSecretHash.toString()] as const;

    const { result: messageHash } = await this.contract.simulate.depositToAztecPublic(args);

    await this.publicClient.waitForTransactionReceipt({
      hash: await this.contract.write.depositToAztecPublic(args),
    });

    return {
      claimAmount: new Fr(amount),
      claimSecret,
      messageHash,
    };
  }

  public static async new(
    pxe: PXE,
    publicClient: PublicClient<HttpTransport, Chain>,
    walletClient: WalletClient<HttpTransport, Chain, Account>,
    logger: DebugLogger,
  ): Promise<FeeJuicePortalManager> {
    const {
      l1ContractAddresses: { feeJuiceAddress, feeJuicePortalAddress },
    } = await pxe.getNodeInfo();

    if (feeJuiceAddress.isZero() || feeJuicePortalAddress.isZero()) {
      throw new Error('Portal or token not deployed on L1');
    }

    return new FeeJuicePortalManager(feeJuicePortalAddress, feeJuiceAddress, publicClient, walletClient, logger);
  }
}

export class L1PortalManager {
  contract: GetContractReturnType<typeof TokenPortalAbi, WalletClient<HttpTransport, Chain, Account>>;
  private tokenManager: L1TokenManager;

  constructor(
    portalAddress: EthAddress,
    tokenAddress: EthAddress,
    private publicClient: PublicClient<HttpTransport, Chain>,
    private walletClient: WalletClient<HttpTransport, Chain, Account>,
    private logger: DebugLogger,
  ) {
    this.tokenManager = new L1TokenManager(tokenAddress, publicClient, walletClient, logger);
    this.contract = getContract({
      address: portalAddress.toString(),
      abi: TokenPortalAbi,
      client: this.walletClient,
    });
  }

  public bridgeTokensPublic(to: AztecAddress, amount: bigint, mint = false): Promise<L2Claim> {
    return this.bridgeTokens(to, amount, mint, /* privateTransfer */ false);
  }

  public bridgeTokensPrivate(to: AztecAddress, amount: bigint, mint = false): Promise<L2Claim> {
    return this.bridgeTokens(to, amount, mint, /* privateTransfer */ true);
  }

  private async bridgeTokens(
    to: AztecAddress,
    amount: bigint,
    mint: boolean,
    privateTransfer: boolean,
  ): Promise<L2Claim> {
    const [claimSecret, claimSecretHash] = generateClaimSecret();

    if (mint) {
      await this.tokenManager.mint(amount, this.walletClient.account.address);
    }

    await this.tokenManager.approve(amount, this.contract.address, 'TokenPortal');

    let messageHash: `0x${string}`;

    if (privateTransfer) {
      const secret = Fr.random();
      const secretHash = computeSecretHash(secret);
      this.logger.info('Sending L1 tokens to L2 to be claimed privately');
      ({ result: messageHash } = await this.contract.simulate.depositToAztecPrivate([
        secretHash.toString(),
        amount,
        claimSecretHash.toString(),
      ]));

      await this.publicClient.waitForTransactionReceipt({
        hash: await this.contract.write.depositToAztecPrivate([
          secretHash.toString(),
          amount,
          claimSecretHash.toString(),
        ]),
      });
      this.logger.info(`Redeem shield secret: ${secret.toString()}, secret hash: ${secretHash.toString()}`);
    } else {
      this.logger.info('Sending L1 tokens to L2 to be claimed publicly');
      ({ result: messageHash } = await this.contract.simulate.depositToAztecPublic([
        to.toString(),
        amount,
        claimSecretHash.toString(),
      ]));

      await this.publicClient.waitForTransactionReceipt({
        hash: await this.contract.write.depositToAztecPublic([to.toString(), amount, claimSecretHash.toString()]),
      });
    }

    return {
      claimAmount: new Fr(amount),
      claimSecret,
      messageHash,
    };
  }
}
