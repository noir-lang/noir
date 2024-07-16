import { Fr, type TxHash, computeSecretHash } from '@aztec/aztec.js';

import { BITSIZE_TOO_BIG_ERROR, U128_OVERFLOW_ERROR } from '../fixtures/fixtures.js';
import { TokenContractTest } from './token_contract_test.js';

describe('e2e_token_contract minting', () => {
  const t = new TokenContractTest('minting');
  let { asset, accounts, tokenSim, wallets } = t;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.setup();
    ({ asset, accounts, tokenSim, wallets } = t);
  });

  afterAll(async () => {
    await t.teardown();
  });

  afterEach(async () => {
    await t.tokenSim.check();
  });

  describe('Public', () => {
    it('as minter', async () => {
      const amount = 10000n;
      await asset.methods.mint_public(accounts[0].address, amount).send().wait();

      tokenSim.mintPublic(accounts[0].address, amount);
      expect(await asset.methods.balance_of_public(accounts[0].address).simulate()).toEqual(
        tokenSim.balanceOfPublic(accounts[0].address),
      );
      expect(await asset.methods.total_supply().simulate()).toEqual(tokenSim.totalSupply);
    });

    describe('failure cases', () => {
      it('as non-minter', async () => {
        const amount = 10000n;
        await expect(
          asset.withWallet(wallets[1]).methods.mint_public(accounts[0].address, amount).simulate(),
        ).rejects.toThrow('Assertion failed: caller is not minter');
      });

      it('mint >u128 tokens to overflow', async () => {
        const amount = 2n ** 128n; // U128::max() + 1;
        await expect(asset.methods.mint_public(accounts[0].address, amount).simulate()).rejects.toThrow(
          BITSIZE_TOO_BIG_ERROR,
        );
      });

      it('mint <u128 but recipient balance >u128', async () => {
        const amount = 2n ** 128n - tokenSim.balanceOfPublic(accounts[0].address);
        await expect(asset.methods.mint_public(accounts[0].address, amount).simulate()).rejects.toThrow(
          U128_OVERFLOW_ERROR,
        );
      });

      it('mint <u128 but such that total supply >u128', async () => {
        const amount = 2n ** 128n - tokenSim.balanceOfPublic(accounts[0].address);
        await expect(asset.methods.mint_public(accounts[1].address, amount).simulate()).rejects.toThrow(
          U128_OVERFLOW_ERROR,
        );
      });
    });
  });

  describe('Private', () => {
    const secret = Fr.random();
    const amount = 10000n;
    let secretHash: Fr;
    let txHash: TxHash;

    beforeAll(() => {
      secretHash = computeSecretHash(secret);
    });

    describe('Mint flow', () => {
      it('mint_private as minter', async () => {
        const receipt = await asset.methods.mint_private(amount, secretHash).send().wait();
        tokenSim.mintPrivate(amount);
        txHash = receipt.txHash;
      });

      it('redeem as recipient', async () => {
        await t.addPendingShieldNoteToPXE(0, amount, secretHash, txHash);
        const txClaim = asset.methods.redeem_shield(accounts[0].address, amount, secret).send();
        // docs:start:debug
        const receiptClaim = await txClaim.wait({ debug: true });
        // docs:end:debug
        tokenSim.redeemShield(accounts[0].address, amount);
        // 1 note should be created containing `amount` of tokens
        const { visibleIncomingNotes } = receiptClaim.debugInfo!;
        expect(visibleIncomingNotes.length).toBe(1);
        expect(visibleIncomingNotes[0].note.items[0].toBigInt()).toBe(amount);
      });
    });

    describe('failure cases', () => {
      it('try to redeem as recipient (double-spend) [REVERTS]', async () => {
        await expect(t.addPendingShieldNoteToPXE(0, amount, secretHash, txHash)).rejects.toThrow(
          'The note has been destroyed.',
        );
        await expect(asset.methods.redeem_shield(accounts[0].address, amount, secret).simulate()).rejects.toThrow(
          `Assertion failed: Cannot return zero notes`,
        );
      });

      it('mint_private as non-minter', async () => {
        await expect(asset.withWallet(wallets[1]).methods.mint_private(amount, secretHash).simulate()).rejects.toThrow(
          'Assertion failed: caller is not minter',
        );
      });

      it('mint_private as non-minter, bypassing account entrypoint', async () => {
        const request = await asset.withWallet(wallets[1]).methods.mint_private(amount, secretHash).create();
        await expect(wallets[1].simulateTx(request, true, accounts[0].address)).rejects.toThrow(
          'Assertion failed: Users cannot set msg_sender in first call',
        );
      });

      it('mint >u128 tokens to overflow', async () => {
        const amount = 2n ** 128n; // U128::max() + 1;
        await expect(asset.methods.mint_private(amount, secretHash).simulate()).rejects.toThrow(BITSIZE_TOO_BIG_ERROR);
      });

      it('mint <u128 but recipient balance >u128', async () => {
        const amount = 2n ** 128n - tokenSim.balanceOfPrivate(accounts[0].address);
        expect(amount).toBeLessThan(2n ** 128n);
        await expect(asset.methods.mint_private(amount, secretHash).simulate()).rejects.toThrow(U128_OVERFLOW_ERROR);
      });

      it('mint <u128 but such that total supply >u128', async () => {
        const amount = 2n ** 128n - tokenSim.totalSupply;
        await expect(asset.methods.mint_private(amount, secretHash).simulate()).rejects.toThrow(U128_OVERFLOW_ERROR);
      });
    });
  });
});
