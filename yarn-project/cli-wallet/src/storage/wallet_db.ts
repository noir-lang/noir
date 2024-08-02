import { AztecAddress, Fr } from '@aztec/circuits.js';
import { type AztecKVStore, type AztecMap } from '@aztec/kv-store';

export class WalletDB {
  #accounts!: AztecMap<string, Buffer>;

  private static instance: WalletDB;

  static getInstance() {
    if (!WalletDB.instance) {
      WalletDB.instance = new WalletDB();
    }

    return WalletDB.instance;
  }

  init(store: AztecKVStore) {
    this.#accounts = store.openMap('accounts');
  }

  async storeAccount(
    address: AztecAddress,
    { privateKey, salt, alias }: { privateKey: Fr; salt: Fr; alias: string | undefined },
  ) {
    if (alias) {
      await this.#accounts.set(`${alias}`, address.toBuffer());
    }
    await this.#accounts.set(`${address.toString()}-pk`, privateKey.toBuffer());
    await this.#accounts.set(`${address.toString()}-salt`, salt.toBuffer());
  }

  retrieveAccount(aliasOrAddress: AztecAddress | string) {
    const address =
      typeof aliasOrAddress === 'object'
        ? aliasOrAddress
        : AztecAddress.fromBuffer(this.#accounts.get(aliasOrAddress)!);
    const privateKeyBuffer = this.#accounts.get(`${address.toString()}-pk`);
    if (!privateKeyBuffer) {
      throw new Error(
        `Could not find ${address}-pk. Account "${aliasOrAddress.toString}" does not exist on this wallet.`,
      );
    }
    const privateKey = Fr.fromBuffer(privateKeyBuffer);
    const salt = Fr.fromBuffer(this.#accounts.get(`${address.toString()}-salt`)!);
    return { privateKey, salt };
  }
}
