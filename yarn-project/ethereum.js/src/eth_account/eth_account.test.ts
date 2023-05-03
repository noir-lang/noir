import { EthAddress } from '@aztec/foundation/eth-address';
import { EthTransaction } from '../eth_transaction/index.js';
import { EthAccount } from './eth_account.js';

describe('eth_account', () => {
  it('should create account', () => {
    const account = EthAccount.create();
    expect(account).toBeInstanceOf(EthAccount);
  });

  it('should create account from private key', () => {
    const privateKey = Buffer.from('7a28b5ba57c53603b0b07b56bba752f7784bf506fa95edc395f5cf6c7514fe9d', 'hex');
    const account = new EthAccount(privateKey);
    expect(account.address.toChecksumString()).toBe('0x008AeEda4D805471dF9b2A5B0f38A0C3bCBA786b');
  });

  it('should create account from mnemonic and path', () => {
    const mnemonic = 'uncover parade truck rhythm cinnamon cattle polar luxury chest anchor cinnamon coil';
    const path = "m/44'/60'/0'/0/0";
    const account = EthAccount.fromMnemonicAndPath(mnemonic, path);
    expect(account.address.toChecksumString()).toBe('0xb897DF5d6c6D5b15E7340D7Ea2A8B8dC776B43F4');
    expect(account.privateKey.toString('hex')).toBe('dc21e91bcb468f2c2484f44f947f38625b441366f9afe82cda6f3d9de0135c3b');
  });

  it('should encrypt and decrypt account', async () => {
    const account = EthAccount.create();
    const keyStore = await account.toKeyStoreJson('password');
    const decrypted = await EthAccount.fromKeyStoreJson(keyStore, 'password');
    expect(decrypted.address).toEqual(account.address);
    expect(decrypted.privateKey).toEqual(account.privateKey);
  });

  it('should sign message', () => {
    const privateKey = Buffer.from('7a28b5ba57c53603b0b07b56bba752f7784bf506fa95edc395f5cf6c7514fe9d', 'hex');
    const account = new EthAccount(privateKey);
    const message = Buffer.from('data to sign');
    const signature = account.signMessage(message);
    expect(account.signedMessage(message, signature)).toBe(true);
  });

  it('should sign transaction', () => {
    const privateKey = Buffer.from('4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318', 'hex');
    const tx: EthTransaction = {
      chainId: 1,
      nonce: 0,
      maxFeePerGas: BigInt('20000000000'),
      maxPriorityFeePerGas: BigInt('10000000000'),
      gas: 21000,
      to: EthAddress.fromString('0xF0109fC8DF283027b6285cc889F5aA624EaC1F55'),
      value: BigInt('1000000000'),
    };
    const account = new EthAccount(privateKey);
    const signedTx = account.signTransaction(tx);
    expect(account.signedTransaction(tx, signedTx.signature)).toBe(true);
  });
});
