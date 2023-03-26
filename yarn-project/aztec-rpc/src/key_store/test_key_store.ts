import { TxRequest } from '../circuits.js';
import { ConstantKeyPair, KeyPair } from './key_pair.js';
import { KeyStore } from './key_store.js';

export class TestKeyStore implements KeyStore {
  private accounts: KeyPair[] = [];

  constructor() {
    this.accounts.push(ConstantKeyPair.random());
  }

  public addAccount() {
    const keyPair = ConstantKeyPair.random();
    this.accounts.push(keyPair);
    return Promise.resolve(keyPair.getPublicKey());
  }

  getAccounts() {
    return Promise.resolve(this.accounts.map(a => a.getPublicKey()));
  }

  getSigningPublicKeys() {
    return this.getAccounts();
  }

  signTxRequest(txRequest: TxRequest) {
    const account = this.accounts.find(a => a.getPublicKey().equals(txRequest.from));
    if (!account) {
      throw new Error('Unknown account.');
    }

    return account.signMessage(txRequest.toBuffer());
  }
}
