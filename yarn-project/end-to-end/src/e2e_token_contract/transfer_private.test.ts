import { Fr, computeAuthWitMessageHash } from '@aztec/aztec.js';

import { TokenContractTest } from './token_contract_test.js';

describe('e2e_token_contract transfer private', () => {
  const t = new TokenContractTest('transfer_private');
  let { asset, accounts, tokenSim, wallets, badAccount } = t;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.applyMintSnapshot();
    await t.setup();
    ({ asset, accounts, tokenSim, wallets, badAccount } = t);
  });

  afterAll(async () => {
    await t.teardown();
  });

  afterEach(async () => {
    await t.tokenSim.check();
  });

  it('transfer less than balance', async () => {
    const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
    const amount = balance0 / 2n;
    expect(amount).toBeGreaterThan(0n);
    await asset.methods.transfer(accounts[0].address, accounts[1].address, amount, 0).send().wait();
    tokenSim.transferPrivate(accounts[0].address, accounts[1].address, amount);
  });

  it('transfer to self', async () => {
    const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
    const amount = balance0 / 2n;
    expect(amount).toBeGreaterThan(0n);
    await asset.methods.transfer(accounts[0].address, accounts[0].address, amount, 0).send().wait();
    tokenSim.transferPrivate(accounts[0].address, accounts[0].address, amount);
  });

  it('transfer on behalf of other', async () => {
    const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
    const amount = balance0 / 2n;
    const nonce = Fr.random();
    expect(amount).toBeGreaterThan(0n);

    // We need to compute the message we want to sign and add it to the wallet as approved
    // docs:start:authwit_transfer_example
    const action = asset
      .withWallet(wallets[1])
      .methods.transfer(accounts[0].address, accounts[1].address, amount, nonce);

    const witness = await wallets[0].createAuthWit({ caller: accounts[1].address, action });
    await wallets[1].addAuthWitness(witness);
    expect(await wallets[0].lookupValidity(wallets[0].getAddress(), { caller: accounts[1].address, action })).toEqual({
      isValidInPrivate: true,
      isValidInPublic: false,
    });
    // docs:end:authwit_transfer_example

    // Perform the transfer
    await action.send().wait();
    tokenSim.transferPrivate(accounts[0].address, accounts[1].address, amount);

    // Perform the transfer again, should fail
    const txReplay = asset
      .withWallet(wallets[1])
      .methods.transfer(accounts[0].address, accounts[1].address, amount, nonce)
      .send();
    await expect(txReplay.wait()).rejects.toThrow('Transaction ');
  });

  describe('failure cases', () => {
    it('transfer more than balance', async () => {
      const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
      const amount = balance0 + 1n;
      expect(amount).toBeGreaterThan(0n);
      await expect(
        asset.methods.transfer(accounts[0].address, accounts[1].address, amount, 0).simulate(),
      ).rejects.toThrow('Assertion failed: Balance too low');
    });

    it('transfer on behalf of self with non-zero nonce', async () => {
      const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
      const amount = balance0 - 1n;
      expect(amount).toBeGreaterThan(0n);
      await expect(
        asset.methods.transfer(accounts[0].address, accounts[1].address, amount, 1).simulate(),
      ).rejects.toThrow('Assertion failed: invalid nonce');
    });

    it('transfer more than balance on behalf of other', async () => {
      const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
      const balance1 = await asset.methods.balance_of_private(accounts[1].address).simulate();
      const amount = balance0 + 1n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      // We need to compute the message we want to sign and add it to the wallet as approved
      const action = asset
        .withWallet(wallets[1])
        .methods.transfer(accounts[0].address, accounts[1].address, amount, nonce);

      // Both wallets are connected to same node and PXE so we could just insert directly using
      // await wallet.signAndAddAuthWitness(messageHash, );
      // But doing it in two actions to show the flow.
      const witness = await wallets[0].createAuthWit({ caller: accounts[1].address, action });
      await wallets[1].addAuthWitness(witness);

      // Perform the transfer
      await expect(action.simulate()).rejects.toThrow('Assertion failed: Balance too low');
      expect(await asset.methods.balance_of_private(accounts[0].address).simulate()).toEqual(balance0);
      expect(await asset.methods.balance_of_private(accounts[1].address).simulate()).toEqual(balance1);
    });

    it.skip('transfer into account to overflow', () => {
      // This should already be covered by the mint case earlier. e.g., since we cannot mint to overflow, there is not
      // a way to get funds enough to overflow.
      // Require direct storage manipulation for us to perform a nice explicit case though.
      // See https://github.com/AztecProtocol/aztec-packages/issues/1259
    });

    it('transfer on behalf of other without approval', async () => {
      const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
      const amount = balance0 / 2n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      // We need to compute the message we want to sign and add it to the wallet as approved
      const action = asset
        .withWallet(wallets[1])
        .methods.transfer(accounts[0].address, accounts[1].address, amount, nonce);
      const messageHash = computeAuthWitMessageHash(
        accounts[1].address,
        wallets[0].getChainId(),
        wallets[0].getVersion(),
        action.request(),
      );

      await expect(action.simulate()).rejects.toThrow(
        `Unknown auth witness for message hash ${messageHash.toString()}`,
      );
    });

    it('transfer on behalf of other, wrong designated caller', async () => {
      const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
      const amount = balance0 / 2n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      // We need to compute the message we want to sign and add it to the wallet as approved
      const action = asset
        .withWallet(wallets[2])
        .methods.transfer(accounts[0].address, accounts[1].address, amount, nonce);
      const expectedMessageHash = computeAuthWitMessageHash(
        accounts[2].address,
        wallets[0].getChainId(),
        wallets[0].getVersion(),
        action.request(),
      );

      const witness = await wallets[0].createAuthWit({ caller: accounts[1].address, action });
      await wallets[2].addAuthWitness(witness);

      await expect(action.simulate()).rejects.toThrow(
        `Unknown auth witness for message hash ${expectedMessageHash.toString()}`,
      );
      expect(await asset.methods.balance_of_private(accounts[0].address).simulate()).toEqual(balance0);
    });

    it('transfer on behalf of other, cancelled authwit', async () => {
      const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
      const amount = balance0 / 2n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      // We need to compute the message we want to sign and add it to the wallet as approved
      const action = asset
        .withWallet(wallets[1])
        .methods.transfer(accounts[0].address, accounts[1].address, amount, nonce);

      const witness = await wallets[0].createAuthWit({ caller: accounts[1].address, action });
      await wallets[1].addAuthWitness(witness);

      await wallets[0].cancelAuthWit(witness.requestHash).send().wait();

      // Perform the transfer, should fail because nullifier already emitted
      const txCancelledAuthwit = asset
        .withWallet(wallets[1])
        .methods.transfer(accounts[0].address, accounts[1].address, amount, nonce)
        .send();
      await expect(txCancelledAuthwit.wait()).rejects.toThrowError('Transaction ');
    });

    it('transfer on behalf of other, cancelled authwit, flow 2', async () => {
      const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
      const amount = balance0 / 2n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      // We need to compute the message we want to sign and add it to the wallet as approved
      const action = asset
        .withWallet(wallets[1])
        .methods.transfer(accounts[0].address, accounts[1].address, amount, nonce);

      const witness = await wallets[0].createAuthWit({ caller: accounts[1].address, action });
      await wallets[1].addAuthWitness(witness);

      await wallets[0].cancelAuthWit({ caller: accounts[1].address, action }).send().wait();

      // Perform the transfer, should fail because nullifier already emitted
      const txCancelledAuthwit = asset
        .withWallet(wallets[1])
        .methods.transfer(accounts[0].address, accounts[1].address, amount, nonce)
        .send();
      await expect(txCancelledAuthwit.wait()).rejects.toThrow('Transaction ');
    });

    it('transfer on behalf of other, invalid spend_private_authwit on "from"', async () => {
      const nonce = Fr.random();

      // Should fail as the returned value from the badAccount is malformed
      const txCancelledAuthwit = asset
        .withWallet(wallets[1])
        .methods.transfer(badAccount.address, accounts[1].address, 0, nonce)
        .send();
      await expect(txCancelledAuthwit.wait()).rejects.toThrow(
        "Assertion failed: Message not authorized by account 'result == IS_VALID_SELECTOR'",
      );
    });
  });
});
