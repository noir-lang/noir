/* eslint-disable jsdoc/require-jsdoc */
import { type AztecAddress, BatchCall, type DebugLogger, type Wallet } from '@aztec/aztec.js';
import { type TokenContract } from '@aztec/noir-contracts.js/Token';

import chunk from 'lodash.chunk';

export class TokenSimulator {
  private balancesPrivate: Map<string, bigint> = new Map();
  private balancePublic: Map<string, bigint> = new Map();
  public totalSupply: bigint = 0n;

  constructor(protected token: TokenContract, protected logger: DebugLogger, protected accounts: AztecAddress[]) {}

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

  public async check(wallet: Wallet) {
    const calls = [this.token.methods.total_supply().request()];
    for (const address of this.accounts) {
      calls.push(this.token.methods.balance_of_public(address).request());
      calls.push(this.token.methods.balance_of_private(address).request());
    }

    const batchedCalls = chunk(calls, 4);

    const results = (await Promise.all(batchedCalls.map(batch => new BatchCall(wallet, batch).simulate()))).flat();

    expect(results[0]).toEqual(this.totalSupply);

    // Check that all our balances match
    for (let i = 0; i < this.accounts.length; i++) {
      expect(results[i * 2 + 1]).toEqual(this.balanceOfPublic(this.accounts[i]));
      expect(results[i * 2 + 2]).toEqual(this.balanceOfPrivate(this.accounts[i]));
    }
  }
}
