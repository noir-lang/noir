import { Fr, PublicKey, getContractDeploymentInfo } from '@aztec/circuits.js';
import { AztecRPC, CompleteAddress, PrivateKey } from '@aztec/types';

import { AccountWallet, ContractDeployer, DeployMethod, WaitOpts, generatePublicKey } from '../index.js';
import { DeployAccountSentTx } from './deploy_account_sent_tx.js';
import { AccountContract, Entrypoint, Salt } from './index.js';

/**
 * Manages a user account. Provides methods for calculating the account's address, deploying the account contract,
 * and creating and registering the user wallet in the RPC server.
 */
export class Account {
  /** Deployment salt for the account contract. */
  public readonly salt?: Fr;

  private completeAddress?: CompleteAddress;
  private encryptionPublicKey?: PublicKey;
  private deployMethod?: DeployMethod;

  constructor(
    private rpc: AztecRPC,
    private encryptionPrivateKey: PrivateKey,
    private accountContract: AccountContract,
    saltOrAddress?: Salt | CompleteAddress,
  ) {
    if (saltOrAddress instanceof CompleteAddress) {
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
   * Returns the entrypoint for this account as defined by its account contract.
   * @returns An entrypoint.
   */
  public async getEntrypoint(): Promise<Entrypoint> {
    const nodeInfo = await this.rpc.getNodeInfo();
    const completeAddress = await this.getCompleteAddress();
    return this.accountContract.getEntrypoint(completeAddress, nodeInfo);
  }

  /**
   * Gets the calculated complete address associated with this account.
   * Does not require the account to be deployed or registered.
   * @returns The address, partial address, and encryption public key.
   */
  public async getCompleteAddress(): Promise<CompleteAddress> {
    if (!this.completeAddress) {
      const encryptionPublicKey = await generatePublicKey(this.encryptionPrivateKey);
      const contractDeploymentInfo = await getContractDeploymentInfo(
        this.accountContract.getContractAbi(),
        await this.accountContract.getDeploymentArgs(),
        this.salt!,
        encryptionPublicKey,
      );
      this.completeAddress = contractDeploymentInfo.completeAddress;
    }
    return this.completeAddress;
  }

  /**
   * Returns a Wallet instance associated with this account. Use it to create Contract
   * instances to be interacted with from this account.
   * @returns A Wallet instance.
   */
  public async getWallet(): Promise<AccountWallet> {
    const entrypoint = await this.getEntrypoint();
    return new AccountWallet(this.rpc, entrypoint, await this.getCompleteAddress());
  }

  /**
   * Registers this account in the RPC server and returns the associated wallet. Registering
   * the account on the RPC server is required for managing private state associated with it.
   * Use the returned wallet to create Contract instances to be interacted with from this account.
   * @returns A Wallet instance.
   */
  public async register(): Promise<AccountWallet> {
    const completeAddress = await this.getCompleteAddress();
    await this.rpc.registerAccount(this.encryptionPrivateKey, completeAddress);
    return this.getWallet();
  }

  /**
   * Returns the pre-populated deployment method to deploy the account contract that backs this account.
   * Typically you will not need this method and can call `deploy` directly. Use this for having finer
   * grained control on when to create, simulate, and send the deployment tx.
   * @returns A DeployMethod instance that deploys this account contract.
   */
  public async getDeployMethod() {
    if (!this.deployMethod) {
      if (!this.salt) throw new Error(`Cannot deploy account contract without known salt.`);
      await this.register();
      const encryptionPublicKey = await this.getEncryptionPublicKey();
      const deployer = new ContractDeployer(this.accountContract.getContractAbi(), this.rpc, encryptionPublicKey);
      const args = await this.accountContract.getDeploymentArgs();
      this.deployMethod = deployer.deploy(...args);
    }
    return this.deployMethod;
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
    const deployMethod = await this.getDeployMethod();
    const wallet = await this.getWallet();
    const sentTx = deployMethod.send({ contractAddressSalt: this.salt });
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
  public async waitDeploy(opts: WaitOpts = {}): Promise<AccountWallet> {
    await this.deploy().then(tx => tx.wait(opts));
    return this.getWallet();
  }
}
