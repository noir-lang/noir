// Convenience struct to hold an account's address and secret that can easily be passed around.
import { CheatCodes } from '@aztec/aztec.js';
import { AztecAddress, CircuitsWasm, Fr } from '@aztec/circuits.js';
import { pedersenPlookupCommitInputs } from '@aztec/circuits.js/barretenberg';
import { LendingContract } from '@aztec/noir-contracts/types';

import { TokenSimulator } from './token_simulator.js';

/**
 * Contains utilities to compute the "key" for private holdings in the public state.
 */
export class LendingAccount {
  /** The address that owns this account */
  public readonly address: AztecAddress;
  /** The secret used for private deposits */
  public readonly secret: Fr;

  constructor(address: AztecAddress, secret: Fr) {
    this.address = address;
    this.secret = secret;
  }

  /**
   * Computes the key for the private holdings of this account.
   * @returns Key in public space
   */
  public async key(): Promise<Fr> {
    return Fr.fromBuffer(
      pedersenPlookupCommitInputs(
        await CircuitsWasm.get(),
        [this.address, this.secret].map(f => f.toBuffer()),
      ),
    );
  }
}

const WAD = 10n ** 18n;
const BASE = 10n ** 9n;

const muldivDown = (a: bigint, b: bigint, c: bigint) => (a * b) / c;

const muldivUp = (a: bigint, b: bigint, c: bigint) => {
  const adder = (a * b) % c > 0n ? 1n : 0n;
  return muldivDown(a, b, c) + adder;
};

const computeMultiplier = (rate: bigint, dt: bigint) => {
  if (dt == 0n) {
    return BASE;
  }

  const expMinusOne = dt - 1n;
  const expMinusTwo = dt > 2 ? dt - 2n : 0n;

  const basePowerTwo = muldivDown(rate, rate, WAD);
  const basePowerThree = muldivDown(basePowerTwo, rate, WAD);

  const temp = dt * expMinusOne;
  const secondTerm = muldivDown(temp, basePowerTwo, 2n);
  const thirdTerm = muldivDown(temp * expMinusTwo, basePowerThree, 6n);

  const offset = (dt * rate + secondTerm + thirdTerm) / (WAD / BASE);

  return BASE + offset;
};

/**
 * Helper class that emulates the logic of the lending contract. Used to have a "twin" to check values against.
 */
export class LendingSimulator {
  /** interest rate accumulator */
  public accumulator: bigint = 0n;
  /** the timestamp of the simulator*/
  public time: number = 0;

  private collateral: { [key: string]: Fr } = {};
  private staticDebt: { [key: string]: Fr } = {};
  private borrowed: bigint = 0n;
  private mintedOutside: bigint = 0n;

  constructor(
    private cc: CheatCodes,
    private account: LendingAccount,
    private rate: bigint,
    /** the lending contract */
    public lendingContract: LendingContract,
    /** the collateral asset used in the lending contract */
    public collateralAsset: TokenSimulator,
    /** the stable-coin borrowed in the lending contract */
    public stableCoin: TokenSimulator,
  ) {}

  async prepare() {
    this.accumulator = BASE;
    const ts = await this.cc.eth.timestamp();
    this.time = ts + 10 + (ts % 10);
    await this.cc.aztec.warp(this.time);
  }

  async progressTime(diff: number) {
    this.time = this.time + diff;
    await this.cc.aztec.warp(this.time);
    this.accumulator = muldivDown(this.accumulator, computeMultiplier(this.rate, BigInt(diff)), BASE);
  }

  depositPrivate(from: AztecAddress, onBehalfOf: Fr, amount: bigint) {
    this.collateralAsset.unshield(from, this.lendingContract.address, amount);
    this.deposit(onBehalfOf, amount);
  }

  depositPublic(from: AztecAddress, onBehalfOf: Fr, amount: bigint) {
    this.collateralAsset.transferPublic(from, this.lendingContract.address, amount);
    this.deposit(onBehalfOf, amount);
  }

  private deposit(onBehalfOf: Fr, amount: bigint) {
    const coll = this.collateral[onBehalfOf.toString()] ?? Fr.ZERO;
    this.collateral[onBehalfOf.toString()] = new Fr(coll.value + amount);
  }

  withdraw(owner: Fr, recipient: AztecAddress, amount: bigint) {
    const coll = this.collateral[owner.toString()] ?? Fr.ZERO;
    this.collateral[owner.toString()] = new Fr(coll.value - amount);
    this.collateralAsset.transferPublic(this.lendingContract.address, recipient, amount);
  }

  borrow(owner: Fr, recipient: AztecAddress, amount: bigint) {
    const staticDebtBal = this.staticDebt[owner.toString()] ?? Fr.ZERO;
    const increase = muldivUp(amount, BASE, this.accumulator);
    this.staticDebt[owner.toString()] = new Fr(staticDebtBal.value + increase);

    this.stableCoin.mintPublic(recipient, amount);
    this.borrowed += amount;
  }

  repayPrivate(from: AztecAddress, onBehalfOf: Fr, amount: bigint) {
    this.stableCoin.burnPrivate(from, amount);
    this.repay(onBehalfOf, onBehalfOf, amount);
  }

  repayPublic(from: AztecAddress, onBehalfOf: Fr, amount: bigint) {
    this.stableCoin.burnPublic(from, amount);
    this.repay(onBehalfOf, onBehalfOf, amount);
  }

  private repay(from: Fr, onBehalfOf: Fr, amount: bigint) {
    const staticDebtBal = this.staticDebt[onBehalfOf.toString()] ?? Fr.ZERO;
    const decrease = muldivDown(amount, BASE, this.accumulator);
    this.staticDebt[onBehalfOf.toString()] = new Fr(staticDebtBal.value - decrease);

    this.borrowed -= amount;
  }

  mintStableCoinOutsideLoan(recipient: AztecAddress, amount: bigint, priv = false) {
    if (priv) {
      this.stableCoin.mintPrivate(amount);
    } else {
      this.stableCoin.mintPublic(recipient, amount);
    }
    this.mintedOutside += amount;
  }

  async check() {
    // Run checks on both underlying assets
    await this.collateralAsset.check();
    await this.stableCoin.check();

    // Check that total collateral equals total holdings by contract.
    const totalCollateral = Object.values(this.collateral).reduce((a, b) => new Fr(a.value + b.value), Fr.ZERO);
    expect(totalCollateral).toEqual(new Fr(this.collateralAsset.balanceOfPublic(this.lendingContract.address)));

    expect(this.borrowed).toEqual(this.stableCoin.totalSupply - this.mintedOutside);

    const asset = await this.lendingContract.methods.get_asset(0).view();
    expect(asset['interest_accumulator']).toEqual(this.accumulator);
    expect(asset['last_updated_ts']).toEqual(BigInt(this.time));

    for (const key of [this.account.address, await this.account.key()]) {
      const privatePos = await this.lendingContract.methods.get_position(key).view();
      expect(new Fr(privatePos['collateral'])).toEqual(this.collateral[key.toString()] ?? Fr.ZERO);
      expect(new Fr(privatePos['static_debt'])).toEqual(this.staticDebt[key.toString()] ?? Fr.ZERO);
      expect(privatePos['debt']).toEqual(
        muldivUp((this.staticDebt[key.toString()] ?? Fr.ZERO).value, this.accumulator, BASE),
      );
    }
  }
}
