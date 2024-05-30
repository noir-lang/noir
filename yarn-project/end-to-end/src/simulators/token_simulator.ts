/* eslint-disable jsdoc/require-jsdoc */
import { type AztecAddress, BatchCall, type DebugLogger, type Wallet } from '@aztec/aztec.js';
import { type TokenContract } from '@aztec/noir-contracts.js/Token';

import chunk from 'lodash.chunk';

export class TokenSimulator {
  private balancesPrivate: Map<string, bigint> = new Map();
  private balancePublic: Map<string, bigint> = new Map();
  public totalSupply: bigint = 0n;

  private lookupProvider: Map<string, Wallet> = new Map();

  constructor(
    protected token: TokenContract,
    protected defaultWallet: Wallet,
    protected logger: DebugLogger,
    protected accounts: AztecAddress[],
  ) {}

  public addAccount(account: AztecAddress) {
    this.accounts.push(account);
  }

  public setLookupProvider(account: AztecAddress, wallet: Wallet) {
    this.lookupProvider.set(account.toString(), wallet);
  }

  public mintPrivate(amount: bigint) {
    this.totalSupply += amount;
  }

  public mintPublic(to: AztecAddress, amount: bigint) {
    this.totalSupply += amount;
    const value = this.balancePublic.get(to.toString()) || 0n;
    this.balancePublic.set(to.toString(), value + amount);
  }

  public transferPublic(from: AztecAddress, to: AztecAddress, amount: bigint) {
    const fromBalance = this.balancePublic.get(from.toString()) || 0n;
    this.balancePublic.set(from.toString(), fromBalance - amount);
    expect(fromBalance).toBeGreaterThanOrEqual(amount);

    const toBalance = this.balancePublic.get(to.toString()) || 0n;
    this.balancePublic.set(to.toString(), toBalance + amount);
  }

  public transferPrivate(from: AztecAddress, to: AztecAddress, amount: bigint) {
    const fromBalance = this.balancesPrivate.get(from.toString()) || 0n;
    expect(fromBalance).toBeGreaterThanOrEqual(amount);
    this.balancesPrivate.set(from.toString(), fromBalance - amount);

    const toBalance = this.balancesPrivate.get(to.toString()) || 0n;
    this.balancesPrivate.set(to.toString(), toBalance + amount);
  }

  public shield(from: AztecAddress, amount: bigint) {
    const fromBalance = this.balancePublic.get(from.toString()) || 0n;
    expect(fromBalance).toBeGreaterThanOrEqual(amount);
    this.balancePublic.set(from.toString(), fromBalance - amount);
  }

  public redeemShield(to: AztecAddress, amount: bigint) {
    const toBalance = this.balancesPrivate.get(to.toString()) || 0n;
    this.balancesPrivate.set(to.toString(), toBalance + amount);
  }

  public unshield(from: AztecAddress, to: AztecAddress, amount: bigint) {
    const fromBalance = this.balancesPrivate.get(from.toString()) || 0n;
    const toBalance = this.balancePublic.get(to.toString()) || 0n;
    expect(fromBalance).toBeGreaterThanOrEqual(amount);
    this.balancesPrivate.set(from.toString(), fromBalance - amount);
    this.balancePublic.set(to.toString(), toBalance + amount);
  }

  public burnPrivate(from: AztecAddress, amount: bigint) {
    const fromBalance = this.balancesPrivate.get(from.toString()) || 0n;
    expect(fromBalance).toBeGreaterThanOrEqual(amount);
    this.balancesPrivate.set(from.toString(), fromBalance - amount);

    this.totalSupply -= amount;
  }

  public burnPublic(from: AztecAddress, amount: bigint) {
    const fromBalance = this.balancePublic.get(from.toString()) || 0n;
    expect(fromBalance).toBeGreaterThanOrEqual(amount);
    this.balancePublic.set(from.toString(), fromBalance - amount);

    this.totalSupply -= amount;
  }

  public balanceOfPublic(address: AztecAddress) {
    return this.balancePublic.get(address.toString()) || 0n;
  }

  public balanceOfPrivate(address: AztecAddress) {
    return this.balancesPrivate.get(address.toString()) || 0n;
  }

  async checkPublic() {
    // public calls
    const calls = [this.token.methods.total_supply().request()];
    for (const address of this.accounts) {
      calls.push(this.token.methods.balance_of_public(address).request());
    }

    const results = (
      await Promise.all(chunk(calls, 4).map(batch => new BatchCall(this.defaultWallet, batch).simulate()))
    ).flat();
    expect(results[0]).toEqual(this.totalSupply);

    // Check that all our balances match
    for (let i = 0; i < this.accounts.length; i++) {
      expect(results[i + 1]).toEqual(this.balanceOfPublic(this.accounts[i]));
    }
  }

  async checkPrivate() {
    // Private calls
    const defaultLookups = [];
    const nonDefaultLookups = [];

    for (const address of this.accounts) {
      if (this.lookupProvider.has(address.toString())) {
        nonDefaultLookups.push(address);
      } else {
        defaultLookups.push(address);
      }
    }

    const defaultCalls = [];
    for (const address of defaultLookups) {
      defaultCalls.push(this.token.methods.balance_of_private(address).request());
    }
    const results = (
      await Promise.all(chunk(defaultCalls, 4).map(batch => new BatchCall(this.defaultWallet, batch).simulate()))
    ).flat();
    for (let i = 0; i < defaultLookups.length; i++) {
      expect(results[i]).toEqual(this.balanceOfPrivate(defaultLookups[i]));
    }

    // We are just running individual calls for the non-default lookups
    // @todo We should also batch these
    for (const address of nonDefaultLookups) {
      const wallet = this.lookupProvider.get(address.toString());
      const asset = wallet ? this.token.withWallet(wallet) : this.token;

      const actualPrivateBalance = await asset.methods.balance_of_private({ address }).simulate();
      expect(actualPrivateBalance).toEqual(this.balanceOfPrivate(address));
    }
  }

  public async check() {
    await this.checkPublic();
    await this.checkPrivate();
  }
}
