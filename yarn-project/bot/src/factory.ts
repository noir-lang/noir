import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { type AccountWallet, BatchCall, createDebugLogger, createPXEClient } from '@aztec/aztec.js';
import { type FunctionCall, type PXE } from '@aztec/circuit-types';
import { Fr, deriveSigningKey } from '@aztec/circuits.js';
import { TokenContract } from '@aztec/noir-contracts.js/Token';

import { type BotConfig } from './config.js';
import { getBalances } from './utils.js';

const MINT_BALANCE = 1e12;
const MIN_BALANCE = 1e3;

export class BotFactory {
  private pxe: PXE;
  private log = createDebugLogger('aztec:bot');

  constructor(private readonly config: BotConfig, dependencies: { pxe?: PXE } = {}) {
    if (!dependencies.pxe && !config.pxeUrl) {
      throw new Error(`Either a PXE client or a PXE URL must be provided`);
    }

    this.pxe = dependencies.pxe ?? createPXEClient(config.pxeUrl!);
  }

  /**
   * Initializes a new bot by setting up the sender account, registering the recipient,
   * deploying the token contract, and minting tokens if necessary.
   */
  public async setup() {
    const recipient = await this.registerRecipient();
    const wallet = await this.setupAccount();
    const token = await this.setupToken(wallet);
    await this.mintTokens(token);
    return { wallet, token, pxe: this.pxe, recipient };
  }

  /**
   * Checks if the sender account contract is initialized, and initializes it if necessary.
   * @returns The sender wallet.
   */
  private async setupAccount() {
    const salt = Fr.ONE;
    const signingKey = deriveSigningKey(this.config.senderPrivateKey);
    const account = getSchnorrAccount(this.pxe, this.config.senderPrivateKey, signingKey, salt);
    const isInit = await this.pxe.isContractInitialized(account.getAddress());
    if (isInit) {
      this.log.info(`Account at ${account.getAddress().toString()} already initialized`);
      return account.register();
    } else {
      this.log.info(`Initializing account at ${account.getAddress().toString()}`);
      return account.waitSetup();
    }
  }

  /**
   * Registers the recipient for txs in the pxe.
   */
  private async registerRecipient() {
    const recipient = await this.pxe.registerAccount(this.config.recipientEncryptionSecret, Fr.ONE);
    return recipient.address;
  }

  /**
   * Checks if the token contract is deployed and deploys it if necessary.
   * @param wallet - Wallet to deploy the token contract from.
   * @returns The TokenContract instance.
   */
  private async setupToken(wallet: AccountWallet): Promise<TokenContract> {
    const deploy = TokenContract.deploy(wallet, wallet.getAddress(), 'BotToken', 'BOT', 18);
    const deployOpts = { contractAddressSalt: this.config.tokenSalt, universalDeploy: true };
    const address = deploy.getInstance(deployOpts).address;
    if (await this.pxe.isContractPubliclyDeployed(address)) {
      this.log.info(`Token at ${address.toString()} already deployed`);
      return deploy.register();
    } else {
      this.log.info(`Deploying token contract at ${address.toString()}`);
      return deploy.send(deployOpts).deployed();
    }
  }

  /**
   * Mints private and public tokens for the sender if their balance is below the minimum.
   * @param token - Token contract.
   */
  private async mintTokens(token: TokenContract) {
    const sender = token.wallet.getAddress();
    const { privateBalance, publicBalance } = await getBalances(token, sender);
    const calls: FunctionCall[] = [];
    if (privateBalance < MIN_BALANCE) {
      this.log.info(`Minting private tokens for ${sender.toString()}`);
      calls.push(token.methods.privately_mint_private_note(MINT_BALANCE).request());
    }
    if (publicBalance < MIN_BALANCE) {
      this.log.info(`Minting public tokens for ${sender.toString()}`);
      calls.push(token.methods.mint_public(sender, MINT_BALANCE).request());
    }
    if (calls.length === 0) {
      this.log.info(`Skipping minting as ${sender.toString()} has enough tokens`);
      return;
    }
    await new BatchCall(token.wallet, calls).send().wait();
  }
}
