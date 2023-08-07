import { Fr, PublicKey, getContractDeploymentInfo } from '@aztec/circuits.js';
import { AztecRPC, PrivateKey } from '@aztec/types';

import { AccountWallet, ContractDeployer, WaitOpts, Wallet, generatePublicKey } from '../index.js';
import { CompleteAddress, isCompleteAddress } from './complete_address.js';
import { DeployAccountSentTx } from './deploy_account_sent_tx.js';
import { AccountContract, Salt } from './index.js';

/**
 * Manages a user account. Provides methods for calculating the account's address, deploying the account contract,
 * and creating and registering the user wallet in the RPC server.
 */
export class Account {
  /** Deployment salt for the account contract. */
  public readonly salt?: Fr;

  private completeAddress?: CompleteAddress;
  private encryptionPublicKey?: PublicKey;

  constructor(
    private rpc: AztecRPC,
    private encryptionPrivateKey: PrivateKey,
    private accountContract: AccountContract,
    saltOrAddress?: Salt | CompleteAddress,
  ) {
    if (isCompleteAddress(saltOrAddress)) {
      this.completeAddress = saltOrAddress;
    } else {
      this.salt = saltOrAddress ? new Fr(saltOrAddress) : Fr.random();
    }
  }

  protected async getEncryptionPublicKey() {
    if (!this.encryptionPublicKey) {
      this.encryptionPublicKey = await generatePublicKey(this.encryptionPrivateKey);
    }
    return this.encryptionPublicKey;
  }

  /**
   * Gets the calculated complete address associated with this account.
   * Does not require the account to be deployed or registered.
   * @returns The address, partial address, and encryption public key.
   */
  public async getCompleteAddress(): Promise<CompleteAddress> {
    if (!this.completeAddress) {
      const encryptionPublicKey = await generatePublicKey(this.encryptionPrivateKey);
      this.completeAddress = await getContractDeploymentInfo(
        this.accountContract.getContractAbi(),
        await this.accountContract.getDeploymentArgs(),
        this.salt!,
        encryptionPublicKey,
      );
    }
    return this.completeAddress;
  }

  /**
   * Returns a Wallet instance associated with this account. Use it to create Contract
   * instances to be interacted with from this account.
   * @returns A Wallet instance.
   */
  public async getWallet(): Promise<Wallet> {
    const nodeInfo = await this.rpc.getNodeInfo();
    const completeAddress = await this.getCompleteAddress();
    const account = await this.accountContract.getEntrypoint(completeAddress, nodeInfo);
    return new AccountWallet(this.rpc, account);
  }

  /**
   * Registers this account in the RPC server and returns the associated wallet. Registering
   * the account on the RPC server is required for managing private state associated with it.
   * Use the returned wallet to create Contract instances to be interacted with from this account.
   * @returns A Wallet instance.
   */
  public async register(): Promise<Wallet> {
    const { address, partialAddress } = await this.getCompleteAddress();
    await this.rpc.addAccount(this.encryptionPrivateKey, address, partialAddress);
    return this.getWallet();
  }

  /**
   * Deploys the account contract that backs this account.
   * Uses the salt provided in the constructor or a randomly generated one.
   * Note that if the Account is constructed with an explicit complete address
   * it is assumed that the account contract has already been deployed and this method will throw.
   * Registers the account in the RPC server before deploying the contract.
   * @returns A SentTx object that can be waited to get the associated Wallet.
   */
  public async deploy(): Promise<DeployAccountSentTx> {
    if (!this.salt) throw new Error(`Cannot deploy account contract without known salt.`);
    const wallet = await this.register();
    const encryptionPublicKey = await this.getEncryptionPublicKey();
    const deployer = new ContractDeployer(this.accountContract.getContractAbi(), this.rpc, encryptionPublicKey);
    const args = await this.accountContract.getDeploymentArgs();
    const sentTx = deployer.deploy(...args).send({ contractAddressSalt: this.salt });
    return new DeployAccountSentTx(wallet, sentTx.getTxHash());
  }

  /**
   * Deploys the account contract that backs this account and awaits the tx to be mined.
   * Uses the salt provided in the constructor or a randomly generated one.
   * Note that if the Account is constructed with an explicit complete address
   * it is assumed that the account contract has already been deployed and this method will throw.
   * Registers the account in the RPC server before deploying the contract.
   * @param opts - Options to wait for the tx to be mined.
   * @returns A Wallet instance.
   */
  public async waitDeploy(opts: WaitOpts): Promise<Wallet> {
    return (await this.deploy()).getWallet(opts);
  }
}
