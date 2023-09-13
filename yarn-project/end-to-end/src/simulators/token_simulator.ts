/* eslint-disable jsdoc/require-jsdoc */
import { DebugLogger } from '@aztec/aztec.js';
import { AztecAddress } from '@aztec/circuits.js';
import { TokenContract } from '@aztec/noir-contracts/types';

export class TokenSimulator {
  private balancesPrivate: Map<AztecAddress, bigint> = new Map();
  private balancePublic: Map<AztecAddress, bigint> = new Map();
  public totalSupply: bigint = 0n;

  constructor(protected token: TokenContract, protected logger: DebugLogger, protected accounts: AztecAddress[]) {}

  public mintPrivate(amount: bigint) {
    this.totalSupply += amount;
  }

  public mintPublic(to: AztecAddress, amount: bigint) {
    this.totalSupply += amount;
    const value = this.balancePublic.get(to) || 0n;
    this.balancePublic.set(to, value + amount);
  }

  public transferPublic(from: AztecAddress, to: AztecAddress, amount: bigint) {
    const fromBalance = this.balancePublic.get(from) || 0n;
    this.balancePublic.set(from, fromBalance - amount);
    expect(fromBalance).toBeGreaterThanOrEqual(amount);

    const toBalance = this.balancePublic.get(to) || 0n;
    this.balancePublic.set(to, toBalance + amount);
  }

  public transferPrivate(from: AztecAddress, to: AztecAddress, amount: bigint) {
    const fromBalance = this.balancesPrivate.get(from) || 0n;
    expect(fromBalance).toBeGreaterThanOrEqual(amount);
    this.balancesPrivate.set(from, fromBalance - amount);

    const toBalance = this.balancesPrivate.get(to) || 0n;
    this.balancesPrivate.set(to, toBalance + amount);
  }

  public shield(from: AztecAddress, amount: bigint) {
    const fromBalance = this.balancePublic.get(from) || 0n;
    expect(fromBalance).toBeGreaterThanOrEqual(amount);
    this.balancePublic.set(from, fromBalance - amount);
  }

  public redeemShield(to: AztecAddress, amount: bigint) {
    const toBalance = this.balancesPrivate.get(to) || 0n;
    this.balancesPrivate.set(to, toBalance + amount);
  }

  public unshield(from: AztecAddress, to: AztecAddress, amount: bigint) {
    const fromBalance = this.balancesPrivate.get(from) || 0n;
    const toBalance = this.balancePublic.get(to) || 0n;
    expect(fromBalance).toBeGreaterThanOrEqual(amount);
    this.balancesPrivate.set(from, fromBalance - amount);
    this.balancePublic.set(to, toBalance + amount);
  }

  public burnPrivate(from: AztecAddress, amount: bigint) {
    const fromBalance = this.balancesPrivate.get(from) || 0n;
    expect(fromBalance).toBeGreaterThanOrEqual(amount);
    this.balancesPrivate.set(from, fromBalance - amount);

    this.totalSupply -= amount;
  }

  public burnPublic(from: AztecAddress, amount: bigint) {
    const fromBalance = this.balancePublic.get(from) || 0n;
    expect(fromBalance).toBeGreaterThanOrEqual(amount);
    this.balancePublic.set(from, fromBalance - amount);

    this.totalSupply -= amount;
  }

  public balanceOfPublic(address: AztecAddress) {
    return this.balancePublic.get(address) || 0n;
  }

  public balanceOfPrivate(address: AztecAddress) {
    return this.balancesPrivate.get(address) || 0n;
  }

  public async check() {
    expect(await this.token.methods.total_supply().view()).toEqual(this.totalSupply);

    // Check that all our public matches
    for (const address of this.accounts) {
      expect(await this.token.methods.balance_of_public({ address }).view()).toEqual(this.balanceOfPublic(address));
      expect(await this.token.methods.balance_of_private({ address }).view()).toEqual(this.balanceOfPrivate(address));
    }
  }
}
