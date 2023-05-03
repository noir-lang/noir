import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { AztecAddress, TxRequest } from '@aztec/circuits.js';
import { ConstantKeyPair, KeyPair } from './key_pair.js';
import { KeyStore } from './key_store.js';
import { Point } from '@aztec/foundation/fields';

export class TestKeyStore implements KeyStore {
  private accounts: KeyPair[] = [];

  constructor(private grumpkin: Grumpkin) {}

  public addAccount() {
    const keyPair = ConstantKeyPair.random(this.grumpkin);
    this.accounts.push(keyPair);
    return Promise.resolve(keyPair.getPublicKey().toAddress());
  }

  public getAccounts() {
    return Promise.resolve(this.accounts.map(a => a.getPublicKey().toAddress()));
  }

  public getAccountPrivateKey(address: AztecAddress): Promise<Buffer> {
    const account = this.getAccount(address);
    return account.getPrivateKey();
  }

  public getAccountPublicKey(address: AztecAddress): Promise<Point> {
    const account = this.getAccount(address);
    return Promise.resolve(account.getPublicKey());
  }

  public getSigningPublicKeys() {
    return Promise.resolve(this.accounts.map(a => a.getPublicKey()));
  }

  public signTxRequest(txRequest: TxRequest) {
    const account = this.getAccount(txRequest.from);
    // TODO - Define signing data.
    const signingData = txRequest.toBuffer();
    return account.signMessage(signingData);
  }

  private getAccount(address: AztecAddress) {
    const account = this.accounts.find(a => a.getPublicKey().toAddress().equals(address));
    if (!account) {
      throw new Error('Unknown account.');
    }
    return account;
  }
}
