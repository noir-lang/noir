import {
  EthereumProvider,
  ProviderConnectInfo,
  ProviderMessage,
  ProviderRpcError,
  RequestArguments,
} from './ethereum_provider.js';
import { EthAddress } from '@aztec/foundation';
import { EthWallet } from '../eth_wallet/index.js';
import { readFile } from 'fs/promises';
import { JsonRpcProvider } from './json_rpc_provider.js';
import { EthAccount } from '../eth_account/index.js';
import { EthTransaction, populateTransaction, signTransaction } from '../eth_transaction/index.js';
import { EthereumRpc } from '../eth_rpc/index.js';
import { getTypedDataHash } from '../eth_typed_data/index.js';

/**
 * Given an EIP1193 provider, wraps it, and provides the ability to add local accounts.
 */
export class WalletProvider implements EthereumProvider {
  constructor(private provider: EthereumProvider, private wallet = new EthWallet()) {}

  /**
   * Create a WalletProvider instance using the given Ethereum host and an optional EthWallet instance.
   * This function initializes a JsonRpcProvider with the provided Ethereum host and then
   * creates a WalletProvider using the initialized JsonRpcProvider and the provided wallet.
   * If no wallet is provided, a new EthWallet instance will be created.
   *
   * @param ethereumHost - The Ethereum host URL used to initialize the JsonRpcProvider.
   * @param wallet - (Optional) An EthWallet instance to be used as the local wallet.
   * @returns A WalletProvider instance with the initialized JsonRpcProvider and the given wallet.
   */
  public static fromHost(ethereumHost: string, wallet = new EthWallet()) {
    const provider = new JsonRpcProvider(ethereumHost);
    return new WalletProvider(provider, wallet);
  }

  /**
   * Adds an account to the wallet using a private key.
   * The input 'privateKey' should be a Buffer containing the private key bytes of the account.
   * Returns the corresponding EthAddress of the added account.
   *
   * @param privateKey - The Buffer containing the private key bytes of the account.
   * @returns An EthAddress instance.
   */
  public addAccount(privateKey: Buffer) {
    return this.wallet.add(privateKey).address;
  }

  /**
   * Add multiple accounts to the wallet using a mnemonic phrase and the specified number of accounts.
   * The accounts will be generated based on the provided BIP32 account index and indexed from 0 to (num-1).
   * This function is useful for importing a set of accounts that were derived from a single mnemonic.
   *
   * @param mnemonic - The mnemonic phrase used to generate the accounts.
   * @param num - The number of accounts to add to the wallet.
   * @param bip32Account - The BIP32 account index to derive the accounts from, default is 0.
   */
  public addAccountsFromMnemonic(mnemonic: string, num: number, bip32Account = 0) {
    for (let i = 0; i < num; ++i) {
      this.addAccountFromMnemonicAndPath(mnemonic, `m/44'/60'/${bip32Account}'/0/${i}`);
    }
  }

  /**
   * Adds an account to the wallet using a mnemonic and a specified BIP32 derivation path.
   * The generated account is derived from the given mnemonic using the provided path,
   * following the BIP32 hierarchical deterministic key generation standard.
   * Returns the Ethereum address of the added account.
   *
   * @param mnemonic - The seed phrase used to generate the private key for the account.
   * @param path - The BIP32 derivation path used to derive the account from the mnemonic.
   * @returns The Ethereum address of the added account.
   */
  public addAccountFromMnemonicAndPath(mnemonic: string, path: string) {
    return this.wallet.add(EthAccount.fromMnemonicAndPath(mnemonic, path)).address;
  }

  /**
   * Adds an account to the wallet provider by loading a keystore file and decrypting the private key using the provided password.
   * The keystore file should follow the JSON format defined in Ethereum Improvement Proposal 55 (EIP-55).
   * Throws an error if the input file is invalid or decryption fails due to incorrect password.
   *
   * @param file - The path to the keystore file containing the encrypted private key.
   * @param password - The password used for decryption of the keystore file (default: '').
   * @returns An EthAddress instance representing the address of the added account.
   */
  public async addAccountFromKeystore(file: string, password = '') {
    const json = JSON.parse(await readFile(file, { encoding: 'ascii' }));
    return this.wallet.add(await EthAccount.fromKeyStoreJson(json, password)).address;
  }

  /**
   * Retrieve all available accounts in the wallet.
   * This function returns an array of EthAddress instances corresponding to each account in the wallet.
   * If no accounts have been added, it returns an empty array.
   *
   * @returns An array of EthAddress instances representing the accounts in the wallet.
   */
  public getAccounts() {
    return this.wallet.getAccountAddresses();
  }

  /**
   * Retrieve the EthAddress instance of an account at a specific index within the wallet.
   * Returns `undefined` if the provided index is out of range or does not correspond to an existing account.
   *
   * @param account - The index (integer) of the account to be fetched from the wallet.
   * @returns The EthAddress instance corresponding to the account, or `undefined` if not found.
   */
  public getAccount(account: number) {
    return this.wallet.getAccount(account)?.address;
  }

  /**
   * Retrieve the private key associated with the specified account index.
   * Returns the private key as a Buffer if the account exists in the wallet, otherwise returns undefined.
   *
   * @param account - The index of the account whose private key is to be retrieved.
   * @returns The private key as a Buffer, or undefined if the account does not exist.
   */
  public getPrivateKey(account: number) {
    return this.wallet.getAccount(account)?.privateKey;
  }

  /**
   * Retrieves the private key associated with the given Ethereum address.
   * Returns the private key as a Buffer if found within the wallet, or undefined otherwise.
   *
   * @param account - The EthAddress instance representing the Ethereum address to lookup.
   * @returns The private key as a Buffer or undefined if not found in the wallet.
   */
  public getPrivateKeyForAddress(account: EthAddress) {
    return this.wallet.getAccount(account)?.privateKey;
  }

  /**
   * Handles the processing of various Ethereum JSON-RPC requests, delegating them to the appropriate internal methods.
   * If a local account is available for signing transactions or messages, it will be used, otherwise the request
   * is forwarded to the underlying provider for further processing. This allows adding and managing local accounts
   * while still interacting with external providers such as remote nodes or browser-based wallets like MetaMask.
   *
   * @param args - The RequestArguments object containing the method to be called and any necessary parameters.
   * @returns A Promise resolving to the result of the requested operation or an error if the operation fails.
   */
  public async request(args: RequestArguments): Promise<any> {
    switch (args.method) {
      case 'eth_accounts':
        return this.wallet.length ? this.getAccounts().map(a => a.toString()) : await this.provider.request(args);
      case 'eth_sign':
        return await this.ethSign(args);
      case 'personal_sign':
        return await this.personalSign(args);
      case 'eth_signTypedData_v4':
        return this.signTypedData(args);
      case 'eth_signTransaction':
        return this.ethSignTransaction(args);
      case 'eth_sendTransaction':
        return this.ethSendTransaction(args);
      default: {
        return await this.provider.request(args);
      }
    }
  }

  /**
   * The message will be prefixed and hashed, and the hash is signed.
   * @returns Promise.
   */
  private async ethSign(args: RequestArguments) {
    const [from, message] = args.params!;
    const account = this.wallet.getAccount(EthAddress.fromString(from));
    if (account) {
      const signature = account.signMessage(Buffer.from(message.slice(2), 'hex'));
      return signature.toString();
    }
    return await this.provider.request(args);
  }

  /**
   * Personal_sign is the same as eth_sign but with args reversed.
   * This is favoured as it has better client support r.e. Displaying the message to the user before signing.
   * @returns Promise | string.
   */
  private async personalSign(args: RequestArguments) {
    const [message, from] = args.params!;
    const account = this.wallet.getAccount(EthAddress.fromString(from));
    if (account) {
      const signature = account.signMessage(Buffer.from(message.slice(2), 'hex'));
      return signature.toString();
    }
    return await this.provider.request(args);
  }

  /**
   * Sign the provided typed data using an account's private key.
   * This method is used for EIP-712 compliant signing of structured data.
   * The signed digest can be used to verify the signer's identity and authenticity of the data.
   *
   * @param args - RequestArguments object containing the method name and input parameters.
   * @returns A Promise that resolves to a string representing the signature of the typed data.
   * @throws An error if the specified account is not found in the wallet or if the request to the provider fails.
   */
  private async signTypedData(args: RequestArguments) {
    const [from, data] = args.params!;
    const account = this.wallet.getAccount(EthAddress.fromString(from));
    if (account) {
      const digest = getTypedDataHash(typeof data === 'string' ? JSON.parse(data) : data);
      return account.signDigest(digest).toString();
    }
    return await this.provider.request(args);
  }

  /**
   * Given a tx in Eth Json Rpc format, create an EthTransaction, populate any missing fields, and sign.
   * @returns Buffer.
   */
  private async signTxLocally(tx: any, account: EthAccount) {
    const { chainId, to, gas, maxFeePerGas, maxPriorityFeePerGas, value, data, nonce } = tx;

    const txReq: Partial<EthTransaction> = {
      chainId: chainId !== undefined ? Number(chainId) : undefined,
      to: to !== undefined ? EthAddress.fromString(to) : undefined,
      gas: gas !== undefined ? Number(gas) : undefined,
      maxFeePerGas: maxFeePerGas !== undefined ? BigInt(maxFeePerGas) : undefined,
      maxPriorityFeePerGas: maxPriorityFeePerGas !== undefined ? BigInt(maxPriorityFeePerGas) : undefined,
      value: value !== undefined ? BigInt(value) : undefined,
      data: data !== undefined ? Buffer.from(data.slice(2), 'hex') : undefined,
      nonce: nonce !== undefined ? Number(nonce) : undefined,
    };
    const rpc = new EthereumRpc(this.provider);
    const populatedTx = await populateTransaction(txReq, account.privateKey, rpc);

    return signTransaction(populatedTx, account.privateKey).rawTransaction;
  }

  /**
   * Sign a transaction using the local account associated with the 'from' address in the given transaction object.
   * If the 'from' address is not managed by the WalletProvider, the transaction will be forwarded to the underlying provider.
   * The input transaction should be in Ethereum JSON-RPC format.
   *
   * @param args - A RequestArguments object containing the transaction details.
   * @returns A Promise that resolves to the signed transaction as a hex-encoded string.
   */
  private async ethSignTransaction(args: RequestArguments) {
    const tx = args.params![0];
    const account = this.wallet.getAccount(EthAddress.fromString(tx.from));
    if (account) {
      const result = await this.signTxLocally(tx, account);
      return '0x' + result.toString('hex');
    }
    return await this.provider.request(args);
  }

  /**
   * Process and send a given Ethereum transaction using the EthAccount instance associated with the 'from' address.
   * If the account is found in the local wallet, it will sign the transaction locally and send the raw transaction
   * to the Ethereum provider. If the account is not found in the local wallet, it will forward the request to the provider.
   *
   * @param args - The RequestArguments object containing the Ethereum transaction details.
   * @returns A promise that resolves to the transaction hash of the submitted transaction.
   */
  private async ethSendTransaction(args: RequestArguments) {
    const tx = args.params![0];
    const account = this.wallet.getAccount(EthAddress.fromString(tx.from));
    if (account) {
      const result = await this.signTxLocally(tx, account);
      return this.provider.request({
        method: 'eth_sendRawTransaction',
        params: ['0x' + result.toString('hex')],
      });
    }
    return this.provider.request(args);
  }

  /**
   * Attaches a callback function to the specified event type on the Ethereum provider.
   * The listener will be invoked whenever the event occurs for the given notification.
   * Common events include 'connect', 'disconnect', 'chainChanged', 'accountsChanged', and 'message'.
   *
   * @param notification - The event type to listen for.
   * @param listener - The callback function to be invoked when the event occurs.
   * @returns The WalletProvider instance, allowing for method chaining.
   */
  on(notification: 'connect', listener: (connectInfo: ProviderConnectInfo) => void): this;
  /**
   * Registers an event listener for the specified notification type.
   * The listener will be invoked with the relevant data when the provider emits that notification.
   *
   * @param notification - The notification type to subscribe to, such as 'connect', 'disconnect', 'chainChanged', 'accountsChanged', or 'message'.
   * @param listener - The callback function to be invoked when the provider emits the specified notification.
   * @returns This WalletProvider instance, allowing for chained method calls.
   */
  on(notification: 'disconnect', listener: (error: ProviderRpcError) => void): this;
  /**
   * Registers an event listener for the specified notification type.
   * The listener function will be invoked when the corresponding event is emitted by the provider.
   * This allows for handling of various events such as connection changes, account changes, etc.
   *
   * @param notification - The type of event to listen for ('connect', 'disconnect', 'chainChanged', 'accountsChanged', or 'message').
   * @param listener - The callback function to be invoked when the specified event occurs.
   * @returns The WalletProvider instance for chaining purposes.
   */
  on(notification: 'chainChanged', listener: (chainId: string) => void): this;
  /**
   * Add an event listener for the specified notification type.
   * The listener function will be called whenever an event of the specified type is emitted from the provider.
   * Supported notification types are: 'connect', 'disconnect', 'chainChanged', 'accountsChanged', and 'message'.
   *
   * @param notification - The type of event to listen for.
   * @param listener - The function to be called when the event occurs.
   * @returns The WalletProvider instance, allowing chained calls.
   */
  on(notification: 'accountsChanged', listener: (accounts: string[]) => void): this;
  /**
   * Add an event listener for the specified notification on this WalletProvider instance.
   * The listener will be called with relevant information when the event occurs.
   * Supported notifications include 'connect', 'disconnect', 'chainChanged', 'accountsChanged', and 'message'.
   *
   * @param notification - The type of event to listen for.
   * @param listener - The function that will be called when the event occurs, with arguments based on the event type.
   * @returns This WalletProvider instance, allowing for method chaining.
   */
  on(notification: 'message', listener: (message: ProviderMessage) => void): this;
  /**
   * Registers a listener function to be called when the specified event occurs.
   * The available events are 'connect', 'disconnect', 'chainChanged', 'accountsChanged', and 'message'.
   * The listener function should take the appropriate argument based on the event type.
   *
   * @param notification - The event type to listen for. One of: 'connect', 'disconnect', 'chainChanged', 'accountsChanged', or 'message'.
   * @param listener - The function to be called when the specified event occurs, taking the appropriate argument based on the event type.
   * @returns The WalletProvider instance, allowing for method chaining.
   */
  on(notification: any, listener: any) {
    return this.provider.on(notification, listener);
  }

  /**
   * Removes an event listener for the specified notification type.
   * The listener should be a function previously added using the `on` method.
   * If the listener is not found, this method does nothing.
   *
   * @param notification - The notification type to remove the listener from.
   * @param listener - The event listener function to remove.
   * @returns The WalletProvider instance.
   */
  removeListener(notification: 'connect', listener: (connectInfo: ProviderConnectInfo) => void): this;
  /**
   * Removes a specified listener function from the given event notification.
   * The listener function will no longer be triggered when the specified event occurs.
   *
   * @param notification - The event notification type for which the listener needs to be removed.
   * @param listener - The listener function that was previously added and needs to be removed.
   * @returns The modified WalletProvider instance.
   */
  removeListener(notification: 'disconnect', listener: (error: ProviderRpcError) => void): this;
  /**
   * Removes a previously added listener function for the specified event notification.
   * The function will no longer be invoked when the specified event is emitted.
   *
   * @param notification - The event notification from which to remove the listener.
   * @param listener - The listener function that was previously added and now needs to be removed.
   * @returns The WalletProvider instance for chaining.
   */
  removeListener(notification: 'chainChanged', listener: (chainId: string) => void): this;
  /**
   * Removes a previously added listener function for the specified notification event.
   * The listener function will no longer be called when the corresponding event occurs.
   * This helps to prevent unwanted side-effects, memory leaks and improve performance
   * by unregistering listeners that are no longer needed.
   *
   * @param notification - The event name for which the listener should be removed.
   * @param listener - The callback function that was previously added as a listener for the specified event.
   * @returns The WalletProvider instance, allowing for method chaining.
   */
  removeListener(notification: 'accountsChanged', listener: (accounts: string[]) => void): this;
  /**
   * Removes a previously added event listener for the specified notification type.
   * The listener function should be the same as the one used when calling 'on' to add the listener.
   * If the listener is not found, this method will have no effect.
   *
   * @param notification - The notification type for which to remove the listener.
   * @param listener - The listener function that was previously added.
   * @returns The WalletProvider instance with the listener removed.
   */
  removeListener(notification: 'message', listener: (message: ProviderMessage) => void): this;
  /**
   * Removes a specified listener function from the given event notification.
   * Listeners are functions that have been previously added via the `on` method.
   * If the listener is successfully removed, it will no longer be called when the corresponding event is triggered.
   *
   * @param notification - The event notification type from which the listener should be removed.
   * @param listener - The listener function to be removed from the specified event notification.
   * @returns The WalletProvider instance with the updated set of listeners.
   */
  removeListener(notification: any, listener: any) {
    return this.provider.removeListener(notification, listener);
  }
}
