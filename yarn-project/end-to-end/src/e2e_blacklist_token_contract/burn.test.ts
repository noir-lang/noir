import { Fr, computeAuthWitMessageHash } from '@aztec/aztec.js';

import { U128_UNDERFLOW_ERROR } from '../fixtures/index.js';
import { BlacklistTokenContractTest } from './blacklist_token_contract_test.js';

describe('e2e_blacklist_token_contract burn', () => {
  const t = new BlacklistTokenContractTest('burn');
  let { asset, tokenSim, wallets, blacklisted } = t;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    // Beware that we are adding the admin as minter here, which is very slow because it needs multiple blocks.
    await t.applyMintSnapshot();
    await t.setup();
    // Have to destructure again to ensure we have latest refs.
    ({ asset, tokenSim, wallets, blacklisted } = t);
  });

  afterAll(async () => {
    await t.teardown();
  });

  afterEach(async () => {
    await t.tokenSim.check();
  });

  describe('public', () => {
    it('burn less than balance', async () => {
      const balance0 = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
      const amount = balance0 / 2n;
      expect(amount).toBeGreaterThan(0n);
      await asset.methods.burn_public(wallets[0].getAddress(), amount, 0).send().wait();

      tokenSim.burnPublic(wallets[0].getAddress(), amount);
    });

    it('burn on behalf of other', async () => {
      const balance0 = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
      const amount = balance0 / 2n;
      expect(amount).toBeGreaterThan(0n);
      const nonce = Fr.random();

      // We need to compute the message we want to sign and add it to the wallet as approved
      const action = asset.withWallet(wallets[1]).methods.burn_public(wallets[0].getAddress(), amount, nonce);
      await wallets[0].setPublicAuthWit({ caller: wallets[1].getAddress(), action }, true).send().wait();

      await action.send().wait();

      tokenSim.burnPublic(wallets[0].getAddress(), amount);

      // Check that the message hash is no longer valid. Need to try to send since nullifiers are handled by sequencer.
      const txReplay = asset.withWallet(wallets[1]).methods.burn_public(wallets[0].getAddress(), amount, nonce).send();
      await expect(txReplay.wait()).rejects.toThrow('Transaction ');
    });

    describe('failure cases', () => {
      it('burn more than balance', async () => {
        const balance0 = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
        const amount = balance0 + 1n;
        const nonce = 0;
        await expect(asset.methods.burn_public(wallets[0].getAddress(), amount, nonce).prove()).rejects.toThrow(
          U128_UNDERFLOW_ERROR,
        );
      });

      it('burn on behalf of self with non-zero nonce', async () => {
        const balance0 = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
        const amount = balance0 - 1n;
        expect(amount).toBeGreaterThan(0n);
        const nonce = 1;
        await expect(asset.methods.burn_public(wallets[0].getAddress(), amount, nonce).prove()).rejects.toThrow(
          'Assertion failed: invalid nonce',
        );
      });

      it('burn on behalf of other without "approval"', async () => {
        const balance0 = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
        const amount = balance0 + 1n;
        const nonce = Fr.random();
        await expect(
          asset.withWallet(wallets[1]).methods.burn_public(wallets[0].getAddress(), amount, nonce).prove(),
        ).rejects.toThrow('Assertion failed: Message not authorized by account');
      });

      it('burn more than balance on behalf of other', async () => {
        const balance0 = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
        const amount = balance0 + 1n;
        const nonce = Fr.random();
        expect(amount).toBeGreaterThan(0n);

        // We need to compute the message we want to sign and add it to the wallet as approved
        const action = asset.withWallet(wallets[1]).methods.burn_public(wallets[0].getAddress(), amount, nonce);
        await wallets[0].setPublicAuthWit({ caller: wallets[1].getAddress(), action }, true).send().wait();

        await expect(action.prove()).rejects.toThrow(U128_UNDERFLOW_ERROR);
      });

      it('burn on behalf of other, wrong designated caller', async () => {
        const balance0 = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
        const amount = balance0 + 2n;
        const nonce = Fr.random();
        expect(amount).toBeGreaterThan(0n);

        // We need to compute the message we want to sign and add it to the wallet as approved
        const action = asset.withWallet(wallets[1]).methods.burn_public(wallets[0].getAddress(), amount, nonce);
        await wallets[0].setPublicAuthWit({ caller: wallets[0].getAddress(), action }, true).send().wait();

        await expect(
          asset.withWallet(wallets[1]).methods.burn_public(wallets[0].getAddress(), amount, nonce).prove(),
        ).rejects.toThrow('Assertion failed: Message not authorized by account');
      });

      it('burn from blacklisted account', async () => {
        await expect(asset.methods.burn_public(blacklisted.getAddress(), 1n, 0).prove()).rejects.toThrow(
          "Assertion failed: Blacklisted: Sender '!from_roles.is_blacklisted'",
        );
      });
    });
  });

  describe('private', () => {
    it('burn less than balance', async () => {
      const balance0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
      const amount = balance0 / 2n;
      expect(amount).toBeGreaterThan(0n);
      await asset.methods.burn(wallets[0].getAddress(), amount, 0).send().wait();
      tokenSim.burnPrivate(wallets[0].getAddress(), amount);
    });

    it('burn on behalf of other', async () => {
      const balance0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
      const amount = balance0 / 2n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      // We need to compute the message we want to sign and add it to the wallet as approved
      const action = asset.withWallet(wallets[1]).methods.burn(wallets[0].getAddress(), amount, nonce);

      // Both wallets are connected to same node and PXE so we could just insert directly
      // But doing it in two actions to show the flow.
      const witness = await wallets[0].createAuthWit({ caller: wallets[1].getAddress(), action });
      await wallets[1].addAuthWitness(witness);

      await asset.withWallet(wallets[1]).methods.burn(wallets[0].getAddress(), amount, nonce).send().wait();
      tokenSim.burnPrivate(wallets[0].getAddress(), amount);

      // Perform the transfer again, should fail
      const txReplay = asset.withWallet(wallets[1]).methods.burn(wallets[0].getAddress(), amount, nonce).send();
      await expect(txReplay.wait()).rejects.toThrow('Transaction ');
    });

    describe('failure cases', () => {
      it('burn more than balance', async () => {
        const balance0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
        const amount = balance0 + 1n;
        expect(amount).toBeGreaterThan(0n);
        await expect(asset.methods.burn(wallets[0].getAddress(), amount, 0).prove()).rejects.toThrow(
          'Assertion failed: Balance too low',
        );
      });

      it('burn on behalf of self with non-zero nonce', async () => {
        const balance0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
        const amount = balance0 - 1n;
        expect(amount).toBeGreaterThan(0n);
        await expect(asset.methods.burn(wallets[0].getAddress(), amount, 1).prove()).rejects.toThrow(
          'Assertion failed: invalid nonce',
        );
      });

      it('burn more than balance on behalf of other', async () => {
        const balance0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
        const amount = balance0 + 1n;
        const nonce = Fr.random();
        expect(amount).toBeGreaterThan(0n);

        // We need to compute the message we want to sign and add it to the wallet as approved
        const action = asset.withWallet(wallets[1]).methods.burn(wallets[0].getAddress(), amount, nonce);

        // Both wallets are connected to same node and PXE so we could just insert directly
        // But doing it in two actions to show the flow.
        const witness = await wallets[0].createAuthWit({ caller: wallets[1].getAddress(), action });
        await wallets[1].addAuthWitness(witness);

        await expect(action.prove()).rejects.toThrow('Assertion failed: Balance too low');
      });

      it('burn on behalf of other without approval', async () => {
        const balance0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
        const amount = balance0 / 2n;
        const nonce = Fr.random();
        expect(amount).toBeGreaterThan(0n);

        // We need to compute the message we want to sign and add it to the wallet as approved
        const action = asset.withWallet(wallets[1]).methods.burn(wallets[0].getAddress(), amount, nonce);
        const messageHash = computeAuthWitMessageHash(
          wallets[1].getAddress(),
          wallets[0].getChainId(),
          wallets[0].getVersion(),
          action.request(),
        );

        await expect(action.prove()).rejects.toThrow(`Unknown auth witness for message hash ${messageHash.toString()}`);
      });

      it('on behalf of other (invalid designated caller)', async () => {
        const balancePriv0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
        const amount = balancePriv0 + 2n;
        const nonce = Fr.random();
        expect(amount).toBeGreaterThan(0n);

        // We need to compute the message we want to sign and add it to the wallet as approved
        const action = asset.withWallet(wallets[2]).methods.burn(wallets[0].getAddress(), amount, nonce);
        const expectedMessageHash = computeAuthWitMessageHash(
          wallets[2].getAddress(),
          wallets[0].getChainId(),
          wallets[0].getVersion(),
          action.request(),
        );

        const witness = await wallets[0].createAuthWit({ caller: wallets[1].getAddress(), action });
        await wallets[2].addAuthWitness(witness);

        await expect(action.prove()).rejects.toThrow(
          `Unknown auth witness for message hash ${expectedMessageHash.toString()}`,
        );
      });

      it('burn from blacklisted account', async () => {
        await expect(asset.methods.burn(blacklisted.getAddress(), 1n, 0).prove()).rejects.toThrow(
          "Assertion failed: Blacklisted: Sender '!from_roles.is_blacklisted'",
        );
      });
    });
  });
});
