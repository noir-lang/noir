import {
  type AccountWallet,
  type CheatCodes,
  type DebugLogger,
  type DeployL1Contracts,
  ExtendedNote,
  Fr,
  Note,
  computeSecretHash,
} from '@aztec/aztec.js';
import { RollupAbi } from '@aztec/l1-artifacts';
import { LendingContract, PriceFeedContract, TokenContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';
import { getContract } from 'viem';

import { publicDeployAccounts, setup } from './fixtures/utils.js';
import { LendingAccount, LendingSimulator, TokenSimulator } from './simulators/index.js';

describe('e2e_lending_contract', () => {
  jest.setTimeout(100_000);
  let wallet: AccountWallet;
  let deployL1ContractsValues: DeployL1Contracts;

  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let cc: CheatCodes;
  const SLOT_JUMP = 10;

  let lendingContract: LendingContract;
  let priceFeedContract: PriceFeedContract;
  let collateralAsset: TokenContract;
  let stableCoin: TokenContract;

  let lendingAccount: LendingAccount;
  let lendingSim: LendingSimulator;

  const deployContracts = async () => {
    logger.info(`Deploying price feed contract...`);
    const priceFeedContract = await PriceFeedContract.deploy(wallet).send().deployed();
    logger.info(`Price feed deployed to ${priceFeedContract.address}`);

    logger.info(`Deploying collateral asset feed contract...`);
    const collateralAsset = await TokenContract.deploy(wallet, wallet.getAddress(), 'TokenName', 'TokenSymbol', 18)
      .send()
      .deployed();
    logger.info(`Collateral asset deployed to ${collateralAsset.address}`);

    logger.info(`Deploying stable coin contract...`);
    const stableCoin = await TokenContract.deploy(wallet, wallet.getAddress(), 'TokenName', 'TokenSymbol', 18)
      .send()
      .deployed();
    logger.info(`Stable coin asset deployed to ${stableCoin.address}`);

    logger.info(`Deploying L2 public contract...`);
    const lendingContract = await LendingContract.deploy(wallet).send().deployed();
    logger.info(`CDP deployed at ${lendingContract.address}`);

    await collateralAsset.methods.set_minter(lendingContract.address, true).send().wait();
    await stableCoin.methods.set_minter(lendingContract.address, true).send().wait();

    return { priceFeedContract, lendingContract, collateralAsset, stableCoin };
  };

  beforeAll(async () => {
    ({ teardown, logger, cheatCodes: cc, wallet, deployL1ContractsValues } = await setup(1));
    ({ lendingContract, priceFeedContract, collateralAsset, stableCoin } = await deployContracts());
    await publicDeployAccounts(wallet, [wallet]);

    const rollup = getContract({
      address: deployL1ContractsValues.l1ContractAddresses.rollupAddress.toString(),
      abi: RollupAbi,
      client: deployL1ContractsValues.publicClient,
    });

    lendingAccount = new LendingAccount(wallet.getAddress(), new Fr(42));

    // Also specified in `noir-contracts/contracts/lending_contract/src/main.nr`
    const rate = 1268391679n;
    lendingSim = new LendingSimulator(
      cc,
      lendingAccount,
      rate,
      rollup,
      lendingContract,
      new TokenSimulator(collateralAsset, wallet, logger, [lendingContract.address, wallet.getAddress()]),
      new TokenSimulator(stableCoin, wallet, logger, [lendingContract.address, wallet.getAddress()]),
    );
  }, 300_000);

  afterAll(() => teardown());

  afterEach(async () => {
    await lendingSim.check();
  });

  it('Mint assets for later usage', async () => {
    await priceFeedContract.methods
      .set_price(0n, 2n * 10n ** 9n)
      .send()
      .wait();

    {
      const assets = [collateralAsset, stableCoin];
      const mintAmount = 10000n;
      for (const asset of assets) {
        const secret = Fr.random();
        const secretHash = computeSecretHash(secret);

        const a = asset.methods.mint_public(lendingAccount.address, mintAmount).send();
        const b = asset.methods.mint_private(mintAmount, secretHash).send();
        await Promise.all([a, b].map(tx => tx.wait()));

        const note = new Note([new Fr(mintAmount), secretHash]);
        const txHash = await b.getTxHash();
        const extendedNote = new ExtendedNote(
          note,
          wallet.getAddress(),
          asset.address,
          TokenContract.storage.pending_shields.slot,
          TokenContract.notes.TransparentNote.id,
          txHash,
        );
        await wallet.addNote(extendedNote);

        await asset.methods.redeem_shield(lendingAccount.address, mintAmount, secret).send().wait();
      }
    }

    lendingSim.mintStableCoinOutsideLoan(lendingAccount.address, 10000n, true);
    lendingSim.stableCoin.redeemShield(lendingAccount.address, 10000n);
    lendingSim.mintStableCoinOutsideLoan(lendingAccount.address, 10000n, false);

    lendingSim.collateralAsset.mintPrivate(10000n);
    lendingSim.collateralAsset.redeemShield(lendingAccount.address, 10000n);
    lendingSim.collateralAsset.mintPublic(lendingAccount.address, 10000n);
  });

  it('Initialize the contract', async () => {
    await lendingSim.prepare();
    logger.info('Initializing contract');
    await lendingContract.methods
      .init(priceFeedContract.address, 8000, collateralAsset.address, stableCoin.address)
      .send()
      .wait();
  });

  describe('Deposits', () => {
    it('Depositing 🥸 : 💰 -> 🏦', async () => {
      const depositAmount = 420n;
      const nonce = Fr.random();
      await wallet.createAuthWit({
        caller: lendingContract.address,
        action: collateralAsset.methods.unshield(lendingAccount.address, lendingContract.address, depositAmount, nonce),
      });
      await lendingSim.progressSlots(SLOT_JUMP);
      lendingSim.depositPrivate(lendingAccount.address, lendingAccount.key(), depositAmount);

      // Make a private deposit of funds into own account.
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - increase the private collateral.
      logger.info('Depositing 🥸 : 💰 -> 🏦');
      await lendingContract.methods
        .deposit_private(
          lendingAccount.address,
          depositAmount,
          nonce,
          lendingAccount.secret,
          0n,
          collateralAsset.address,
        )
        .send()
        .wait();
    });

    it('Depositing 🥸 on behalf of recipient: 💰 -> 🏦', async () => {
      const depositAmount = 421n;
      const nonce = Fr.random();
      await wallet.createAuthWit({
        caller: lendingContract.address,
        action: collateralAsset.methods.unshield(lendingAccount.address, lendingContract.address, depositAmount, nonce),
      });

      await lendingSim.progressSlots(SLOT_JUMP);
      lendingSim.depositPrivate(lendingAccount.address, lendingAccount.address.toField(), depositAmount);
      // Make a private deposit of funds into another account, in this case, a public account.
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - increase the public collateral.
      logger.info('Depositing 🥸 on behalf of recipient: 💰 -> 🏦');
      await lendingContract.methods
        .deposit_private(
          lendingAccount.address,
          depositAmount,
          nonce,
          0n,
          lendingAccount.address,
          collateralAsset.address,
        )
        .send()
        .wait();
    });

    it('Depositing: 💰 -> 🏦', async () => {
      const depositAmount = 211n;

      const nonce = Fr.random();

      // Add it to the wallet as approved
      await wallet
        .setPublicAuthWit(
          {
            caller: lendingContract.address,
            action: collateralAsset.methods.transfer_public(
              lendingAccount.address,
              lendingContract.address,
              depositAmount,
              nonce,
            ),
          },
          true,
        )
        .send()
        .wait();

      await lendingSim.progressSlots(SLOT_JUMP);
      lendingSim.depositPublic(lendingAccount.address, lendingAccount.address.toField(), depositAmount);

      // Make a public deposit of funds into self.
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - increase the public collateral.

      logger.info('Depositing: 💰 -> 🏦');
      await lendingContract.methods
        .deposit_public(depositAmount, nonce, lendingAccount.address, collateralAsset.address)
        .send()
        .wait();
    });
  });

  describe('Borrow', () => {
    it('Borrow 🥸 : 🏦 -> 🍌', async () => {
      const borrowAmount = 69n;
      await lendingSim.progressSlots(SLOT_JUMP);
      lendingSim.borrow(lendingAccount.key(), lendingAccount.address, borrowAmount);

      // Make a private borrow using the private account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - increase the private debt.

      logger.info('Borrow 🥸 : 🏦 -> 🍌');
      await lendingContract.methods
        .borrow_private(lendingAccount.secret, lendingAccount.address, borrowAmount)
        .send()
        .wait();
    });

    it('Borrow: 🏦 -> 🍌', async () => {
      const borrowAmount = 69n;
      await lendingSim.progressSlots(SLOT_JUMP);
      lendingSim.borrow(lendingAccount.address.toField(), lendingAccount.address, borrowAmount);

      // Make a public borrow using the private account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - increase the public debt.

      logger.info('Borrow: 🏦 -> 🍌');
      await lendingContract.methods.borrow_public(lendingAccount.address, borrowAmount).send().wait();
    });
  });

  describe('Repay', () => {
    it('Repay 🥸 : 🍌 -> 🏦', async () => {
      const repayAmount = 20n;
      const nonce = Fr.random();
      await wallet.createAuthWit({
        caller: lendingContract.address,
        action: stableCoin.methods.burn(lendingAccount.address, repayAmount, nonce),
      });

      await lendingSim.progressSlots(SLOT_JUMP);
      lendingSim.repayPrivate(lendingAccount.address, lendingAccount.key(), repayAmount);

      // Make a private repay of the debt in the private account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - decrease the private debt.

      logger.info('Repay 🥸 : 🍌 -> 🏦');
      await lendingContract.methods
        .repay_private(lendingAccount.address, repayAmount, nonce, lendingAccount.secret, 0n, stableCoin.address)
        .send()
        .wait();
    });

    it('Repay 🥸  on behalf of public: 🍌 -> 🏦', async () => {
      const repayAmount = 21n;
      const nonce = Fr.random();
      await wallet.createAuthWit({
        caller: lendingContract.address,
        action: stableCoin.methods.burn(lendingAccount.address, repayAmount, nonce),
      });

      await lendingSim.progressSlots(SLOT_JUMP);
      lendingSim.repayPrivate(lendingAccount.address, lendingAccount.address.toField(), repayAmount);

      // Make a private repay of the debt in the public account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - decrease the public debt.

      logger.info('Repay 🥸  on behalf of public: 🍌 -> 🏦');
      await lendingContract.methods
        .repay_private(lendingAccount.address, repayAmount, nonce, 0n, lendingAccount.address, stableCoin.address)
        .send()
        .wait();
    });

    it('Repay: 🍌 -> 🏦', async () => {
      const repayAmount = 20n;
      const nonce = Fr.random();

      // Add it to the wallet as approved
      await wallet
        .setPublicAuthWit(
          {
            caller: lendingContract.address,
            action: stableCoin.methods.burn_public(lendingAccount.address, repayAmount, nonce).request(),
          },
          true,
        )
        .send()
        .wait();

      await lendingSim.progressSlots(SLOT_JUMP);
      lendingSim.repayPublic(lendingAccount.address, lendingAccount.address.toField(), repayAmount);

      // Make a public repay of the debt in the public account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - decrease the public debt.

      logger.info('Repay: 🍌 -> 🏦');
      await lendingContract.methods
        .repay_public(repayAmount, nonce, lendingAccount.address, stableCoin.address)
        .send()
        .wait();
    });
  });

  describe('Withdraw', () => {
    it('Withdraw: 🏦 -> 💰', async () => {
      const withdrawAmount = 42n;
      await lendingSim.progressSlots(SLOT_JUMP);
      lendingSim.withdraw(lendingAccount.address.toField(), lendingAccount.address, withdrawAmount);

      // Withdraw funds from the public account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - decrease the public collateral.

      logger.info('Withdraw: 🏦 -> 💰');
      await lendingContract.methods.withdraw_public(lendingAccount.address, withdrawAmount).send().wait();
    });

    it('Withdraw 🥸 : 🏦 -> 💰', async () => {
      const withdrawAmount = 42n;
      await lendingSim.progressSlots(SLOT_JUMP);
      lendingSim.withdraw(lendingAccount.key(), lendingAccount.address, withdrawAmount);

      // Withdraw funds from the private account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - decrease the private collateral.

      logger.info('Withdraw 🥸 : 🏦 -> 💰');
      await lendingContract.methods
        .withdraw_private(lendingAccount.secret, lendingAccount.address, withdrawAmount)
        .send()
        .wait();
    });

    describe('failure cases', () => {
      it('withdraw more than possible to revert', async () => {
        // Withdraw more than possible to test the revert.
        logger.info('Withdraw: trying to withdraw more than possible');
        await expect(
          lendingContract.methods.withdraw_public(lendingAccount.address, 10n ** 9n).prove(),
        ).rejects.toThrow();
      });
    });
  });
});
