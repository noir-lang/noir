import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { AztecAddress, TxRequest } from '@aztec/circuits.js';
import { ConstantKeyPair, KeyPair } from './key_pair.js';
import { KeyStore } from './key_store.js';

export class TestKeyStore implements KeyStore {
  private accounts: KeyPair[] = [];

  constructor(private grumpkin: Grumpkin) {}

  public addAccount() {
    const keyPair = ConstantKeyPair.random(this.grumpkin);
    this.accounts.push(keyPair);
    return Promise.resolve(keyPair.getPublicKey().toAddress());
  }

  getAccounts() {
    return Promise.resolve(this.accounts.map(a => a.getPublicKey().toAddress()));
  }

  getAccountPrivateKey(address: AztecAddress): Promise<Buffer> {
    const account = this.accounts.find(a => a.getPublicKey().toAddress().equals(address));
    if (!account) {
      throw new Error('Unknown account.');
    }

    return account.getPrivateKey();
  }

  getSigningPublicKeys() {
    return Promise.resolve(this.accounts.map(a => a.getPublicKey()));
  }

  signTxRequest(txRequest: TxRequest) {
    const account = txRequest.from.equals(AztecAddress.ZERO)
      ? this.accounts[0]
      : this.accounts.find(a => a.getPublicKey().toAddress().equals(txRequest.from));
    if (!account) {
      throw new Error('Unknown account.');
    }

    return account.signMessage(txRequest.toBuffer());
  }
}
