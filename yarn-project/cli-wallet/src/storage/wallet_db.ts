import { type AztecAddress, Fr } from '@aztec/circuits.js';
import { type AztecKVStore, type AztecMap } from '@aztec/kv-store';

import { type AccountType } from '../utils/accounts.js';

export const Aliases = ['accounts', 'contracts', 'artifacts'] as const;
export type AliasType = (typeof Aliases)[number];

export class WalletDB {
  #accounts!: AztecMap<string, Buffer>;
  #aliases!: AztecMap<string, Buffer>;

  private static instance: WalletDB;

  static getInstance() {
    if (!WalletDB.instance) {
      WalletDB.instance = new WalletDB();
    }

    return WalletDB.instance;
  }

  init(store: AztecKVStore) {
    this.#accounts = store.openMap('accounts');
    this.#aliases = store.openMap('aliases');
  }

  async storeAccount(
    address: AztecAddress,
    { type, secretKey, salt, alias }: { type: AccountType; secretKey: Fr; salt: Fr; alias: string | undefined },
  ) {
    if (alias) {
      await this.#aliases.set(`accounts:${alias}`, Buffer.from(address.toString()));
    }
    await this.#accounts.set(`${address.toString()}-type`, Buffer.from(type));
    await this.#accounts.set(`${address.toString()}-sk`, secretKey.toBuffer());
    await this.#accounts.set(`${address.toString()}-salt`, salt.toBuffer());
    await this.#aliases.set('accounts:last', Buffer.from(address.toString()));
  }

  async storeContract(address: AztecAddress, artifactPath: string, alias?: string) {
    if (alias) {
      await this.#aliases.set(`contracts:${alias}`, Buffer.from(address.toString()));
      await this.#aliases.set(`artifacts:${alias}`, Buffer.from(artifactPath));
    }
    await this.#aliases.set(`contracts:last`, Buffer.from(address.toString()));
    await this.#aliases.set(`artifacts:last`, Buffer.from(artifactPath));
    await this.#aliases.set(`artifacts:${address.toString()}`, Buffer.from(artifactPath));
  }

  tryRetrieveAlias(arg: string) {
    if (Aliases.find(alias => arg.startsWith(`${alias}:`))) {
      const [type, alias] = arg.split(':');
      const data = this.#aliases.get(`${type}:${alias ?? 'last'}`);
      return data ? data.toString() : arg;
    }

    return arg;
  }

  async storeAccountMetadata(aliasOrAddress: AztecAddress | string, metadataKey: string, metadata: Buffer) {
    const { address } = this.retrieveAccount(aliasOrAddress);
    await this.#accounts.set(`${address.toString()}-${metadataKey}`, metadata);
  }

  retrieveAccountMetadata(aliasOrAddress: AztecAddress | string, metadataKey: string) {
    const { address } = this.retrieveAccount(aliasOrAddress);
    const result = this.#accounts.get(`${address.toString()}-${metadataKey}`);
    if (!result) {
      throw new Error(`Could not find metadata with key ${metadataKey} for account ${aliasOrAddress}`);
    }
    return result;
  }

  retrieveAccount(address: AztecAddress | string) {
    const secretKeyBuffer = this.#accounts.get(`${address.toString()}-sk`);
    if (!secretKeyBuffer) {
      throw new Error(`Could not find ${address}-sk. Account "${address.toString}" does not exist on this wallet.`);
    }
    const secretKey = Fr.fromBuffer(secretKeyBuffer);
    const salt = Fr.fromBuffer(this.#accounts.get(`${address.toString()}-salt`)!);
    const type = this.#accounts.get(`${address.toString()}-type`)!.toString('utf8') as AccountType;
    return { address, secretKey, salt, type };
  }
}
