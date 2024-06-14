import { Fr, computeSecretHash } from '@aztec/aztec.js';

import { U128_UNDERFLOW_ERROR } from '../fixtures/fixtures.js';
import { TokenContractTest } from './token_contract_test.js';

describe('e2e_token_contract shield + redeem shield', () => {
  const t = new TokenContractTest('shielding');
  let { asset, accounts, tokenSim, wallets } = t;
  const secret = Fr.random();
  let secretHash: Fr;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.applyMintSnapshot();
    await t.setup();
    // Have to destructure again to ensure we have latest refs.
    ({ asset, accounts, tokenSim, wallets } = t);
    secretHash = computeSecretHash(secret);
  });

  afterAll(async () => {
    await t.teardown();
  });

  afterEach(async () => {
    await t.tokenSim.check();
  });

  it('on behalf of self', async () => {
    const balancePub = await asset.methods.balance_of_public(accounts[0].address).simulate();
    const amount = balancePub / 2n;
    expect(amount).toBeGreaterThan(0n);

    const receipt = await asset.methods.shield(accounts[0].address, amount, secretHash, 0).send().wait();

    tokenSim.shield(accounts[0].address, amount);
    await tokenSim.check();

    // Redeem it
    await t.addPendingShieldNoteToPXE(0, amount, secretHash, receipt.txHash);
    await asset.methods.redeem_shield(accounts[0].address, amount, secret).send().wait();

    tokenSim.redeemShield(accounts[0].address, amount);
  });

  it('on behalf of other', async () => {
    const balancePub = await asset.methods.balance_of_public(accounts[0].address).simulate();
    const amount = balancePub / 2n;
    const nonce = Fr.random();
    expect(amount).toBeGreaterThan(0n);

    // We need to compute the message we want to sign and add it to the wallet as approved
    const action = asset.withWallet(wallets[1]).methods.shield(accounts[0].address, amount, secretHash, nonce);
    await wallets[0].setPublicAuthWit({ caller: accounts[1].address, action }, true).send().wait();

    const receipt = await action.send().wait();

    tokenSim.shield(accounts[0].address, amount);
    await tokenSim.check();

    // Check that replaying the shield should fail!
    await expect(
      asset.withWallet(wallets[1]).methods.shield(accounts[0].address, amount, secretHash, nonce).simulate(),
    ).rejects.toThrow(/unauthorized/);

    // Redeem it
    await t.addPendingShieldNoteToPXE(0, amount, secretHash, receipt.txHash);
    await asset.methods.redeem_shield(accounts[0].address, amount, secret).send().wait();

    tokenSim.redeemShield(accounts[0].address, amount);
  });

  describe('failure cases', () => {
    it('on behalf of self (more than balance)', async () => {
      const balancePub = await asset.methods.balance_of_public(accounts[0].address).simulate();
      const amount = balancePub + 1n;
      expect(amount).toBeGreaterThan(0n);

      await expect(asset.methods.shield(accounts[0].address, amount, secretHash, 0).simulate()).rejects.toThrow(
        U128_UNDERFLOW_ERROR,
      );
    });

    it('on behalf of self (invalid nonce)', async () => {
      const balancePub = await asset.methods.balance_of_public(accounts[0].address).simulate();
      const amount = balancePub + 1n;
      expect(amount).toBeGreaterThan(0n);

      await expect(asset.methods.shield(accounts[0].address, amount, secretHash, 1).simulate()).rejects.toThrow(
        'Assertion failed: invalid nonce',
      );
    });

    it('on behalf of other (more than balance)', async () => {
      const balancePub = await asset.methods.balance_of_public(accounts[0].address).simulate();
      const amount = balancePub + 1n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      // We need to compute the message we want to sign and add it to the wallet as approved
      const action = asset.withWallet(wallets[1]).methods.shield(accounts[0].address, amount, secretHash, nonce);
      await wallets[0].setPublicAuthWit({ caller: accounts[1].address, action }, true).send().wait();

      await expect(action.simulate()).rejects.toThrow(U128_UNDERFLOW_ERROR);
    });

    it('on behalf of other (wrong designated caller)', async () => {
      const balancePub = await asset.methods.balance_of_public(accounts[0].address).simulate();
      const amount = balancePub + 1n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      // We need to compute the message we want to sign and add it to the wallet as approved
      const action = asset.withWallet(wallets[2]).methods.shield(accounts[0].address, amount, secretHash, nonce);
      await wallets[0].setPublicAuthWit({ caller: accounts[1].address, action }, true).send().wait();

      await expect(action.simulate()).rejects.toThrow(/unauthorized/);
    });

    it('on behalf of other (without approval)', async () => {
      const balance = await asset.methods.balance_of_public(accounts[0].address).simulate();
      const amount = balance / 2n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      await expect(
        asset.withWallet(wallets[1]).methods.shield(accounts[0].address, amount, secretHash, nonce).simulate(),
      ).rejects.toThrow(/unauthorized/);
    });
  });
});
