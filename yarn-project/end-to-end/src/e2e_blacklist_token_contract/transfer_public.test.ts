import { Fr } from '@aztec/aztec.js';

import { U128_UNDERFLOW_ERROR } from '../fixtures/index.js';
import { BlacklistTokenContractTest } from './blacklist_token_contract_test.js';

describe('e2e_blacklist_token_contract transfer public', () => {
  const t = new BlacklistTokenContractTest('transfer_public');
  let { asset, tokenSim, wallets, blacklisted } = t;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    // Beware that we are adding the admin as minter here, which is very slow because it needs multiple blocks.
    await t.applyMintSnapshot();
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

  it('transfer less than balance', async () => {
    const balance0 = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
    const amount = balance0 / 2n;
    expect(amount).toBeGreaterThan(0n);
    await asset.methods.transfer_public(wallets[0].getAddress(), wallets[1].getAddress(), amount, 0).send().wait();

    tokenSim.transferPublic(wallets[0].getAddress(), wallets[1].getAddress(), amount);
  });

  it('transfer to self', async () => {
    const balance = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
    const amount = balance / 2n;
    expect(amount).toBeGreaterThan(0n);
    await asset.methods.transfer_public(wallets[0].getAddress(), wallets[0].getAddress(), amount, 0).send().wait();

    tokenSim.transferPublic(wallets[0].getAddress(), wallets[0].getAddress(), amount);
  });

  it('transfer on behalf of other', async () => {
    const balance0 = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
    const amount = balance0 / 2n;
    expect(amount).toBeGreaterThan(0n);
    const nonce = Fr.random();

    // docs:start:authwit_public_transfer_example
    const action = asset
      .withWallet(wallets[1])
      .methods.transfer_public(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce);

    await wallets[0].setPublicAuthWit({ caller: wallets[1].getAddress(), action }, true).send().wait();
    // docs:end:authwit_public_transfer_example

    // Perform the transfer
    await action.send().wait();

    tokenSim.transferPublic(wallets[0].getAddress(), wallets[1].getAddress(), amount);

    await expect(
      asset
        .withWallet(wallets[1])
        .methods.transfer_public(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce)
        .simulate(),
    ).rejects.toThrow(/unauthorized/);
  });

  describe('failure cases', () => {
    it('transfer more than balance', async () => {
      const balance0 = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
      const amount = balance0 + 1n;
      const nonce = 0;
      await expect(
        asset.methods.transfer_public(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce).prove(),
      ).rejects.toThrow(U128_UNDERFLOW_ERROR);
    });

    it('transfer on behalf of self with non-zero nonce', async () => {
      const balance0 = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
      const amount = balance0 - 1n;
      const nonce = 1;
      await expect(
        asset.methods.transfer_public(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce).prove(),
      ).rejects.toThrow('Assertion failed: invalid nonce');
    });

    it('transfer on behalf of other without "approval"', async () => {
      const balance0 = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
      const amount = balance0 + 1n;
      const nonce = Fr.random();
      await expect(
        asset
          .withWallet(wallets[1])
          .methods.transfer_public(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce)
          .simulate(),
      ).rejects.toThrow(/unauthorized/);
    });

    it('transfer more than balance on behalf of other', async () => {
      const balance0 = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
      const balance1 = await asset.methods.balance_of_public(wallets[1].getAddress()).simulate();
      const amount = balance0 + 1n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      const action = asset
        .withWallet(wallets[1])
        .methods.transfer_public(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce);

      // We need to compute the message we want to sign and add it to the wallet as approved
      // docs:start:set_public_authwit
      await wallets[0].setPublicAuthWit({ caller: wallets[1].getAddress(), action }, true).send().wait();
      // docs:end:set_public_authwit
      // Perform the transfer
      await expect(action.prove()).rejects.toThrow(U128_UNDERFLOW_ERROR);

      expect(await asset.methods.balance_of_public(wallets[0].getAddress()).simulate()).toEqual(balance0);
      expect(await asset.methods.balance_of_public(wallets[1].getAddress()).simulate()).toEqual(balance1);
    });

    it('transfer on behalf of other, wrong designated caller', async () => {
      const balance0 = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
      const balance1 = await asset.methods.balance_of_public(wallets[1].getAddress()).simulate();
      const amount = balance0 + 2n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      // We need to compute the message we want to sign and add it to the wallet as approved
      const action = asset
        .withWallet(wallets[1])
        .methods.transfer_public(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce);

      await wallets[0].setPublicAuthWit({ caller: wallets[0].getAddress(), action }, true).send().wait();

      // Perform the transfer
      await expect(action.simulate()).rejects.toThrow(/unauthorized/);

      expect(await asset.methods.balance_of_public(wallets[0].getAddress()).simulate()).toEqual(balance0);
      expect(await asset.methods.balance_of_public(wallets[1].getAddress()).simulate()).toEqual(balance1);
    });

    it.skip('transfer into account to overflow', () => {
      // This should already be covered by the mint case earlier. e.g., since we cannot mint to overflow, there is not
      // a way to get funds enough to overflow.
      // Require direct storage manipulation for us to perform a nice explicit case though.
      // See https://github.com/AztecProtocol/aztec-packages/issues/1259
    });

    it('transfer from a blacklisted account', async () => {
      await expect(
        asset.methods.transfer_public(blacklisted.getAddress(), wallets[0].getAddress(), 1n, 0n).prove(),
      ).rejects.toThrow(/Assertion failed: Blacklisted: Sender .*/);
    });

    it('transfer to a blacklisted account', async () => {
      await expect(
        asset.methods.transfer_public(wallets[0].getAddress(), blacklisted.getAddress(), 1n, 0n).prove(),
      ).rejects.toThrow(/Assertion failed: Blacklisted: Recipient .*/);
    });
  });
});
