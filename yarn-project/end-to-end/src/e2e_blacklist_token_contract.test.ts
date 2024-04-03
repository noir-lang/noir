import {
  type AccountWallet,
  AztecAddress,
  type CheatCodes,
  type CompleteAddress,
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
import { openTmpStore } from '@aztec/kv-store/utils';
import { Pedersen, SparseTree, newTree } from '@aztec/merkle-tree';
import { SlowTreeContract, TokenBlacklistContract, type TokenContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { BITSIZE_TOO_BIG_ERROR, U128_OVERFLOW_ERROR, U128_UNDERFLOW_ERROR } from './fixtures/fixtures.js';
import { publicDeployAccounts, setup } from './fixtures/utils.js';
import { TokenSimulator } from './simulators/token_simulator.js';

const TIMEOUT = 120_000;

describe('e2e_blacklist_token_contract', () => {
  jest.setTimeout(TIMEOUT);

  let teardown: () => Promise<void>;
  let wallets: AccountWallet[];
  let accounts: CompleteAddress[];
  let logger: DebugLogger;

  let asset: TokenBlacklistContract;
  let slowTree: SlowTreeContract;

  let tokenSim: TokenSimulator;

  let slowUpdateTreeSimulator: SparseTree<Fr>;

  let cheatCodes: CheatCodes;

  const getMembershipProof = async (index: bigint, includeUncommitted: boolean) => {
    return {
      index,
      value: slowUpdateTreeSimulator.getLeafValue(index, includeUncommitted)!,
      // eslint-disable-next-line camelcase
      sibling_path: (await slowUpdateTreeSimulator.getSiblingPath(index, includeUncommitted)).toFields(),
    };
  };

  const getMembershipCapsule = (proof: { index: bigint; value: Fr; sibling_path: Fr[] }) => {
    return [new Fr(proof.index), proof.value, ...proof.sibling_path];
  };

  const getUpdateProof = async (newValue: bigint, index: bigint) => {
    const beforeProof = await getMembershipProof(index, false);
    const afterProof = await getMembershipProof(index, true);

    return {
      index,
      // eslint-disable-next-line camelcase
      new_value: newValue,
      // eslint-disable-next-line camelcase
      before: { value: beforeProof.value, sibling_path: beforeProof.sibling_path },
      // eslint-disable-next-line camelcase
      after: { value: afterProof.value, sibling_path: afterProof.sibling_path },
    };
  };

  const getUpdateCapsule = (proof: {
    index: bigint;
    new_value: bigint;
    before: { value: Fr; sibling_path: Fr[] };
    after: { value: Fr; sibling_path: Fr[] };
  }) => {
    return [
      new Fr(proof.index),
      new Fr(proof.new_value),
      proof.before.value,
      ...proof.before.sibling_path,
      proof.after.value,
      ...proof.after.sibling_path,
    ];
  };

  const addPendingShieldNoteToPXE = async (accountIndex: number, amount: bigint, secretHash: Fr, txHash: TxHash) => {
    const storageSlot = new Fr(4); // The storage slot of `pending_shields` is 4.
    const noteTypeId = new Fr(84114971101151129711410111011678111116101n); // TransparentNote
    const note = new Note([new Fr(amount), secretHash]);
    const extendedNote = new ExtendedNote(
      note,
      accounts[accountIndex].address,
      asset.address,
      storageSlot,
      noteTypeId,
      txHash,
    );
    await wallets[accountIndex].addNote(extendedNote);
  };

  const updateSlowTree = async (tree: SparseTree<Fr>, wallet: Wallet, index: AztecAddress, value: bigint) => {
    await wallet.addCapsule(getUpdateCapsule(await getUpdateProof(value, index.toBigInt())));
    await tree.updateLeaf(new Fr(value), index.toBigInt());
  };

  beforeAll(async () => {
    ({ teardown, logger, wallets, accounts, cheatCodes } = await setup(4));
    await publicDeployAccounts(wallets[0], accounts.slice(0, 3));

    slowTree = await SlowTreeContract.deploy(wallets[0]).send().deployed();

    const depth = 254;
    slowUpdateTreeSimulator = await newTree(SparseTree, openTmpStore(), new Pedersen(), 'test', Fr, depth);

    // Add account[0] as admin
    await updateSlowTree(slowUpdateTreeSimulator, wallets[0], accounts[0].address, 4n);

    const deployTx = TokenBlacklistContract.deploy(wallets[0], accounts[0], slowTree.address).send({});
    const receipt = await deployTx.wait();
    asset = receipt.contract;

    await asset.methods.init_slow_tree(accounts[0].address).send().wait();

    // Progress to next "epoch"
    const time = await cheatCodes.eth.timestamp();
    await cheatCodes.aztec.warp(time + 200);
    await slowUpdateTreeSimulator.commit();

    const roleLeaf = await slowTree.methods.un_read_leaf_at(asset.address, accounts[0].address).simulate();
    expect(roleLeaf['next_change']).toBeGreaterThan(0n);
    expect(roleLeaf['before']).toEqual(0n);
    expect(roleLeaf['after']).toEqual(4n);

    logger(`Token deployed to ${asset.address}`);
    tokenSim = new TokenSimulator(
      asset as unknown as TokenContract,
      logger,
      accounts.map(a => a.address),
    );

    asset.artifact.functions.forEach(fn => {
      logger(
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
    it('Extend account[0] roles with minter as admin', async () => {
      const newMinter = accounts[0].address;
      const newRoles = 2n + 4n;

      const beforeLeaf = await slowTree.methods.un_read_leaf_at(asset.address, newMinter).simulate();
      // eslint-disable-next-line camelcase
      expect(beforeLeaf['next_change']).toBeGreaterThan(0n);
      expect(beforeLeaf['before']).toEqual(0n);
      expect(beforeLeaf['after']).toEqual(4n);

      await updateSlowTree(slowUpdateTreeSimulator, wallets[0], newMinter, newRoles);
      await wallets[0].addCapsule(
        getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), false)),
      );

      await asset.methods.update_roles(newMinter, newRoles).send().wait();
      await slowUpdateTreeSimulator.commit();

      const afterLeaf = await slowTree.methods.un_read_leaf_at(asset.address, newMinter).simulate();
      expect(afterLeaf['next_change']).toBeGreaterThan(beforeLeaf['next_change']);
      expect(afterLeaf['before']).toEqual(4n);
      expect(afterLeaf['after']).toEqual(newRoles);

      const time = await cheatCodes.eth.timestamp();
      await cheatCodes.aztec.warp(time + 200);
    });

    it('Make account[1] admin', async () => {
      const newAdmin = accounts[1].address;
      const newRoles = 4n;

      let v = await slowTree.methods.un_read_leaf_at(asset.address, newAdmin).simulate();
      // eslint-disable-next-line camelcase
      expect(v).toEqual({ next_change: 0n, before: 0n, after: 0n });

      await updateSlowTree(slowUpdateTreeSimulator, wallets[0], newAdmin, newRoles);
      await wallets[0].addCapsule(
        getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), false)),
      );

      await asset.methods.update_roles(newAdmin, newRoles).send().wait();
      await slowUpdateTreeSimulator.commit();

      v = await slowTree.methods.un_read_leaf_at(asset.address, newAdmin).simulate();
      expect(v['next_change']).toBeGreaterThan(0n);
      expect(v['before']).toEqual(0n);
      expect(v['after']).toEqual(newRoles);

      // Progress to next "epoch"
      const time = await cheatCodes.eth.timestamp();
      await cheatCodes.aztec.warp(time + 200);
    });

    it('Revoke admin as admin', async () => {
      const actor = accounts[1].address;
      const newRoles = 0n;
      const currentRoles = 4n;

      const beforeLeaf = await slowTree.methods.un_read_leaf_at(asset.address, actor).simulate();
      // eslint-disable-next-line camelcase
      expect(beforeLeaf['next_change']).toBeGreaterThan(0n);
      expect(beforeLeaf['before']).toEqual(0n);
      expect(beforeLeaf['after']).toEqual(currentRoles);

      await updateSlowTree(slowUpdateTreeSimulator, wallets[0], actor, newRoles);
      await wallets[0].addCapsule(
        getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), false)),
      );

      await asset.methods.update_roles(actor, newRoles).send().wait();
      await slowUpdateTreeSimulator.commit();

      const afterLeaf = await slowTree.methods.un_read_leaf_at(asset.address, actor).simulate();
      expect(afterLeaf['next_change']).toBeGreaterThan(beforeLeaf['next_change']);
      expect(afterLeaf['before']).toEqual(currentRoles);
      expect(afterLeaf['after']).toEqual(newRoles);

      const time = await cheatCodes.eth.timestamp();
      await cheatCodes.aztec.warp(time + 200);
    });

    it('Add account[3] to blacklist', async () => {
      let v = await slowTree.methods.un_read_leaf_at(asset.address, accounts[3].address).simulate();
      // eslint-disable-next-line camelcase
      expect(v).toEqual({ next_change: 0n, before: 0n, after: 0n });

      await updateSlowTree(slowUpdateTreeSimulator, wallets[0], accounts[3].address, 1n);
      await wallets[0].addCapsule(
        getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), false)),
      );

      await asset.methods.update_roles(accounts[3].address, 1n).send().wait();
      await slowUpdateTreeSimulator.commit();

      v = await slowTree.methods.un_read_leaf_at(asset.address, accounts[3].address).simulate();
      expect(v['next_change']).toBeGreaterThan(0n);
      expect(v['before']).toEqual(0n);
      expect(v['after']).toEqual(1n);

      const time = await cheatCodes.eth.timestamp();
      await cheatCodes.aztec.warp(time + 200);
    });

    describe('failure cases', () => {
      it('Set admin (not admin)', async () => {
        const account = AztecAddress.random();
        const v = await slowTree.methods.un_read_leaf_at(asset.address, account).simulate();
        const newRoles = 4n;
        // eslint-disable-next-line camelcase
        expect(v).toEqual({ next_change: 0n, before: 0n, after: 0n });

        await wallets[3].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[3].address.toBigInt(), false)),
        );
        await expect(asset.withWallet(wallets[3]).methods.update_roles(account, newRoles).prove()).rejects.toThrow(
          "Assertion failed: caller is not admin 'caller_roles.is_admin'",
        );
      });

      it('Revoke minter not as admin', async () => {
        const adminAccount = accounts[0].address;
        const v = await slowTree.methods.un_read_leaf_at(asset.address, adminAccount).simulate();
        const newRoles = 0n;
        // eslint-disable-next-line camelcase
        expect(v['next_change']).toBeGreaterThan(0n);
        expect(v['before']).toEqual(4n);
        expect(v['after']).toEqual(6n);

        await wallets[3].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[3].address.toBigInt(), false)),
        );
        await expect(asset.withWallet(wallets[3]).methods.update_roles(adminAccount, newRoles).prove()).rejects.toThrow(
          "Assertion failed: caller is not admin 'caller_roles.is_admin'",
        );
      });
    });
  });

  describe('Minting', () => {
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
            asset.withWallet(wallets[1]).methods.mint_public(accounts[0].address, amount).prove(),
          ).rejects.toThrow('Assertion failed: caller is not minter');
        });

        it('mint >u128 tokens to overflow', async () => {
          const amount = 2n ** 128n; // U128::max() + 1;
          await expect(asset.methods.mint_public(accounts[0].address, amount).prove()).rejects.toThrow(
            BITSIZE_TOO_BIG_ERROR,
          );
        });

        it('mint <u128 but recipient balance >u128', async () => {
          const amount = 2n ** 128n - tokenSim.balanceOfPublic(accounts[0].address);
          await expect(asset.methods.mint_public(accounts[0].address, amount).prove()).rejects.toThrow(
            U128_OVERFLOW_ERROR,
          );
        });

        it('mint <u128 but such that total supply >u128', async () => {
          const amount = 2n ** 128n - tokenSim.balanceOfPublic(accounts[0].address);
          await expect(asset.methods.mint_public(accounts[1].address, amount).prove()).rejects.toThrow(
            U128_OVERFLOW_ERROR,
          );
        });

        it('mint to blacklisted entity', async () => {
          await expect(
            asset.withWallet(wallets[1]).methods.mint_public(accounts[3].address, 1n).prove(),
          ).rejects.toThrow("Assertion failed: Blacklisted: Recipient '!to_roles.is_blacklisted'");
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
          await wallets[0].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
          );
          const receiptClaim = await asset.methods
            .redeem_shield(accounts[0].address, amount, secret)
            .send()
            .wait({ debug: true });
          tokenSim.redeemShield(accounts[0].address, amount);
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
          await wallets[0].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
          );
          await expect(asset.methods.redeem_shield(accounts[0].address, amount, secret).prove()).rejects.toThrow(
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
          // @todo @LHerskind this one don't make sense. It fails because of total supply overflowing.
          const amount = 2n ** 128n - tokenSim.balanceOfPrivate(accounts[0].address);
          expect(amount).toBeLessThan(2n ** 128n);
          await expect(asset.methods.mint_private(amount, secretHash).prove()).rejects.toThrow(U128_OVERFLOW_ERROR);
        });

        it('mint <u128 but such that total supply >u128', async () => {
          const amount = 2n ** 128n - tokenSim.totalSupply;
          await expect(asset.methods.mint_private(amount, secretHash).prove()).rejects.toThrow(U128_OVERFLOW_ERROR);
        });

        it('mint and try to redeem at blacklist', async () => {
          await wallets[3].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[3].address.toBigInt(), true)),
          );
          await expect(asset.methods.redeem_shield(accounts[3].address, amount, secret).prove()).rejects.toThrow(
            "Assertion failed: Blacklisted: Recipient '!to_roles.is_blacklisted'",
          );
        });
      });
    });
  });

  describe('Transfer', () => {
    describe('public', () => {
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

        // Check that the message hash is no longer valid. Need to try to send since nullifiers are handled by sequencer.
        const txReplay = asset
          .withWallet(wallets[1])
          .methods.transfer_public(accounts[0].address, accounts[1].address, amount, nonce)
          .send();
        await expect(txReplay.wait()).rejects.toThrow('Transaction ');
      });

      describe('failure cases', () => {
        it('transfer more than balance', async () => {
          const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
          const amount = balance0 + 1n;
          const nonce = 0;
          await expect(
            asset.methods.transfer_public(accounts[0].address, accounts[1].address, amount, nonce).prove(),
          ).rejects.toThrow(U128_UNDERFLOW_ERROR);
        });

        it('transfer on behalf of self with non-zero nonce', async () => {
          const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
          const amount = balance0 - 1n;
          const nonce = 1;
          await expect(
            asset.methods.transfer_public(accounts[0].address, accounts[1].address, amount, nonce).prove(),
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
              .prove(),
          ).rejects.toThrow('Assertion failed: Message not authorized by account');
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

          // We need to compute the message we want to sign and add it to the wallet as approved
          await wallets[0].setPublicAuthWit({ caller: accounts[1].address, action }, true).send().wait();

          // Perform the transfer
          await expect(action.prove()).rejects.toThrow(U128_UNDERFLOW_ERROR);

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
          await expect(action.prove()).rejects.toThrow('Assertion failed: Message not authorized by account');

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
          await expect(action.prove()).rejects.toThrow('Assertion failed: Message not authorized by account');

          expect(await asset.methods.balance_of_public(accounts[0].address).simulate()).toEqual(balance0);
          expect(await asset.methods.balance_of_public(accounts[1].address).simulate()).toEqual(balance1);
        });

        it.skip('transfer into account to overflow', () => {
          // This should already be covered by the mint case earlier. e.g., since we cannot mint to overflow, there is not
          // a way to get funds enough to overflow.
          // Require direct storage manipulation for us to perform a nice explicit case though.
          // See https://github.com/AztecProtocol/aztec-packages/issues/1259
        });

        it('transfer from a blacklisted account', async () => {
          await expect(
            asset.methods.transfer_public(accounts[3].address, accounts[0].address, 1n, 0n).prove(),
          ).rejects.toThrow("Assertion failed: Blacklisted: Sender '!from_roles.is_blacklisted'");
        });

        it('transfer to a blacklisted account', async () => {
          await expect(
            asset.methods.transfer_public(accounts[0].address, accounts[3].address, 1n, 0n).prove(),
          ).rejects.toThrow("Assertion failed: Blacklisted: Recipient '!to_roles.is_blacklisted'");
        });
      });
    });

    describe('private', () => {
      it('transfer less than balance', async () => {
        const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
        const amount = balance0 / 2n;
        expect(amount).toBeGreaterThan(0n);
        await wallets[0].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[1].address.toBigInt(), true)),
        );
        await wallets[0].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
        );
        await asset.methods.transfer(accounts[0].address, accounts[1].address, amount, 0).send().wait();
        tokenSim.transferPrivate(accounts[0].address, accounts[1].address, amount);
      });

      it('transfer to self', async () => {
        const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
        const amount = balance0 / 2n;
        expect(amount).toBeGreaterThan(0n);
        await wallets[0].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
        );
        await wallets[0].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
        );
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
        // docs:start:authwit_computeAuthWitMessageHash
        const action = asset
          .withWallet(wallets[1])
          .methods.transfer(accounts[0].address, accounts[1].address, amount, nonce);
        // docs:end:authwit_computeAuthWitMessageHash

        const witness = await wallets[0].createAuthWit({ caller: accounts[1].address, action });
        await wallets[1].addAuthWitness(witness);
        // docs:end:authwit_transfer_example
        await wallets[1].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[1].address.toBigInt(), true)),
        );
        await wallets[1].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
        );

        // Perform the transfer
        await action.send().wait();
        tokenSim.transferPrivate(accounts[0].address, accounts[1].address, amount);

        await wallets[1].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[1].address.toBigInt(), true)),
        );
        await wallets[1].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
        );

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
          await wallets[0].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[1].address.toBigInt(), true)),
          );
          await wallets[0].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
          );
          await expect(
            asset.methods.transfer(accounts[0].address, accounts[1].address, amount, 0).prove(),
          ).rejects.toThrow('Assertion failed: Balance too low');
        });

        it('transfer on behalf of self with non-zero nonce', async () => {
          const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
          const amount = balance0 - 1n;
          expect(amount).toBeGreaterThan(0n);
          await wallets[0].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[1].address.toBigInt(), true)),
          );
          await wallets[0].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
          );
          await expect(
            asset.methods.transfer(accounts[0].address, accounts[1].address, amount, 1).prove(),
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

          // Both wallets are connected to same node and PXE so we could just insert directly
          // But doing it in two actions to show the flow.
          const witness = await wallets[0].createAuthWit({ caller: accounts[1].address, action });
          await wallets[1].addAuthWitness(witness);
          await wallets[1].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[1].address.toBigInt(), true)),
          );
          await wallets[1].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
          );

          // Perform the transfer
          await expect(action.prove()).rejects.toThrow('Assertion failed: Balance too low');
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
          await wallets[1].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[1].address.toBigInt(), true)),
          );
          await wallets[1].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
          );

          await expect(action.prove()).rejects.toThrow(
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
          await wallets[2].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[1].address.toBigInt(), true)),
          );
          await wallets[2].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
          );

          await expect(action.prove()).rejects.toThrow(
            `Unknown auth witness for message hash ${expectedMessageHash.toString()}`,
          );
          expect(await asset.methods.balance_of_private(accounts[0].address).simulate()).toEqual(balance0);
        });

        it('transfer from a blacklisted account', async () => {
          await wallets[3].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
          );
          await wallets[3].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[3].address.toBigInt(), true)),
          );
          await expect(asset.methods.transfer(accounts[3].address, accounts[0].address, 1n, 0).prove()).rejects.toThrow(
            "Assertion failed: Blacklisted: Sender '!from_roles.is_blacklisted'",
          );
        });

        it('transfer to a blacklisted account', async () => {
          await wallets[0].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[3].address.toBigInt(), true)),
          );
          await wallets[0].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
          );
          await expect(asset.methods.transfer(accounts[0].address, accounts[3].address, 1n, 0).prove()).rejects.toThrow(
            "Assertion failed: Blacklisted: Recipient '!to_roles.is_blacklisted'",
          );
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
      const balancePub = await asset.methods.balance_of_public(accounts[0].address).simulate();
      const amount = balancePub / 2n;
      expect(amount).toBeGreaterThan(0n);

      const receipt = await asset.methods.shield(accounts[0].address, amount, secretHash, 0).send().wait();

      tokenSim.shield(accounts[0].address, amount);
      await tokenSim.check();

      // Redeem it
      await addPendingShieldNoteToPXE(0, amount, secretHash, receipt.txHash);
      await wallets[0].addCapsule(getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)));
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
      const txReplay = asset
        .withWallet(wallets[1])
        .methods.shield(accounts[0].address, amount, secretHash, nonce)
        .send();
      await expect(txReplay.wait()).rejects.toThrow('Transaction ');

      // Redeem it
      await addPendingShieldNoteToPXE(0, amount, secretHash, receipt.txHash);
      await wallets[0].addCapsule(getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)));
      await asset.methods.redeem_shield(accounts[0].address, amount, secret).send().wait();

      tokenSim.redeemShield(accounts[0].address, amount);
    });

    describe('failure cases', () => {
      it('on behalf of self (more than balance)', async () => {
        const balancePub = await asset.methods.balance_of_public(accounts[0].address).simulate();
        const amount = balancePub + 1n;
        expect(amount).toBeGreaterThan(0n);

        await expect(asset.methods.shield(accounts[0].address, amount, secretHash, 0).prove()).rejects.toThrow(
          U128_UNDERFLOW_ERROR,
        );
      });

      it('on behalf of self (invalid nonce)', async () => {
        const balancePub = await asset.methods.balance_of_public(accounts[0].address).simulate();
        const amount = balancePub + 1n;
        expect(amount).toBeGreaterThan(0n);

        await expect(asset.methods.shield(accounts[0].address, amount, secretHash, 1).prove()).rejects.toThrow(
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

        await expect(action.prove()).rejects.toThrow(U128_UNDERFLOW_ERROR);
      });

      it('on behalf of other (wrong designated caller)', async () => {
        const balancePub = await asset.methods.balance_of_public(accounts[0].address).simulate();
        const amount = balancePub + 1n;
        const nonce = Fr.random();
        expect(amount).toBeGreaterThan(0n);

        // We need to compute the message we want to sign and add it to the wallet as approved
        const action = asset.withWallet(wallets[2]).methods.shield(accounts[0].address, amount, secretHash, nonce);
        await wallets[0].setPublicAuthWit({ caller: accounts[1].address, action }, true).send().wait();

        await expect(action.prove()).rejects.toThrow('Assertion failed: Message not authorized by account');
      });

      it('on behalf of other (without approval)', async () => {
        const balance = await asset.methods.balance_of_public(accounts[0].address).simulate();
        const amount = balance / 2n;
        const nonce = Fr.random();
        expect(amount).toBeGreaterThan(0n);

        await expect(
          asset.withWallet(wallets[1]).methods.shield(accounts[0].address, amount, secretHash, nonce).prove(),
        ).rejects.toThrow(`Assertion failed: Message not authorized by account`);
      });

      it('shielding from blacklisted account', async () => {
        await wallets[0].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[3].address.toBigInt(), true)),
        );
        await expect(
          asset.withWallet(wallets[3]).methods.shield(accounts[3].address, 1n, secretHash, 0).prove(),
        ).rejects.toThrow("Assertion failed: Blacklisted: Sender '!from_roles.is_blacklisted'");
      });
    });
  });

  describe('Unshielding', () => {
    it('on behalf of self', async () => {
      const balancePriv = await asset.methods.balance_of_private(accounts[0].address).simulate();
      const amount = balancePriv / 2n;
      expect(amount).toBeGreaterThan(0n);

      await wallets[1].addCapsule(getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)));
      await wallets[1].addCapsule(getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)));
      await asset.methods.unshield(accounts[0].address, accounts[0].address, amount, 0).send().wait();

      tokenSim.unshield(accounts[0].address, accounts[0].address, amount);
    });

    it('on behalf of other', async () => {
      const balancePriv0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
      const amount = balancePriv0 / 2n;
      const nonce = Fr.random();
      expect(amount).toBeGreaterThan(0n);

      // We need to compute the message we want to sign and add it to the wallet as approved
      await wallets[1].addCapsule(getMembershipCapsule(await getMembershipProof(accounts[1].address.toBigInt(), true)));
      await wallets[1].addCapsule(getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)));
      const action = asset
        .withWallet(wallets[1])
        .methods.unshield(accounts[0].address, accounts[1].address, amount, nonce);

      // Both wallets are connected to same node and PXE so we could just insert directly
      // But doing it in two actions to show the flow.
      const witness = await wallets[0].createAuthWit({ caller: accounts[1].address, action });
      await wallets[1].addAuthWitness(witness);

      await action.send().wait();
      tokenSim.unshield(accounts[0].address, accounts[1].address, amount);

      // Perform the transfer again, should fail
      await wallets[1].addCapsule(getMembershipCapsule(await getMembershipProof(accounts[1].address.toBigInt(), true)));
      await wallets[1].addCapsule(getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)));
      const txReplay = asset
        .withWallet(wallets[1])
        .methods.unshield(accounts[0].address, accounts[1].address, amount, nonce)
        .send();
      await expect(txReplay.wait()).rejects.toThrow('Transaction ');
      // @todo @LHerskind This error is weird?
    });

    describe('failure cases', () => {
      it('on behalf of self (more than balance)', async () => {
        const balancePriv = await asset.methods.balance_of_private(accounts[0].address).simulate();
        const amount = balancePriv + 1n;
        expect(amount).toBeGreaterThan(0n);

        await wallets[0].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
        );
        await wallets[0].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
        );

        await expect(
          asset.methods.unshield(accounts[0].address, accounts[0].address, amount, 0).prove(),
        ).rejects.toThrow('Assertion failed: Balance too low');
      });

      it('on behalf of self (invalid nonce)', async () => {
        const balancePriv = await asset.methods.balance_of_private(accounts[0].address).simulate();
        const amount = balancePriv + 1n;
        expect(amount).toBeGreaterThan(0n);

        await wallets[0].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
        );
        await wallets[0].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
        );

        await expect(
          asset.methods.unshield(accounts[0].address, accounts[0].address, amount, 1).prove(),
        ).rejects.toThrow('Assertion failed: invalid nonce');
      });

      it('on behalf of other (more than balance)', async () => {
        const balancePriv0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
        const amount = balancePriv0 + 2n;
        const nonce = Fr.random();
        expect(amount).toBeGreaterThan(0n);

        // We need to compute the message we want to sign and add it to the wallet as approved
        const action = asset
          .withWallet(wallets[1])
          .methods.unshield(accounts[0].address, accounts[1].address, amount, nonce);

        // Both wallets are connected to same node and PXE so we could just insert directly
        // But doing it in two actions to show the flow.
        const witness = await wallets[0].createAuthWit({ caller: accounts[1].address, action });
        await wallets[1].addAuthWitness(witness);
        await wallets[1].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[1].address.toBigInt(), true)),
        );
        await wallets[1].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
        );

        await expect(action.prove()).rejects.toThrow('Assertion failed: Balance too low');
      });

      it('on behalf of other (invalid designated caller)', async () => {
        const balancePriv0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
        const amount = balancePriv0 + 2n;
        const nonce = Fr.random();
        expect(amount).toBeGreaterThan(0n);

        // We need to compute the message we want to sign and add it to the wallet as approved
        const action = asset
          .withWallet(wallets[2])
          .methods.unshield(accounts[0].address, accounts[1].address, amount, nonce);
        const expectedMessageHash = computeAuthWitMessageHash(
          accounts[2].address,
          wallets[0].getChainId(),
          wallets[0].getVersion(),
          action.request(),
        );

        // Both wallets are connected to same node and PXE so we could just insert directly
        // But doing it in two actions to show the flow.
        const witness = await wallets[0].createAuthWit({ caller: accounts[1].address, action });
        await wallets[2].addAuthWitness(witness);
        await wallets[2].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[1].address.toBigInt(), true)),
        );
        await wallets[2].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
        );

        await expect(action.prove()).rejects.toThrow(
          `Unknown auth witness for message hash ${expectedMessageHash.toString()}`,
        );
      });

      it('unshield from blacklisted account', async () => {
        await wallets[0].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
        );
        await wallets[0].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[3].address.toBigInt(), true)),
        );
        await expect(asset.methods.unshield(accounts[3].address, accounts[0].address, 1n, 0).prove()).rejects.toThrow(
          "Assertion failed: Blacklisted: Sender '!from_roles.is_blacklisted'",
        );
      });

      it('unshield to blacklisted account', async () => {
        await wallets[0].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[3].address.toBigInt(), true)),
        );
        await wallets[0].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
        );
        await expect(asset.methods.unshield(accounts[0].address, accounts[3].address, 1n, 0).prove()).rejects.toThrow(
          "Assertion failed: Blacklisted: Recipient '!to_roles.is_blacklisted'",
        );
      });
    });
  });

  describe('Burn', () => {
    describe('public', () => {
      it('burn less than balance', async () => {
        const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
        const amount = balance0 / 2n;
        expect(amount).toBeGreaterThan(0n);
        await asset.methods.burn_public(accounts[0].address, amount, 0).send().wait();

        tokenSim.burnPublic(accounts[0].address, amount);
      });

      it('burn on behalf of other', async () => {
        const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
        const amount = balance0 / 2n;
        expect(amount).toBeGreaterThan(0n);
        const nonce = Fr.random();

        // We need to compute the message we want to sign and add it to the wallet as approved
        const action = asset.withWallet(wallets[1]).methods.burn_public(accounts[0].address, amount, nonce);
        await wallets[0].setPublicAuthWit({ caller: accounts[1].address, action }, true).send().wait();

        await action.send().wait();

        tokenSim.burnPublic(accounts[0].address, amount);

        // Check that the message hash is no longer valid. Need to try to send since nullifiers are handled by sequencer.
        const txReplay = asset.withWallet(wallets[1]).methods.burn_public(accounts[0].address, amount, nonce).send();
        await expect(txReplay.wait()).rejects.toThrow('Transaction ');
      });

      describe('failure cases', () => {
        it('burn more than balance', async () => {
          const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
          const amount = balance0 + 1n;
          const nonce = 0;
          await expect(asset.methods.burn_public(accounts[0].address, amount, nonce).prove()).rejects.toThrow(
            U128_UNDERFLOW_ERROR,
          );
        });

        it('burn on behalf of self with non-zero nonce', async () => {
          const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
          const amount = balance0 - 1n;
          expect(amount).toBeGreaterThan(0n);
          const nonce = 1;
          await expect(asset.methods.burn_public(accounts[0].address, amount, nonce).prove()).rejects.toThrow(
            'Assertion failed: invalid nonce',
          );
        });

        it('burn on behalf of other without "approval"', async () => {
          const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
          const amount = balance0 + 1n;
          const nonce = Fr.random();
          await expect(
            asset.withWallet(wallets[1]).methods.burn_public(accounts[0].address, amount, nonce).prove(),
          ).rejects.toThrow('Assertion failed: Message not authorized by account');
        });

        it('burn more than balance on behalf of other', async () => {
          const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
          const amount = balance0 + 1n;
          const nonce = Fr.random();
          expect(amount).toBeGreaterThan(0n);

          // We need to compute the message we want to sign and add it to the wallet as approved
          const action = asset.withWallet(wallets[1]).methods.burn_public(accounts[0].address, amount, nonce);
          await wallets[0].setPublicAuthWit({ caller: accounts[1].address, action }, true).send().wait();

          await expect(action.prove()).rejects.toThrow(U128_UNDERFLOW_ERROR);
        });

        it('burn on behalf of other, wrong designated caller', async () => {
          const balance0 = await asset.methods.balance_of_public(accounts[0].address).simulate();
          const amount = balance0 + 2n;
          const nonce = Fr.random();
          expect(amount).toBeGreaterThan(0n);

          // We need to compute the message we want to sign and add it to the wallet as approved
          const action = asset.withWallet(wallets[1]).methods.burn_public(accounts[0].address, amount, nonce);
          await wallets[0].setPublicAuthWit({ caller: accounts[0].address, action }, true).send().wait();

          await expect(
            asset.withWallet(wallets[1]).methods.burn_public(accounts[0].address, amount, nonce).prove(),
          ).rejects.toThrow('Assertion failed: Message not authorized by account');
        });

        it('burn from blacklisted account', async () => {
          await expect(asset.methods.burn_public(accounts[3].address, 1n, 0).prove()).rejects.toThrow(
            "Assertion failed: Blacklisted: Sender '!from_roles.is_blacklisted'",
          );
        });
      });
    });

    describe('private', () => {
      it('burn less than balance', async () => {
        const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
        const amount = balance0 / 2n;
        expect(amount).toBeGreaterThan(0n);
        await wallets[0].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
        );
        await asset.methods.burn(accounts[0].address, amount, 0).send().wait();
        tokenSim.burnPrivate(accounts[0].address, amount);
      });

      it('burn on behalf of other', async () => {
        const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
        const amount = balance0 / 2n;
        const nonce = Fr.random();
        expect(amount).toBeGreaterThan(0n);

        // We need to compute the message we want to sign and add it to the wallet as approved
        const action = asset.withWallet(wallets[1]).methods.burn(accounts[0].address, amount, nonce);

        // Both wallets are connected to same node and PXE so we could just insert directly
        // But doing it in two actions to show the flow.
        const witness = await wallets[0].createAuthWit({ caller: accounts[1].address, action });
        await wallets[1].addAuthWitness(witness);
        await wallets[1].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
        );

        await asset.withWallet(wallets[1]).methods.burn(accounts[0].address, amount, nonce).send().wait();
        tokenSim.burnPrivate(accounts[0].address, amount);

        // Perform the transfer again, should fail
        await wallets[1].addCapsule(
          getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
        );
        const txReplay = asset.withWallet(wallets[1]).methods.burn(accounts[0].address, amount, nonce).send();
        await expect(txReplay.wait()).rejects.toThrow('Transaction ');
      });

      describe('failure cases', () => {
        it('burn more than balance', async () => {
          const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
          const amount = balance0 + 1n;
          expect(amount).toBeGreaterThan(0n);
          await wallets[0].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
          );
          await expect(asset.methods.burn(accounts[0].address, amount, 0).prove()).rejects.toThrow(
            'Assertion failed: Balance too low',
          );
        });

        it('burn on behalf of self with non-zero nonce', async () => {
          const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
          const amount = balance0 - 1n;
          expect(amount).toBeGreaterThan(0n);
          await wallets[0].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
          );
          await expect(asset.methods.burn(accounts[0].address, amount, 1).prove()).rejects.toThrow(
            'Assertion failed: invalid nonce',
          );
        });

        it('burn more than balance on behalf of other', async () => {
          const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
          const amount = balance0 + 1n;
          const nonce = Fr.random();
          expect(amount).toBeGreaterThan(0n);

          // We need to compute the message we want to sign and add it to the wallet as approved
          const action = asset.withWallet(wallets[1]).methods.burn(accounts[0].address, amount, nonce);

          // Both wallets are connected to same node and PXE so we could just insert directly
          // But doing it in two actions to show the flow.
          const witness = await wallets[0].createAuthWit({ caller: accounts[1].address, action });
          await wallets[1].addAuthWitness(witness);
          await wallets[1].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
          );

          await expect(action.prove()).rejects.toThrow('Assertion failed: Balance too low');
        });

        it('burn on behalf of other without approval', async () => {
          const balance0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
          const amount = balance0 / 2n;
          const nonce = Fr.random();
          expect(amount).toBeGreaterThan(0n);

          // We need to compute the message we want to sign and add it to the wallet as approved
          await wallets[1].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
          );
          const action = asset.withWallet(wallets[1]).methods.burn(accounts[0].address, amount, nonce);
          const messageHash = computeAuthWitMessageHash(
            accounts[1].address,
            wallets[0].getChainId(),
            wallets[0].getVersion(),
            action.request(),
          );

          await expect(action.prove()).rejects.toThrow(
            `Unknown auth witness for message hash ${messageHash.toString()}`,
          );
        });

        it('on behalf of other (invalid designated caller)', async () => {
          const balancePriv0 = await asset.methods.balance_of_private(accounts[0].address).simulate();
          const amount = balancePriv0 + 2n;
          const nonce = Fr.random();
          expect(amount).toBeGreaterThan(0n);

          // We need to compute the message we want to sign and add it to the wallet as approved
          await wallets[2].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[0].address.toBigInt(), true)),
          );
          const action = asset.withWallet(wallets[2]).methods.burn(accounts[0].address, amount, nonce);
          const expectedMessageHash = computeAuthWitMessageHash(
            accounts[2].address,
            wallets[0].getChainId(),
            wallets[0].getVersion(),
            action.request(),
          );

          const witness = await wallets[0].createAuthWit({ caller: accounts[1].address, action });
          await wallets[2].addAuthWitness(witness);

          await expect(action.prove()).rejects.toThrow(
            `Unknown auth witness for message hash ${expectedMessageHash.toString()}`,
          );
        });

        it('burn from blacklisted account', async () => {
          await wallets[0].addCapsule(
            getMembershipCapsule(await getMembershipProof(accounts[3].address.toBigInt(), true)),
          );
          await expect(asset.methods.burn(accounts[3].address, 1n, 0).prove()).rejects.toThrow(
            "Assertion failed: Blacklisted: Sender '!from_roles.is_blacklisted'",
          );
        });
      });
    });
  });
});
