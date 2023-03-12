import { EthAccount } from '../eth_account/index.js';
import { EthAddress } from '../eth_address/index.js';
import { decryptFromKeyStoreJson, KeyStoreEncryptOptions, KeyStoreJson } from '../keystore/index.js';

export class EthWallet {
  public length = 0;
  public accounts: EthAccount[] = [];

  constructor(numberOfAccounts = 0) {
    this.create(numberOfAccounts);
  }

  public static fromMnemonic(mnemonic: string, numberOfAccounts: number) {
    const wallet = new EthWallet();
    for (let i = 0; i < numberOfAccounts; ++i) {
      const path = `m/44'/60'/0'/0/${i}`;
      wallet.add(EthAccount.fromMnemonicAndPath(mnemonic, path));
    }
    return wallet;
  }

  public static fromSeed(seed: Buffer, numberOfAccounts: number) {
    const wallet = new EthWallet();
    for (let i = 0; i < numberOfAccounts; ++i) {
      const path = `m/44'/60'/0'/0/${i}`;
      wallet.add(EthAccount.fromSeedAndPath(seed, path));
    }
    return wallet;
  }

  public static async fromKeystores(keyStores: KeyStoreJson[], password: string) {
    const wallet = new EthWallet();
    await wallet.decrypt(keyStores, password);
    return wallet;
  }

  public create(numberOfAccounts: number, entropy?: Buffer): EthAccount[] {
    for (let i = 0; i < numberOfAccounts; ++i) {
      this.add(EthAccount.create(entropy).privateKey);
    }
    return this.accounts;
  }

  public getAccount(addressOrIndex: EthAddress | number) {
    if (addressOrIndex instanceof EthAddress) {
      return this.accounts.find(a => a && a.address.equals(addressOrIndex));
    }
    return this.accounts[addressOrIndex];
  }

  public getAccountIndex(addressOrIndex: EthAddress | number) {
    if (addressOrIndex instanceof EthAddress) {
      return this.accounts.findIndex(a => a && a.address.equals(addressOrIndex));
    }
    return addressOrIndex;
  }

  public getAccountIndicies() {
    return Object.keys(this.accounts).map(key => +key);
  }

  public getAccountAddresses() {
    return this.accounts.map(account => account.address);
  }

  public add(accountOrKey: Buffer | EthAccount): EthAccount {
    const account = Buffer.isBuffer(accountOrKey) ? new EthAccount(accountOrKey) : accountOrKey;

    const existing = this.getAccount(account.address);
    if (existing) {
      return existing;
    }

    const index = this.findSafeIndex();
    this.accounts[index] = account;
    this.length++;

    return account;
  }

  public remove(addressOrIndex: number | EthAddress) {
    const index = this.getAccountIndex(addressOrIndex);

    if (index === -1) {
      return false;
    }

    delete this.accounts[index];
    this.length--;

    return true;
  }

  public clear() {
    this.accounts = [];
    this.length = 0;
  }

  public encrypt(password: string, options?: KeyStoreEncryptOptions) {
    return Promise.all(this.getAccountIndicies().map(index => this.accounts[index].toKeyStoreJson(password, options)));
  }

  public async decrypt(encryptedWallet: KeyStoreJson[], password: string) {
    const decrypted = await Promise.all(encryptedWallet.map(keystore => decryptFromKeyStoreJson(keystore, password)));
    decrypted.forEach(account => {
      if (!account) {
        throw new Error("Couldn't decrypt accounts. Password wrong?");
      }

      this.add(account);
    });

    return this.accounts;
  }

  private findSafeIndex(pointer = 0) {
    while (this.accounts[pointer]) {
      ++pointer;
    }
    return pointer;
  }
}
