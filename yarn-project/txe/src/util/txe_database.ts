import { type AztecAddress, CompleteAddress } from '@aztec/circuits.js';
import { type AztecKVStore, type AztecMap } from '@aztec/kv-store';
import { KVPxeDatabase } from '@aztec/pxe';

export class TXEDatabase extends KVPxeDatabase {
  #accounts: AztecMap<string, Buffer>;

  constructor(db: AztecKVStore) {
    super(db);
    this.#accounts = db.openMap('accounts');
  }

  getAccount(key: AztecAddress) {
    const completeAddress = this.#accounts.get(key.toString());
    if (!completeAddress) {
      throw new Error(`Account not found: ${key.toString()}`);
    }
    return CompleteAddress.fromBuffer(completeAddress);
  }

  async setAccount(key: AztecAddress, value: CompleteAddress) {
    await this.#accounts.set(key.toString(), value.toBuffer());
  }
}
