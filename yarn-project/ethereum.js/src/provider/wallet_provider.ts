import {
  EthereumProvider,
  ProviderConnectInfo,
  ProviderMessage,
  ProviderRpcError,
  RequestArguments,
} from './ethereum_provider.js';
import { EthAddress } from '../eth_address/index.js';
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

  public static fromHost(ethereumHost: string, wallet = new EthWallet()) {
    const provider = new JsonRpcProvider(ethereumHost);
    return new WalletProvider(provider, wallet);
  }

  public addAccount(privateKey: Buffer) {
    return this.wallet.add(privateKey).address;
  }

  public addAccountsFromMnemonic(mnemonic: string, num: number, bip32Account = 0) {
    for (let i = 0; i < num; ++i) {
      this.addAccountFromMnemonicAndPath(mnemonic, `m/44'/60'/${bip32Account}'/0/${i}`);
    }
  }

  public addAccountFromMnemonicAndPath(mnemonic: string, path: string) {
    return this.wallet.add(EthAccount.fromMnemonicAndPath(mnemonic, path)).address;
  }

  public async addAccountFromKeystore(file: string, password = '') {
    const json = JSON.parse(await readFile(file, { encoding: 'ascii' }));
    return this.wallet.add(await EthAccount.fromKeyStoreJson(json, password)).address;
  }

  public getAccounts() {
    return this.wallet.getAccountAddresses();
  }

  public getAccount(account: number) {
    return this.wallet.getAccount(account)?.address;
  }

  public getPrivateKey(account: number) {
    return this.wallet.getAccount(account)?.privateKey;
  }

  public getPrivateKeyForAddress(account: EthAddress) {
    return this.wallet.getAccount(account)?.privateKey;
  }

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
   * personal_sign is the same as eth_sign but with args reversed.
   * This is favoured as it has better client support r.e. displaying the message to the user before signing.
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

  private async ethSignTransaction(args: RequestArguments) {
    const tx = args.params![0];
    const account = this.wallet.getAccount(EthAddress.fromString(tx.from));
    if (account) {
      const result = await this.signTxLocally(tx, account);
      return '0x' + result.toString('hex');
    }
    return await this.provider.request(args);
  }

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

  on(notification: 'connect', listener: (connectInfo: ProviderConnectInfo) => void): this;
  on(notification: 'disconnect', listener: (error: ProviderRpcError) => void): this;
  on(notification: 'chainChanged', listener: (chainId: string) => void): this;
  on(notification: 'accountsChanged', listener: (accounts: string[]) => void): this;
  on(notification: 'message', listener: (message: ProviderMessage) => void): this;
  on(notification: any, listener: any) {
    return this.provider.on(notification, listener);
  }

  removeListener(notification: 'connect', listener: (connectInfo: ProviderConnectInfo) => void): this;
  removeListener(notification: 'disconnect', listener: (error: ProviderRpcError) => void): this;
  removeListener(notification: 'chainChanged', listener: (chainId: string) => void): this;
  removeListener(notification: 'accountsChanged', listener: (accounts: string[]) => void): this;
  removeListener(notification: 'message', listener: (message: ProviderMessage) => void): this;
  removeListener(notification: any, listener: any) {
    return this.provider.removeListener(notification, listener);
  }
}
