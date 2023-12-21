import { PublicKey, getContractDeploymentInfo } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { CompleteAddress, GrumpkinPrivateKey, PXE } from '@aztec/types';

import { Salt } from '../account/index.js';
import { AccountInterface } from '../account/interface.js';
import {
  AccountContract,
  EcdsaAccountContract,
  SchnorrAccountContract,
  SingleKeyAccountContract,
} from '../account_contract/index.js';
import { DefaultWaitOpts, DeployMethod, WaitOpts } from '../contract/index.js';
import { ContractDeployer } from '../contract_deployer/index.js';
import { generatePublicKey } from '../utils/index.js';
import { AccountWalletWithPrivateKey } from '../wallet/index.js';
import { DeployAccountSentTx } from './deploy_account_sent_tx.js';
import { waitForAccountSynch } from './util.js';

/**
 * Manages a user account. Provides methods for calculating the account's address, deploying the account contract,
 * and creating and registering the user wallet in the PXE Service.
 */
export class AccountManager {
  /** Deployment salt for the account contract. */
  public readonly salt?: Fr;

  private completeAddress?: CompleteAddress;
  private encryptionPublicKey?: PublicKey;
  private deployMethod?: DeployMethod;

  constructor(
    private pxe: PXE,
    private encryptionPrivateKey: GrumpkinPrivateKey,
    private accountContract: AccountContract,
    saltOrAddress?: Salt | CompleteAddress,
  ) {
    if (saltOrAddress instanceof CompleteAddress) {
      this.completeAddress = saltOrAddress;
    } else {
      this.salt = saltOrAddress ? new Fr(saltOrAddress) : Fr.random();
    }
  }

  protected getEncryptionPublicKey() {
    if (!this.encryptionPublicKey) {
      this.encryptionPublicKey = generatePublicKey(this.encryptionPrivateKey);
    }
    return this.encryptionPublicKey;
  }

  /**
   * Returns the entrypoint for this account as defined by its account contract.
   * @returns An entrypoint.
   */
  public async getAccount(): Promise<AccountInterface> {
    const nodeInfo = await this.pxe.getNodeInfo();
    const completeAddress = this.getCompleteAddress();
    return this.accountContract.getInterface(completeAddress, nodeInfo);
  }

  /**
   * Gets the calculated complete address associated with this account.
   * Does not require the account to be deployed or registered.
   * @returns The address, partial address, and encryption public key.
   */
  public getCompleteAddress(): CompleteAddress {
    if (!this.completeAddress) {
      const encryptionPublicKey = generatePublicKey(this.encryptionPrivateKey);
      const contractDeploymentInfo = getContractDeploymentInfo(
        this.accountContract.getContractArtifact(),
        this.accountContract.getDeploymentArgs(),
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
  public async getWallet(): Promise<AccountWalletWithPrivateKey> {
    const entrypoint = await this.getAccount();
    return new AccountWalletWithPrivateKey(this.pxe, entrypoint, this.encryptionPrivateKey);
  }

  /**
   * Registers this account in the PXE Service and returns the associated wallet. Registering
   * the account on the PXE Service is required for managing private state associated with it.
   * Use the returned wallet to create Contract instances to be interacted with from this account.
   * @param opts - Options to wait for the account to be synched.
   * @returns A Wallet instance.
   */
  public async register(opts: WaitOpts = DefaultWaitOpts): Promise<AccountWalletWithPrivateKey> {
    const address = await this.#register();
    await waitForAccountSynch(this.pxe, address, opts);
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
      if (!this.salt) {
        throw new Error(`Cannot deploy account contract without known salt.`);
      }
      await this.#register();
      const encryptionPublicKey = this.getEncryptionPublicKey();
      const deployer = new ContractDeployer(this.accountContract.getContractArtifact(), this.pxe, encryptionPublicKey);
      const args = this.accountContract.getDeploymentArgs();
      this.deployMethod = deployer.deploy(...args);
    }
    return this.deployMethod;
  }

  /**
   * Deploys the account contract that backs this account.
   * Uses the salt provided in the constructor or a randomly generated one.
   * Note that if the Account is constructed with an explicit complete address
   * it is assumed that the account contract has already been deployed and this method will throw.
   * Registers the account in the PXE Service before deploying the contract.
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
   * Registers the account in the PXE Service before deploying the contract.
   * @param opts - Options to wait for the tx to be mined.
   * @returns A Wallet instance.
   */
  public async waitDeploy(opts: WaitOpts = DefaultWaitOpts): Promise<AccountWalletWithPrivateKey> {
    await this.deploy().then(tx => tx.wait(opts));
    return this.getWallet();
  }

  async #register(): Promise<CompleteAddress> {
    const completeAddress = this.getCompleteAddress();
    await this.pxe.registerAccount(this.encryptionPrivateKey, completeAddress.partialAddress);
    return completeAddress;
  }
}

/**
 * Creates an Account that relies on an ECDSA signing key for authentication.
 * @param pxe - An PXE server instance.
 * @param encryptionPrivateKey - Grumpkin key used for note encryption.
 * @param signingPrivateKey - Secp256k1 key used for signing transactions.
 * @param saltOrAddress - Deployment salt or complete address if account contract is already deployed.
 */
export function getEcdsaAccount(
  pxe: PXE,
  encryptionPrivateKey: GrumpkinPrivateKey,
  signingPrivateKey: Buffer,
  saltOrAddress?: Salt | CompleteAddress,
): AccountManager {
  return new AccountManager(pxe, encryptionPrivateKey, new EcdsaAccountContract(signingPrivateKey), saltOrAddress);
}

/**
 * Creates an Account that relies on a Grumpkin signing key for authentication.
 * @param pxe - An PXE server instance.
 * @param encryptionPrivateKey - Grumpkin key used for note encryption.
 * @param signingPrivateKey - Grumpkin key used for signing transactions.
 * @param saltOrAddress - Deployment salt or complete address if account contract is already deployed.
 */
export function getSchnorrAccount(
  pxe: PXE,
  encryptionPrivateKey: GrumpkinPrivateKey,
  signingPrivateKey: GrumpkinPrivateKey,
  saltOrAddress?: Salt | CompleteAddress,
): AccountManager {
  return new AccountManager(pxe, encryptionPrivateKey, new SchnorrAccountContract(signingPrivateKey), saltOrAddress);
}

/**
 * Creates an Account that uses the same Grumpkin key for encryption and authentication.
 * @param pxe - An PXE server instance.
 * @param encryptionAndSigningPrivateKey - Grumpkin key used for note encryption and signing transactions.
 * @param saltOrAddress - Deployment salt or complete address if account contract is already deployed.
 */
export function getUnsafeSchnorrAccount(
  pxe: PXE,
  encryptionAndSigningPrivateKey: GrumpkinPrivateKey,
  saltOrAddress?: Salt | CompleteAddress,
): AccountManager {
  return new AccountManager(
    pxe,
    encryptionAndSigningPrivateKey,
    new SingleKeyAccountContract(encryptionAndSigningPrivateKey),
    saltOrAddress,
  );
}
