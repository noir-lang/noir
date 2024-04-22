import {
  type AccountWallet,
  AztecAddress,
  type DebugLogger,
  ExtendedNote,
  Fr,
  FunctionSelector,
  Note,
  type TxHash,
  type Wallet,
  computeAuthWitMessageHash,
  computeMessageSecretHash,
} from '@aztec/aztec.js';
import { TokenBlacklistContract, type TokenContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { BITSIZE_TOO_BIG_ERROR, U128_OVERFLOW_ERROR, U128_UNDERFLOW_ERROR } from './fixtures/fixtures.js';
import { publicDeployAccounts, setup } from './fixtures/utils.js';
import { TokenSimulator } from './simulators/token_simulator.js';

const TIMEOUT = 120_000;

describe('e2e_blacklist_token_contract', () => {
  jest.setTimeout(TIMEOUT);

  let teardown: () => Promise<void>;
  let wallets: AccountWallet[];
  let logger: DebugLogger;

  let asset: TokenBlacklistContract;

  let admin: Wallet;
  let other: Wallet;
  let blacklisted: Wallet;

  let tokenSim: TokenSimulator;

  const DELAY = 5;

  async function mineBlock() {
    await asset.methods.get_roles(admin.getAddress()).send().wait();
  }

  async function mineBlocks(amount: number) {
    for (let i = 0; i < amount; ++i) {
      await mineBlock();
    }
  }

  class Role {
    private isAdmin = false;
    private isMinter = false;
    private isBlacklisted = false;

    withAdmin() {
      this.isAdmin = true;
      return this;
    }

    withMinter() {
      this.isMinter = true;
      return this;
    }

    withBlacklisted() {
      this.isBlacklisted = true;
      return this;
    }

    toCallValue() {
      return { is_admin: this.isAdmin, is_minter: this.isMinter, is_blacklisted: this.isBlacklisted };
    }

    toReturnValue() {
      // This matches the result of the serialize() function in the Noir struct

      let value = 0;
      if (this.isAdmin) {
        value += 1;
      }

      if (this.isMinter) {
        value += 2;
      }

      if (this.isBlacklisted) {
        value += 4;
      }

      return [BigInt(value)];
    }
  }

  const addPendingShieldNoteToPXE = async (accountIndex: number, amount: bigint, secretHash: Fr, txHash: TxHash) => {
    const note = new Note([new Fr(amount), secretHash]);
    const extendedNote = new ExtendedNote(
      note,
      wallets[accountIndex].getAddress(),
      asset.address,
      TokenBlacklistContract.storage.pending_shields.slot,
      TokenBlacklistContract.notes.TransparentNote.id,
      txHash,
    );
    await wallets[accountIndex].addNote(extendedNote);
  };

  beforeAll(async () => {
    ({ teardown, logger, wallets } = await setup(4));
    admin = wallets[0];
    other = wallets[1];
    blacklisted = wallets[2];

    await publicDeployAccounts(admin, wallets.slice(0, 3));

    const deployTx = TokenBlacklistContract.deploy(admin, admin.getAddress()).send();
    const receipt = await deployTx.wait();
    asset = receipt.contract;

    await mineBlocks(DELAY); // This gets us past the block of change

    expect(await asset.methods.get_roles(admin.getAddress()).simulate()).toEqual(
      new Role().withAdmin().toReturnValue(),
    );

    logger.info(`Token deployed to ${asset.address}`);
    tokenSim = new TokenSimulator(
      asset as unknown as TokenContract,
      logger,
      wallets.map(a => a.getAddress()),
    );

    asset.artifact.functions.forEach(fn => {
      logger.info(
        `Function ${fn.name} has ${fn.bytecode.length} bytes and the selector: ${FunctionSelector.fromNameAndParameters(
          fn.name,
          fn.parameters,
        )}`,
      );
    });
  }, 100_000);

  afterAll(() => teardown());

  afterEach(async () => {
    await tokenSim.check();
  }, TIMEOUT);

  describe('Access controlled functions', () => {
    it('grant mint permission to the admin', async () => {
      const adminMinterRole = new Role().withAdmin().withMinter();
      await asset
        .withWallet(admin)
        .methods.update_roles(admin.getAddress(), adminMinterRole.toCallValue())
        .send()
        .wait();

      await mineBlocks(DELAY); // This gets us past the block of change

      expect(await asset.methods.get_roles(admin.getAddress()).simulate()).toEqual(adminMinterRole.toReturnValue());
    });

    it('create a new admin', async () => {
      const adminRole = new Role().withAdmin();
      await asset.withWallet(admin).methods.update_roles(other.getAddress(), adminRole.toCallValue()).send().wait();

      await mineBlocks(DELAY); // This gets us past the block of change

      expect(await asset.methods.get_roles(other.getAddress()).simulate()).toEqual(adminRole.toReturnValue());
    });

    it('revoke the new admin', async () => {
      const noRole = new Role();
      await asset.withWallet(admin).methods.update_roles(other.getAddress(), noRole.toCallValue()).send().wait();

      await mineBlocks(DELAY); // This gets us past the block of change

      expect(await asset.methods.get_roles(other.getAddress()).simulate()).toEqual(noRole.toReturnValue());
    });

    it('blacklist account', async () => {
      const blacklistRole = new Role().withBlacklisted();
      await asset
        .withWallet(admin)
        .methods.update_roles(blacklisted.getAddress(), blacklistRole.toCallValue())
        .send()
        .wait();

      await mineBlocks(DELAY); // This gets us past the block of change

      expect(await asset.methods.get_roles(blacklisted.getAddress()).simulate()).toEqual(blacklistRole.toReturnValue());
    });

    describe('failure cases', () => {
      it('set roles from non admin', async () => {
        const newRole = new Role().withAdmin().withAdmin();
        await expect(
          asset.withWallet(other).methods.update_roles(AztecAddress.random(), newRole.toCallValue()).prove(),
        ).rejects.toThrow("Assertion failed: caller is not admin 'caller_roles.isAdmin'");
      });

      it('revoke minter from non admin', async () => {
        const noRole = new Role();
        await expect(
          asset.withWallet(other).methods.update_roles(admin.getAddress(), noRole.toCallValue()).prove(),
        ).rejects.toThrow("Assertion failed: caller is not admin 'caller_roles.isAdmin'");
      });
    });
  });

  describe('Minting', () => {
    describe('Public', () => {
      it('as minter', async () => {
        const amount = 10000n;
        await asset.methods.mint_public(wallets[0].getAddress(), amount).send().wait();

        tokenSim.mintPublic(wallets[0].getAddress(), amount);
        expect(await asset.methods.balance_of_public(wallets[0].getAddress()).simulate()).toEqual(
          tokenSim.balanceOfPublic(wallets[0].getAddress()),
        );
        expect(await asset.methods.total_supply().simulate()).toEqual(tokenSim.totalSupply);
      });

      describe('failure cases', () => {
        it('as non-minter', async () => {
          const amount = 10000n;
          await expect(
            asset.withWallet(wallets[1]).methods.mint_public(wallets[0].getAddress(), amount).prove(),
          ).rejects.toThrow('Assertion failed: caller is not minter');
        });

        it('mint >u128 tokens to overflow', async () => {
          const amount = 2n ** 128n; // U128::max() + 1;
          await expect(asset.methods.mint_public(wallets[0].getAddress(), amount).prove()).rejects.toThrow(
            BITSIZE_TOO_BIG_ERROR,
          );
        });

        it('mint <u128 but recipient balance >u128', async () => {
          const amount = 2n ** 128n - tokenSim.balanceOfPublic(wallets[0].getAddress());
          await expect(asset.methods.mint_public(wallets[0].getAddress(), amount).prove()).rejects.toThrow(
            U128_OVERFLOW_ERROR,
          );
        });

        it('mint <u128 but such that total supply >u128', async () => {
          const amount = 2n ** 128n - tokenSim.balanceOfPublic(wallets[0].getAddress());
          await expect(asset.methods.mint_public(wallets[1].getAddress(), amount).prove()).rejects.toThrow(
            U128_OVERFLOW_ERROR,
          );
        });

        it('mint to blacklisted entity', async () => {
          await expect(
            asset.withWallet(wallets[1]).methods.mint_public(blacklisted.getAddress(), 1n).prove(),
          ).rejects.toThrow("Assertion failed: Blacklisted: Recipient '!to_roles.isBlacklisted'");
        });
      });
    });

    describe('Private', () => {
      const secret = Fr.random();
      const amount = 10000n;
      let secretHash: Fr;
      let txHash: TxHash;

      beforeAll(() => {
        secretHash = computeMessageSecretHash(secret);
      });

      describe('Mint flow', () => {
        it('mint_private as minter', async () => {
          const receipt = await asset.methods.mint_private(amount, secretHash).send().wait();
          tokenSim.mintPrivate(amount);
          txHash = receipt.txHash;
        });

        it('redeem as recipient', async () => {
          await addPendingShieldNoteToPXE(0, amount, secretHash, txHash);

          const receiptClaim = await asset.methods
            .redeem_shield(wallets[0].getAddress(), amount, secret)
            .send()
            .wait({ debug: true });

          tokenSim.redeemShield(wallets[0].getAddress(), amount);
          // 1 note should be created containing `amount` of tokens
          const { visibleNotes } = receiptClaim.debugInfo!;
          expect(visibleNotes.length).toBe(1);
          expect(visibleNotes[0].note.items[0].toBigInt()).toBe(amount);
        });
      });

      describe('failure cases', () => {
        it('try to redeem as recipient (double-spend) [REVERTS]', async () => {
          await expect(addPendingShieldNoteToPXE(0, amount, secretHash, txHash)).rejects.toThrow(
            'The note has been destroyed.',
          );
          await expect(asset.methods.redeem_shield(wallets[0].getAddress(), amount, secret).prove()).rejects.toThrow(
            `Assertion failed: Cannot return zero notes`,
          );
        });

        it('mint_private as non-minter', async () => {
          await expect(asset.withWallet(wallets[1]).methods.mint_private(amount, secretHash).prove()).rejects.toThrow(
            'Assertion failed: caller is not minter',
          );
        });

        it('mint >u128 tokens to overflow', async () => {
          const amount = 2n ** 128n; // U128::max() + 1;
          await expect(asset.methods.mint_private(amount, secretHash).prove()).rejects.toThrow(BITSIZE_TOO_BIG_ERROR);
        });

        it('mint <u128 but recipient balance >u128', async () => {
          const amount = 2n ** 128n - tokenSim.balanceOfPrivate(wallets[0].getAddress());
          expect(amount).toBeLessThan(2n ** 128n);
          await expect(asset.methods.mint_private(amount, secretHash).prove()).rejects.toThrow(U128_OVERFLOW_ERROR);
        });

        it('mint <u128 but such that total supply >u128', async () => {
          const amount = 2n ** 128n - tokenSim.totalSupply;
          await expect(asset.methods.mint_private(amount, secretHash).prove()).rejects.toThrow(U128_OVERFLOW_ERROR);
        });

        it('mint and try to redeem at blacklist', async () => {
          await expect(asset.methods.redeem_shield(blacklisted.getAddress(), amount, secret).prove()).rejects.toThrow(
            "Assertion failed: Blacklisted: Recipient '!to_roles.isBlacklisted'",
          );
        });
      });
    });
  });

  describe('Transfer', () => {
    describe('public', () => {
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

        // Check that the message hash is no longer valid. Need to try to send since nullifiers are handled by sequencer.
        const txReplay = asset
          .withWallet(wallets[1])
          .methods.transfer_public(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce)
          .send();
        await expect(txReplay.wait()).rejects.toThrow('Transaction ');
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
              .prove(),
          ).rejects.toThrow('Assertion failed: Message not authorized by account');
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
          await wallets[0].setPublicAuthWit({ caller: wallets[1].getAddress(), action }, true).send().wait();

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
          await expect(action.prove()).rejects.toThrow('Assertion failed: Message not authorized by account');

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
          await expect(action.prove()).rejects.toThrow('Assertion failed: Message not authorized by account');

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
          ).rejects.toThrow("Assertion failed: Blacklisted: Sender '!from_roles.isBlacklisted'");
        });

        it('transfer to a blacklisted account', async () => {
          await expect(
            asset.methods.transfer_public(wallets[0].getAddress(), blacklisted.getAddress(), 1n, 0n).prove(),
          ).rejects.toThrow("Assertion failed: Blacklisted: Recipient '!to_roles.isBlacklisted'");
        });
      });
    });

    describe('private', () => {
      it('transfer less than balance', async () => {
        const balance0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
        const amount = balance0 / 2n;
        expect(amount).toBeGreaterThan(0n);
        await asset.methods.transfer(wallets[0].getAddress(), wallets[1].getAddress(), amount, 0).send().wait();
        tokenSim.transferPrivate(wallets[0].getAddress(), wallets[1].getAddress(), amount);
      });

      it('transfer to self', async () => {
        const balance0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
        const amount = balance0 / 2n;
        expect(amount).toBeGreaterThan(0n);

        await asset.methods.transfer(wallets[0].getAddress(), wallets[0].getAddress(), amount, 0).send().wait();
        tokenSim.transferPrivate(wallets[0].getAddress(), wallets[0].getAddress(), amount);
      });

      it('transfer on behalf of other', async () => {
        const balance0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
        const amount = balance0 / 2n;
        const nonce = Fr.random();
        expect(amount).toBeGreaterThan(0n);

        // We need to compute the message we want to sign and add it to the wallet as approved
        // docs:start:authwit_transfer_example
        // docs:start:authwit_computeAuthWitMessageHash
        const action = asset
          .withWallet(wallets[1])
          .methods.transfer(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce);
        // docs:end:authwit_computeAuthWitMessageHash

        const witness = await wallets[0].createAuthWit({ caller: wallets[1].getAddress(), action });
        await wallets[1].addAuthWitness(witness);
        // docs:end:authwit_transfer_example

        // Perform the transfer
        await action.send().wait();
        tokenSim.transferPrivate(wallets[0].getAddress(), wallets[1].getAddress(), amount);

        // Perform the transfer again, should fail
        const txReplay = asset
          .withWallet(wallets[1])
          .methods.transfer(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce)
          .send();
        await expect(txReplay.wait()).rejects.toThrow('Transaction ');
      });

      describe('failure cases', () => {
        it('transfer more than balance', async () => {
          const balance0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
          const amount = balance0 + 1n;
          expect(amount).toBeGreaterThan(0n);

          await expect(
            asset.methods.transfer(wallets[0].getAddress(), wallets[1].getAddress(), amount, 0).prove(),
          ).rejects.toThrow('Assertion failed: Balance too low');
        });

        it('transfer on behalf of self with non-zero nonce', async () => {
          const balance0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
          const amount = balance0 - 1n;
          expect(amount).toBeGreaterThan(0n);

          await expect(
            asset.methods.transfer(wallets[0].getAddress(), wallets[1].getAddress(), amount, 1).prove(),
          ).rejects.toThrow('Assertion failed: invalid nonce');
        });

        it('transfer more than balance on behalf of other', async () => {
          const balance0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
          const balance1 = await asset.methods.balance_of_private(wallets[1].getAddress()).simulate();
          const amount = balance0 + 1n;
          const nonce = Fr.random();
          expect(amount).toBeGreaterThan(0n);

          // We need to compute the message we want to sign and add it to the wallet as approved
          const action = asset
            .withWallet(wallets[1])
            .methods.transfer(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce);

          // Both wallets are connected to same node and PXE so we could just insert directly
          // But doing it in two actions to show the flow.
          const witness = await wallets[0].createAuthWit({ caller: wallets[1].getAddress(), action });
          await wallets[1].addAuthWitness(witness);

          // Perform the transfer
          await expect(action.prove()).rejects.toThrow('Assertion failed: Balance too low');
          expect(await asset.methods.balance_of_private(wallets[0].getAddress()).simulate()).toEqual(balance0);
          expect(await asset.methods.balance_of_private(wallets[1].getAddress()).simulate()).toEqual(balance1);
        });

        it.skip('transfer into account to overflow', () => {
          // This should already be covered by the mint case earlier. e.g., since we cannot mint to overflow, there is not
          // a way to get funds enough to overflow.
          // Require direct storage manipulation for us to perform a nice explicit case though.
          // See https://github.com/AztecProtocol/aztec-packages/issues/1259
        });

        it('transfer on behalf of other without approval', async () => {
          const balance0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
          const amount = balance0 / 2n;
          const nonce = Fr.random();
          expect(amount).toBeGreaterThan(0n);

          // We need to compute the message we want to sign and add it to the wallet as approved
          const action = asset
            .withWallet(wallets[1])
            .methods.transfer(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce);
          const messageHash = computeAuthWitMessageHash(
            wallets[1].getAddress(),
            wallets[0].getChainId(),
            wallets[0].getVersion(),
            action.request(),
          );

          await expect(action.prove()).rejects.toThrow(
            `Unknown auth witness for message hash ${messageHash.toString()}`,
          );
        });

        it('transfer on behalf of other, wrong designated caller', async () => {
          const balance0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
          const amount = balance0 / 2n;
          const nonce = Fr.random();
          expect(amount).toBeGreaterThan(0n);

          // We need to compute the message we want to sign and add it to the wallet as approved
          const action = asset
            .withWallet(wallets[2])
            .methods.transfer(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce);
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
          expect(await asset.methods.balance_of_private(wallets[0].getAddress()).simulate()).toEqual(balance0);
        });

        it('transfer from a blacklisted account', async () => {
          await expect(
            asset.methods.transfer(blacklisted.getAddress(), wallets[0].getAddress(), 1n, 0).prove(),
          ).rejects.toThrow("Assertion failed: Blacklisted: Sender '!from_roles.isBlacklisted'");
        });

        it('transfer to a blacklisted account', async () => {
          await expect(
            asset.methods.transfer(wallets[0].getAddress(), blacklisted.getAddress(), 1n, 0).prove(),
          ).rejects.toThrow("Assertion failed: Blacklisted: Recipient '!to_roles.isBlacklisted'");
        });
      });
    });
  });

  describe('Shielding (shield + redeem_shield)', () => {
    const secret = Fr.random();
    let secretHash: Fr;

    beforeAll(() => {
      secretHash = computeMessageSecretHash(secret);
    });

    it('on behalf of self', async () => {
      const balancePub = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
      const amount = balancePub / 2n;
      expect(amount).toBeGreaterThan(0n);

      const receipt = await asset.methods.shield(wallets[0].getAddress(), amount, secretHash, 0).send().wait();

      tokenSim.shield(wallets[0].getAddress(), amount);
      await tokenSim.check();

      // Redeem it
      await addPendingShieldNoteToPXE(0, amount, secretHash, receipt.txHash);
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
      await tokenSim.check();

      // Check that replaying the shield should fail!
      const txReplay = asset
        .withWallet(wallets[1])
        .methods.shield(wallets[0].getAddress(), amount, secretHash, nonce)
        .send();
      await expect(txReplay.wait()).rejects.toThrow('Transaction ');

      // Redeem it
      await addPendingShieldNoteToPXE(0, amount, secretHash, receipt.txHash);
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

        await expect(action.prove()).rejects.toThrow('Assertion failed: Message not authorized by account');
      });

      it('on behalf of other (without approval)', async () => {
        const balance = await asset.methods.balance_of_public(wallets[0].getAddress()).simulate();
        const amount = balance / 2n;
        const nonce = Fr.random();
        expect(amount).toBeGreaterThan(0n);

        await expect(
          asset.withWallet(wallets[1]).methods.shield(wallets[0].getAddress(), amount, secretHash, nonce).prove(),
        ).rejects.toThrow(`Assertion failed: Message not authorized by account`);
      });

      it('shielding from blacklisted account', async () => {
        await expect(
          asset.withWallet(blacklisted).methods.shield(blacklisted.getAddress(), 1n, secretHash, 0).prove(),
        ).rejects.toThrow("Assertion failed: Blacklisted: Sender '!from_roles.isBlacklisted'");
      });
    });
  });

  describe('Unshielding', () => {
    it('on behalf of self', async () => {
      const balancePriv = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
      const amount = balancePriv / 2n;
      expect(amount).toBeGreaterThan(0n);

      await asset.methods.unshield(wallets[0].getAddress(), wallets[0].getAddress(), amount, 0).send().wait();

      tokenSim.unshield(wallets[0].getAddress(), wallets[0].getAddress(), amount);
    });

    it('on behalf of other', async () => {
      const balancePriv0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
      const amount = balancePriv0 / 2n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      // We need to compute the message we want to sign and add it to the wallet as approved
      const action = asset
        .withWallet(wallets[1])
        .methods.unshield(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce);

      // Both wallets are connected to same node and PXE so we could just insert directly
      // But doing it in two actions to show the flow.
      const witness = await wallets[0].createAuthWit({ caller: wallets[1].getAddress(), action });
      await wallets[1].addAuthWitness(witness);

      await action.send().wait();
      tokenSim.unshield(wallets[0].getAddress(), wallets[1].getAddress(), amount);

      // Perform the transfer again, should fail
      const txReplay = asset
        .withWallet(wallets[1])
        .methods.unshield(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce)
        .send();
      await expect(txReplay.wait()).rejects.toThrow('Transaction ');
      // @todo @LHerskind This error is weird?
    });

    describe('failure cases', () => {
      it('on behalf of self (more than balance)', async () => {
        const balancePriv = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
        const amount = balancePriv + 1n;
        expect(amount).toBeGreaterThan(0n);

        await expect(
          asset.methods.unshield(wallets[0].getAddress(), wallets[0].getAddress(), amount, 0).prove(),
        ).rejects.toThrow('Assertion failed: Balance too low');
      });

      it('on behalf of self (invalid nonce)', async () => {
        const balancePriv = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
        const amount = balancePriv + 1n;
        expect(amount).toBeGreaterThan(0n);

        await expect(
          asset.methods.unshield(wallets[0].getAddress(), wallets[0].getAddress(), amount, 1).prove(),
        ).rejects.toThrow('Assertion failed: invalid nonce');
      });

      it('on behalf of other (more than balance)', async () => {
        const balancePriv0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
        const amount = balancePriv0 + 2n;
        const nonce = Fr.random();
        expect(amount).toBeGreaterThan(0n);

        // We need to compute the message we want to sign and add it to the wallet as approved
        const action = asset
          .withWallet(wallets[1])
          .methods.unshield(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce);

        // Both wallets are connected to same node and PXE so we could just insert directly
        // But doing it in two actions to show the flow.
        const witness = await wallets[0].createAuthWit({ caller: wallets[1].getAddress(), action });
        await wallets[1].addAuthWitness(witness);

        await expect(action.prove()).rejects.toThrow('Assertion failed: Balance too low');
      });

      it('on behalf of other (invalid designated caller)', async () => {
        const balancePriv0 = await asset.methods.balance_of_private(wallets[0].getAddress()).simulate();
        const amount = balancePriv0 + 2n;
        const nonce = Fr.random();
        expect(amount).toBeGreaterThan(0n);

        // We need to compute the message we want to sign and add it to the wallet as approved
        const action = asset
          .withWallet(wallets[2])
          .methods.unshield(wallets[0].getAddress(), wallets[1].getAddress(), amount, nonce);
        const expectedMessageHash = computeAuthWitMessageHash(
          wallets[2].getAddress(),
          wallets[0].getChainId(),
          wallets[0].getVersion(),
          action.request(),
        );

        // Both wallets are connected to same node and PXE so we could just insert directly
        // But doing it in two actions to show the flow.
        const witness = await wallets[0].createAuthWit({ caller: wallets[1].getAddress(), action });
        await wallets[2].addAuthWitness(witness);

        await expect(action.prove()).rejects.toThrow(
          `Unknown auth witness for message hash ${expectedMessageHash.toString()}`,
        );
      });

      it('unshield from blacklisted account', async () => {
        await expect(
          asset.methods.unshield(blacklisted.getAddress(), wallets[0].getAddress(), 1n, 0).prove(),
        ).rejects.toThrow("Assertion failed: Blacklisted: Sender '!from_roles.isBlacklisted'");
      });

      it('unshield to blacklisted account', async () => {
        await expect(
          asset.methods.unshield(wallets[0].getAddress(), blacklisted.getAddress(), 1n, 0).prove(),
        ).rejects.toThrow("Assertion failed: Blacklisted: Recipient '!to_roles.isBlacklisted'");
      });
    });
  });

  describe('Burn', () => {
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
        const txReplay = asset
          .withWallet(wallets[1])
          .methods.burn_public(wallets[0].getAddress(), amount, nonce)
          .send();
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
            "Assertion failed: Blacklisted: Sender '!from_roles.isBlacklisted'",
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

          await expect(action.prove()).rejects.toThrow(
            `Unknown auth witness for message hash ${messageHash.toString()}`,
          );
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
            "Assertion failed: Blacklisted: Sender '!from_roles.isBlacklisted'",
          );
        });
      });
    });
  });
});
