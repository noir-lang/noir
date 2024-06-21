import { Fr } from '@aztec/aztec.js';

import { U128_UNDERFLOW_ERROR } from '../fixtures/fixtures.js';
import { TokenContractTest } from './token_contract_test.js';

describe('e2e_token_contract transfer public', () => {
  const t = new TokenContractTest('transfer_public');
  let { asset, accounts, tokenSim, wallets, badAccount } = t;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.applyMintSnapshot();
    await t.setup();
    // Have to destructure again to ensure we have latest refs.
    ({ asset, accounts, tokenSim, wallets, badAccount } = t);
  });

  afterAll(async () => {
    await t.teardown();
  });

  afterEach(async () => {
    await t.tokenSim.check();
  });

  it('transfer less than balance', async () => {
    const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
    const amount = balance0 / 2n;
    expect(amount).toBeGreaterThan(0n);
    await asset.methods.transfer_public(accounts[0].address, accounts[1].address, amount, 0).send().wait();

    tokenSim.transferPublic(accounts[0].address, accounts[1].address, amount);
  });

  it('transfer to self', async () => {
    const balance = await asset.methods.balance_of_public(accounts[0].address).simulate();
    const amount = balance / 2n;
    expect(amount).toBeGreaterThan(0n);
    await asset.methods.transfer_public(accounts[0].address, accounts[0].address, amount, 0).send().wait();

    tokenSim.transferPublic(accounts[0].address, accounts[0].address, amount);
  });

  it('transfer on behalf of other', async () => {
    const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
    const amount = balance0 / 2n;
    expect(amount).toBeGreaterThan(0n);
    const nonce = Fr.random();

    // docs:start:authwit_public_transfer_example
    const action = asset
      .withWallet(wallets[1])
      .methods.transfer_public(accounts[0].address, accounts[1].address, amount, nonce);

    await wallets[0].setPublicAuthWit({ caller: accounts[1].address, action }, true).send().wait();
    // docs:end:authwit_public_transfer_example

    // Perform the transfer
    await action.send().wait();

    tokenSim.transferPublic(accounts[0].address, accounts[1].address, amount);

    // Check that the message hash is no longer valid.
    await expect(
      asset
        .withWallet(wallets[1])
        .methods.transfer_public(accounts[0].address, accounts[1].address, amount, nonce)
        .simulate(),
    ).rejects.toThrow(/unauthorized/);
  });

  describe('failure cases', () => {
    it('transfer more than balance', async () => {
      const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
      const amount = balance0 + 1n;
      const nonce = 0;
      await expect(
        asset.methods.transfer_public(accounts[0].address, accounts[1].address, amount, nonce).simulate(),
      ).rejects.toThrow(U128_UNDERFLOW_ERROR);
    });

    it('transfer on behalf of self with non-zero nonce', async () => {
      const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
      const amount = balance0 - 1n;
      const nonce = 1;
      await expect(
        asset.methods.transfer_public(accounts[0].address, accounts[1].address, amount, nonce).simulate(),
      ).rejects.toThrow('Assertion failed: invalid nonce');
    });

    it('transfer on behalf of other without "approval"', async () => {
      const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
      const amount = balance0 + 1n;
      const nonce = Fr.random();
      await expect(
        asset
          .withWallet(wallets[1])
          .methods.transfer_public(accounts[0].address, accounts[1].address, amount, nonce)
          .simulate(),
      ).rejects.toThrow(/unauthorized/);
    });

    it('transfer more than balance on behalf of other', async () => {
      const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
      const balance1 = await asset.methods.balance_of_public(accounts[1].address).simulate();
      const amount = balance0 + 1n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      const action = asset
        .withWallet(wallets[1])
        .methods.transfer_public(accounts[0].address, accounts[1].address, amount, nonce);

      expect(await wallets[0].lookupValidity(wallets[0].getAddress(), { caller: accounts[1].address, action })).toEqual(
        {
          isValidInPrivate: false,
          isValidInPublic: false,
        },
      );

      // We need to compute the message we want to sign and add it to the wallet as approved
      await wallets[0].setPublicAuthWit({ caller: accounts[1].address, action }, true).send().wait();

      expect(await wallets[0].lookupValidity(wallets[0].getAddress(), { caller: accounts[1].address, action })).toEqual(
        {
          isValidInPrivate: false,
          isValidInPublic: true,
        },
      );

      // Perform the transfer
      await expect(action.simulate()).rejects.toThrow(U128_UNDERFLOW_ERROR);

      expect(await asset.methods.balance_of_public(accounts[0].address).simulate()).toEqual(balance0);
      expect(await asset.methods.balance_of_public(accounts[1].address).simulate()).toEqual(balance1);
    });

    it('transfer on behalf of other, wrong designated caller', async () => {
      const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
      const balance1 = await asset.methods.balance_of_public(accounts[1].address).simulate();
      const amount = balance0 + 2n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      // We need to compute the message we want to sign and add it to the wallet as approved
      const action = asset
        .withWallet(wallets[1])
        .methods.transfer_public(accounts[0].address, accounts[1].address, amount, nonce);

      await wallets[0].setPublicAuthWit({ caller: accounts[0].address, action }, true).send().wait();

      // Perform the transfer
      await expect(action.simulate()).rejects.toThrow(/unauthorized/);

      expect(await asset.methods.balance_of_public(accounts[0].address).simulate()).toEqual(balance0);
      expect(await asset.methods.balance_of_public(accounts[1].address).simulate()).toEqual(balance1);
    });

    it('transfer on behalf of other, wrong designated caller', async () => {
      const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
      const balance1 = await asset.methods.balance_of_public(accounts[1].address).simulate();
      const amount = balance0 + 2n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      // We need to compute the message we want to sign and add it to the wallet as approved
      const action = asset
        .withWallet(wallets[1])
        .methods.transfer_public(accounts[0].address, accounts[1].address, amount, nonce);
      await wallets[0].setPublicAuthWit({ caller: accounts[0].address, action }, true).send().wait();

      // Perform the transfer
      await expect(action.simulate()).rejects.toThrow(/unauthorized/);

      expect(await asset.methods.balance_of_public(accounts[0].address).simulate()).toEqual(balance0);
      expect(await asset.methods.balance_of_public(accounts[1].address).simulate()).toEqual(balance1);
    });

    it('transfer on behalf of other, cancelled authwit', async () => {
      const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
      const amount = balance0 / 2n;
      expect(amount).toBeGreaterThan(0n);
      const nonce = Fr.random();

      const action = asset
        .withWallet(wallets[1])
        .methods.transfer_public(accounts[0].address, accounts[1].address, amount, nonce);

      await wallets[0].setPublicAuthWit({ caller: accounts[1].address, action }, true).send().wait();

      await wallets[0].setPublicAuthWit({ caller: accounts[1].address, action }, false).send().wait();

      await expect(
        asset
          .withWallet(wallets[1])
          .methods.transfer_public(accounts[0].address, accounts[1].address, amount, nonce)
          .simulate(),
      ).rejects.toThrowError(/unauthorized/);
    });

    it('transfer on behalf of other, cancelled authwit, flow 2', async () => {
      const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
      const amount = balance0 / 2n;
      expect(amount).toBeGreaterThan(0n);
      const nonce = Fr.random();

      const action = asset
        .withWallet(wallets[1])
        .methods.transfer_public(accounts[0].address, accounts[1].address, amount, nonce);

      await wallets[0].setPublicAuthWit({ caller: accounts[1].address, action }, true).send().wait();

      await wallets[0].setPublicAuthWit({ caller: accounts[1].address, action }, false).send().wait();

      await expect(action.simulate()).rejects.toThrow(/unauthorized/);
    });

    it('transfer on behalf of other, invalid spend_public_authwit on "from"', async () => {
      const nonce = Fr.random();

      await expect(
        asset
          .withWallet(wallets[1])
          .methods.transfer_public(badAccount.address, accounts[1].address, 0, nonce)
          .simulate(),
      ).rejects.toThrow(/unauthorized/);
    });

    it.skip('transfer into account to overflow', () => {
      // This should already be covered by the mint case earlier. e.g., since we cannot mint to overflow, there is not
      // a way to get funds enough to overflow.
      // Require direct storage manipulation for us to perform a nice explicit case though.
      // See https://github.com/AztecProtocol/aztec-packages/issues/1259
    });
  });
});
