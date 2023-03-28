import { TxRequest } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { ConstantKeyPair, KeyPair } from './key_pair.js';
import { KeyStore } from './key_store.js';

export class TestKeyStore implements KeyStore {
  private accounts: KeyPair[] = [];

  constructor() {}

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
    const account = txRequest.from.equals(AztecAddress.ZERO)
      ? this.accounts[0]
      : this.accounts.find(a => a.getPublicKey().toBuffer().equals(txRequest.from.toBuffer()));
    if (!account) {
      throw new Error('Unknown account.');
    }

    return account.signMessage(txRequest.toBuffer());
  }
}
