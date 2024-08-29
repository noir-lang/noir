import { type AuthWitness, type TxHash } from '@aztec/circuit-types';
import { type AztecAddress, Fr, GasSettings } from '@aztec/circuits.js';
import { type LogFn } from '@aztec/foundation/log';
import { type AztecKVStore, type AztecMap } from '@aztec/kv-store';

import { type AccountType } from '../utils/accounts.js';
import { extractECDSAPublicKeyFromBase64String } from '../utils/ecdsa.js';

export const Aliases = ['accounts', 'contracts', 'artifacts', 'secrets', 'transactions', 'authwits'] as const;
export type AliasType = (typeof Aliases)[number];

export class WalletDB {
  #accounts!: AztecMap<string, Buffer>;
  #aliases!: AztecMap<string, Buffer>;
  #bridgedFeeJuice!: AztecMap<string, Buffer>;
  #transactions!: AztecMap<string, Buffer>;

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
    this.#bridgedFeeJuice = store.openMap('bridgedFeeJuice');
    this.#transactions = store.openMap('transactions');
  }

  async pushBridgedFeeJuice(recipient: AztecAddress, secret: Fr, amount: bigint, log: LogFn) {
    let stackPointer = this.#bridgedFeeJuice.get(`${recipient.toString()}:stackPointer`)?.readInt8() || 0;
    stackPointer++;
    await this.#bridgedFeeJuice.set(
      `${recipient.toString()}:${stackPointer}`,
      Buffer.from(`${amount.toString()}:${secret.toString()}`),
    );
    await this.#bridgedFeeJuice.set(`${recipient.toString()}:stackPointer`, Buffer.from([stackPointer]));
    log(`Pushed ${amount} fee juice for recipient ${recipient.toString()}. Stack pointer ${stackPointer}`);
  }

  async popBridgedFeeJuice(recipient: AztecAddress, log: LogFn) {
    let stackPointer = this.#bridgedFeeJuice.get(`${recipient.toString()}:stackPointer`)?.readInt8() || 0;
    const result = this.#bridgedFeeJuice.get(`${recipient.toString()}:${stackPointer}`);
    if (!result) {
      throw new Error(
        `No stored fee juice available for recipient ${recipient.toString()}. Please provide claim amount and secret. Stack pointer ${stackPointer}`,
      );
    }
    const [amountStr, secretStr] = result.toString().split(':');
    await this.#bridgedFeeJuice.set(`${recipient.toString()}:stackPointer`, Buffer.from([--stackPointer]));
    log(`Retrieved ${amountStr} fee juice for recipient ${recipient.toString()}. Stack pointer ${stackPointer}`);
    return { amount: BigInt(amountStr), secret: secretStr };
  }

  async storeAccount(
    address: AztecAddress,
    {
      type,
      secretKey,
      salt,
      alias,
      publicKey,
    }: { type: AccountType; secretKey: Fr; salt: Fr; alias: string | undefined; publicKey: string | undefined },
    log: LogFn,
  ) {
    if (alias) {
      await this.#aliases.set(`accounts:${alias}`, Buffer.from(address.toString()));
    }
    await this.#accounts.set(`${address.toString()}:type`, Buffer.from(type));
    await this.#accounts.set(`${address.toString()}:sk`, secretKey.toBuffer());
    await this.#accounts.set(`${address.toString()}:salt`, salt.toBuffer());
    if (type === 'ecdsasecp256r1ssh' && publicKey) {
      const publicSigningKey = extractECDSAPublicKeyFromBase64String(publicKey);
      await this.storeAccountMetadata(address, 'publicSigningKey', publicSigningKey);
    }
    await this.#aliases.set('accounts:last', Buffer.from(address.toString()));
    log(`Account stored in database with alias${alias ? `es last & ${alias}` : ' last'}`);
  }

  async storeContract(address: AztecAddress, artifactPath: string, log: LogFn, alias?: string) {
    if (alias) {
      await this.#aliases.set(`contracts:${alias}`, Buffer.from(address.toString()));
      await this.#aliases.set(`artifacts:${alias}`, Buffer.from(artifactPath));
    }
    await this.#aliases.set(`contracts:last`, Buffer.from(address.toString()));
    await this.#aliases.set(`artifacts:last`, Buffer.from(artifactPath));
    await this.#aliases.set(`artifacts:${address.toString()}`, Buffer.from(artifactPath));
    log(`Contract stored in database with alias${alias ? `es last & ${alias}` : ' last'}`);
  }

  async storeAuthwitness(authWit: AuthWitness, log: LogFn, alias?: string) {
    if (alias) {
      await this.#aliases.set(`authwits:${alias}`, Buffer.from(authWit.toString()));
    }
    await this.#aliases.set(`authwits:last`, Buffer.from(authWit.toString()));
    log(`Authorization witness stored in database with alias${alias ? `es last & ${alias}` : ' last'}`);
  }

  async storeTx(
    {
      txHash,
      nonce,
      cancellable,
      gasSettings,
    }: { txHash: TxHash; nonce: Fr; cancellable: boolean; gasSettings: GasSettings },
    log: LogFn,
    alias?: string,
  ) {
    if (alias) {
      await this.#aliases.set(`transactions:${alias}`, Buffer.from(txHash.toString()));
    }
    await this.#transactions.set(`${txHash.toString()}:nonce`, nonce.toBuffer());
    await this.#transactions.set(`${txHash.toString()}:cancellable`, Buffer.from(cancellable ? 'true' : 'false'));
    await this.#transactions.set(`${txHash.toString()}:gasSettings`, gasSettings.toBuffer());
    await this.#aliases.set(`transactions:last`, Buffer.from(txHash.toString()));
    log(`Transaction hash stored in database with alias${alias ? `es last & ${alias}` : ' last'}`);
  }

  retrieveTxData(txHash: TxHash) {
    const nonceBuffer = this.#transactions.get(`${txHash.toString()}:nonce`);
    if (!nonceBuffer) {
      throw new Error(
        `Could not find ${txHash.toString()}:nonce. Transaction with hash "${txHash.toString()}" does not exist on this wallet.`,
      );
    }
    const nonce = Fr.fromBuffer(nonceBuffer);
    const cancellable = this.#transactions.get(`${txHash.toString()}:cancellable`)!.toString() === 'true';
    const gasBuffer = this.#transactions.get(`${txHash.toString()}:gasSettings`)!;
    return { txHash, nonce, cancellable, gasSettings: GasSettings.fromBuffer(gasBuffer) };
  }

  tryRetrieveAlias(arg: string) {
    try {
      return this.retrieveAlias(arg);
    } catch (e) {
      return arg;
    }
  }

  retrieveAlias(arg: string) {
    if (Aliases.find(alias => arg.startsWith(`${alias}:`))) {
      const [type, ...alias] = arg.split(':');
      const data = this.#aliases.get(`${type}:${alias.join(':') ?? 'last'}`);
      if (!data) {
        throw new Error(`Could not find alias ${arg}`);
      }
      return data.toString();
    } else {
      throw new Error(`Aliases must start with one of ${Aliases.join(', ')}`);
    }
  }

  listAliases(type?: AliasType) {
    const result = [];
    if (type && !Aliases.includes(type)) {
      throw new Error(`Unknown alias type ${type}`);
    }
    for (const [key, value] of this.#aliases.entries()) {
      if (!type || key.startsWith(`${type}:`)) {
        result.push({ key, value: value.toString() });
      }
    }
    return result;
  }

  async storeAccountMetadata(aliasOrAddress: AztecAddress | string, metadataKey: string, metadata: Buffer) {
    const { address } = this.retrieveAccount(aliasOrAddress);
    await this.#accounts.set(`${address.toString()}:${metadataKey}`, metadata);
  }

  retrieveAccountMetadata(aliasOrAddress: AztecAddress | string, metadataKey: string) {
    const { address } = this.retrieveAccount(aliasOrAddress);
    const result = this.#accounts.get(`${address.toString()}:${metadataKey}`);
    if (!result) {
      throw new Error(`Could not find metadata with key ${metadataKey} for account ${aliasOrAddress}`);
    }
    return result;
  }

  retrieveAccount(address: AztecAddress | string) {
    const secretKeyBuffer = this.#accounts.get(`${address.toString()}:sk`);
    if (!secretKeyBuffer) {
      throw new Error(`Could not find ${address}:sk. Account "${address.toString}" does not exist on this wallet.`);
    }
    const secretKey = Fr.fromBuffer(secretKeyBuffer);
    const salt = Fr.fromBuffer(this.#accounts.get(`${address.toString()}:salt`)!);
    const type = this.#accounts.get(`${address.toString()}:type`)!.toString('utf8') as AccountType;
    return { address, secretKey, salt, type };
  }

  async storeAlias(type: AliasType, key: string, value: Buffer, log: LogFn) {
    await this.#aliases.set(`${type}:${key}`, value);
    log(`Data stored in database with alias ${type}:${key}`);
  }
}
