import { EthAccount } from '../eth_account/index.js';
import { EthAddress } from '@aztec/foundation';
import { decryptFromKeyStoreJson, KeyStoreEncryptOptions, KeyStoreJson } from '../keystore/index.js';

/**
 * The EthWallet class represents an Ethereum wallet consisting of multiple Ethereum accounts.
 * It provides methods for creating and managing accounts, importing wallets from mnemonics, seeds, and keystores,
 * as well as encrypting and decrypting wallets. The class supports account addition, removal, retrieval,
 * and management of the wallet's length and index pointers.
 */
export class EthWallet {
  /**
   * The total number of accounts stored in the wallet.
   */
  public length = 0;
  /**
   * Array containing Ethereum accounts in the wallet.
   */
  public accounts: EthAccount[] = [];

  constructor(numberOfAccounts = 0) {
    this.create(numberOfAccounts);
  }

  /**
   * Create an EthWallet instance from a mnemonic string for the specified number of accounts.
   * The mnemonic should be a BIP-39 compliant seed phrase containing a series of words, used for generating deterministic keys.
   * This function generates EthAccounts based on the given mnemonic and adds them to the wallet.
   *
   * @param mnemonic - The BIP-39 compliant seed phrase as a string.
   * @param numberOfAccounts - The number of accounts to generate and add to the wallet.
   * @returns An EthWallet instance containing the generated accounts.
   */
  public static fromMnemonic(mnemonic: string, numberOfAccounts: number) {
    const wallet = new EthWallet();
    for (let i = 0; i < numberOfAccounts; ++i) {
      const path = `m/44'/60'/0'/0/${i}`;
      wallet.add(EthAccount.fromMnemonicAndPath(mnemonic, path));
    }
    return wallet;
  }

  /**
   * Create an EthWallet instance from a provided seed Buffer and number of accounts.
   * The function generates the specified number of EthAccounts using the seed and
   * BIP44 derivation path, then adds them to the newly created EthWallet.
   *
   * @param seed - A Buffer containing the seed for generating the HD wallet.
   * @param numberOfAccounts - The number of EthAccounts to generate using the seed.
   * @returns An EthWallet instance containing the generated EthAccounts.
   */
  public static fromSeed(seed: Buffer, numberOfAccounts: number) {
    const wallet = new EthWallet();
    for (let i = 0; i < numberOfAccounts; ++i) {
      const path = `m/44'/60'/0'/0/${i}`;
      wallet.add(EthAccount.fromSeedAndPath(seed, path));
    }
    return wallet;
  }

  /**
   * Create an EthWallet instance from an array of KeyStoreJson objects.
   * Decrypts each keystore using the provided password, adds the accounts to the wallet,
   * and returns the wallet with decrypted accounts. Throws an error if decryption fails
   * due to an incorrect password or other issues.
   *
   * @param keyStores - An array of KeyStoreJson objects representing encrypted Ethereum accounts.
   * @param password - The password used for decrypting the keystores.
   * @returns A Promise that resolves to an EthWallet instance with decrypted accounts.
   */
  public static async fromKeystores(keyStores: KeyStoreJson[], password: string) {
    const wallet = new EthWallet();
    await wallet.decrypt(keyStores, password);
    return wallet;
  }

  /**
   * Create a specified number of Ethereum accounts and add them to the wallet.
   * Generates new Ethereum accounts using an optional entropy buffer for randomness.
   * Returns an array of the created EthAccount instances.
   *
   * @param numberOfAccounts - The number of accounts to create.
   * @param entropy - Optional buffer containing entropy bytes for creating accounts.
   * @returns An array of created EthAccount instances.
   */
  public create(numberOfAccounts: number, entropy?: Buffer): EthAccount[] {
    for (let i = 0; i < numberOfAccounts; ++i) {
      this.add(EthAccount.create(entropy).privateKey);
    }
    return this.accounts;
  }

  /**
   * Retrieve an EthAccount instance from the wallet using either an Ethereum address or a numeric index.
   * The function searches for an account based on the provided input and returns it if found.
   * If multiple accounts are present, use the address or index to specify a unique account.
   *
   * @param addressOrIndex - An EthAddress instance or a number representing the account's index in the wallet.
   * @returns The EthAccount instance corresponding to the provided address or index, or undefined if not found.
   */
  public getAccount(addressOrIndex: EthAddress | number) {
    if (addressOrIndex instanceof EthAddress) {
      return this.accounts.find(a => a && a.address.equals(addressOrIndex));
    }
    return this.accounts[addressOrIndex];
  }

  /**
   * Retrieve the index of an account in the wallet based on the provided address or index.
   * If the input is an EthAddress, this function searches for an account with a matching address and returns its index.
   * If the input is a number, it directly returns the input number as the index. Returns -1 if no matching account is found.
   *
   * @param addressOrIndex - An EthAddress object representing the Ethereum address or a number representing the account index.
   * @returns The index of the account within the wallet or -1 if not found.
   */
  public getAccountIndex(addressOrIndex: EthAddress | number) {
    if (addressOrIndex instanceof EthAddress) {
      return this.accounts.findIndex(a => a && a.address.equals(addressOrIndex));
    }
    return addressOrIndex;
  }

  /**
   * Get an array of the indices of all EthAccounts stored in the wallet.
   * The returned indices can be used to access EthAccount instances through the 'getAccount' function.
   *
   * @returns An array of integers representing the indices of the EthAccounts in the wallet.
   */
  public getAccountIndicies() {
    return Object.keys(this.accounts).map(key => +key);
  }

  /**
   * Retrieve the Ethereum addresses of all accounts in the wallet.
   * This function maps the accounts to their corresponding addresses
   * and returns an array of EthAddress instances.
   *
   * @returns An array of EthAddress instances representing the Ethereum addresses of the accounts in the wallet.
   */
  public getAccountAddresses() {
    return this.accounts.map(account => account.address);
  }

  /**
   * Add an EthAccount instance or a private key to the wallet.
   * If an account with the same address already exists in the wallet, it returns the existing account.
   * Otherwise, it adds the new account at a safe index and increments the wallet length.
   *
   * @param accountOrKey - An EthAccount instance or a Buffer containing the private key.
   * @returns The added or existing EthAccount instance.
   */
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

  /**
   * Removes an account from the wallet based on the provided address or index.
   * If the given address or index matches one of the existing accounts in the wallet, the account will be removed,
   * and the function returns true. If the address or index is not found in the wallet, the function returns false.
   *
   * @param addressOrIndex - The EthAddress or index number of the account to be removed from the wallet.
   * @returns A boolean value indicating whether the removal was successful.
   */
  public remove(addressOrIndex: number | EthAddress) {
    const index = this.getAccountIndex(addressOrIndex);

    if (index === -1) {
      return false;
    }

    delete this.accounts[index];
    this.length--;

    return true;
  }

  /**
   * Clears all the accounts stored in the EthWallet instance.
   * The length of EthWallet will be set to 0 and the accounts array will become empty.
   */
  public clear() {
    this.accounts = [];
    this.length = 0;
  }

  /**
   * Encrypts the account private keys in the wallet using the provided password and returns an array of KeyStoreJson objects.
   * The KeyStoreJson objects follow the Ethereum keystore format (UTC / JSON) standard and can be later used to decrypt the accounts.
   * The optional 'options' parameter allows customizing the encryption process, such as the number of iterations or salt.
   *
   * @param password - The user-defined password to use for encrypting the account private keys.
   * @param options - Optional KeyStoreEncryptOptions object for customizing the encryption process.
   * @returns A Promise that resolves to an array of encrypted KeyStoreJson objects.
   */
  public encrypt(password: string, options?: KeyStoreEncryptOptions) {
    return Promise.all(this.getAccountIndicies().map(index => this.accounts[index].toKeyStoreJson(password, options)));
  }

  /**
   * Decrypts an array of KeyStoreJson objects using the provided password and adds the decrypted accounts to the wallet.
   * If any of the accounts cannot be decrypted, it will throw an error with a message indicating that the password might be wrong.
   *
   * @param encryptedWallet - Array of KeyStoreJson objects representing the encrypted wallet.
   * @param password - The password used to decrypt the encrypted wallet.
   * @returns An array of EthAccount instances stored in the wallet after successful decryption.
   */
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

  /**
   * Find an available index to safely add a new account in the accounts array.
   * The method iterates through the accounts array, incrementing the pointer until it finds an empty position.
   *
   * @param pointer - Optional starting index for the search. Default value is 0.
   * @returns The index of the first empty position in the accounts array.
   */
  private findSafeIndex(pointer = 0) {
    while (this.accounts[pointer]) {
      ++pointer;
    }
    return pointer;
  }
}
