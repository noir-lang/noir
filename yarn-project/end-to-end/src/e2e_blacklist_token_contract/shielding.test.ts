import { Fr, computeSecretHash } from '@aztec/aztec.js';

import { U128_UNDERFLOW_ERROR } from '../fixtures/index.js';
import { BlacklistTokenContractTest } from './blacklist_token_contract_test.js';

describe('e2e_blacklist_token_contract shield + redeem_shield', () => {
  const t = new BlacklistTokenContractTest('shield');
  let { asset, tokenSim, wallets, blacklisted } = t;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.applyMintSnapshot(); // Beware that we are adding the admin as minter here
    await t.setup();
    // Have to destructure again to ensure we have latest refs.
    ({ asset, tokenSim, wallets, blacklisted } = t);
  }, 600_000);

  afterAll(async () => {
    await t.teardown();
  });

  afterEach(async () => {
    await t.tokenSim.check();
  });

  const secret = Fr.random();
  let secretHash: Fr;

  beforeAll(() => {
    secretHash = computeSecretHash(secret);
  });

  it('on behalf of self', async () => {
    const balancePub = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
    const amount = balancePub / 2n;
    expect(amount).toBeGreaterThan(0n);

    const receipt = await asset.methods.shield(wallets[0].getAddress(), amount, secretHash, 0).send().wait();

    tokenSim.shield(wallets[0].getAddress(), amount);
    await t.tokenSim.check();

    // Redeem it
    await t.addPendingShieldNoteToPXE(0, amount, secretHash, receipt.txHash);
    await asset.methods.redeem_shield(wallets[0].getAddress(), amount, secret).send().wait();

    tokenSim.redeemShield(wallets[0].getAddress(), amount);
  });

  it('on behalf of other', async () => {
    const balancePub = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
    const amount = balancePub / 2n;
    const nonce = Fr.random();
    expect(amount).toBeGreaterThan(0n);

    // We need to compute the message we want to sign and add it to the wallet as approved
    const action = asset.withWallet(wallets[1]).methods.shield(wallets[0].getAddress(), amount, secretHash, nonce);
    await wallets[0].setPublicAuthWit({ caller: wallets[1].getAddress(), action }, true).send().wait();

    const receipt = await action.send().wait();

    tokenSim.shield(wallets[0].getAddress(), amount);
    await t.tokenSim.check();

    // Check that replaying the shield should fail!
    await expect(
      asset.withWallet(wallets[1]).methods.shield(wallets[0].getAddress(), amount, secretHash, nonce).simulate(),
    ).rejects.toThrow(/unauthorized/);

    // Redeem it
    await t.addPendingShieldNoteToPXE(0, amount, secretHash, receipt.txHash);
    await asset.methods.redeem_shield(wallets[0].getAddress(), amount, secret).send().wait();

    tokenSim.redeemShield(wallets[0].getAddress(), amount);
  });

  describe('failure cases', () => {
    it('on behalf of self (more than balance)', async () => {
      const balancePub = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
      const amount = balancePub + 1n;
      expect(amount).toBeGreaterThan(0n);

      await expect(asset.methods.shield(wallets[0].getAddress(), amount, secretHash, 0).prove()).rejects.toThrow(
        U128_UNDERFLOW_ERROR,
      );
    });

    it('on behalf of self (invalid nonce)', async () => {
      const balancePub = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
      const amount = balancePub + 1n;
      expect(amount).toBeGreaterThan(0n);

      await expect(asset.methods.shield(wallets[0].getAddress(), amount, secretHash, 1).prove()).rejects.toThrow(
        'Assertion failed: invalid nonce',
      );
    });

    it('on behalf of other (more than balance)', async () => {
      const balancePub = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
      const amount = balancePub + 1n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      // We need to compute the message we want to sign and add it to the wallet as approved
      const action = asset.withWallet(wallets[1]).methods.shield(wallets[0].getAddress(), amount, secretHash, nonce);
      await wallets[0].setPublicAuthWit({ caller: wallets[1].getAddress(), action }, true).send().wait();

      await expect(action.prove()).rejects.toThrow(U128_UNDERFLOW_ERROR);
    });

    it('on behalf of other (wrong designated caller)', async () => {
      const balancePub = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
      const amount = balancePub + 1n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      // We need to compute the message we want to sign and add it to the wallet as approved
      const action = asset.withWallet(wallets[2]).methods.shield(wallets[0].getAddress(), amount, secretHash, nonce);
      await wallets[0].setPublicAuthWit({ caller: wallets[1].getAddress(), action }, true).send().wait();

      await expect(action.prove()).rejects.toThrow(/unauthorized/);
    });

    it('on behalf of other (without approval)', async () => {
      const balance = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
      const amount = balance / 2n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      await expect(
        asset.withWallet(wallets[1]).methods.shield(wallets[0].getAddress(), amount, secretHash, nonce).simulate(),
      ).rejects.toThrow(/unauthorized/);
    });

    it('shielding from blacklisted account', async () => {
      await expect(
        asset.withWallet(blacklisted).methods.shield(blacklisted.getAddress(), 1n, secretHash, 0).prove(),
      ).rejects.toThrow(/Assertion failed: Blacklisted: Sender .*/);
    });
  });
});
