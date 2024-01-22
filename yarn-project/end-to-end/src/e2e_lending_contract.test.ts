import {
  AccountWallet,
  CheatCodes,
  CompleteAddress,
  DebugLogger,
  ExtendedNote,
  Fr,
  Note,
  SentTx,
  TxStatus,
  computeAuthWitMessageHash,
  computeMessageSecretHash,
} from '@aztec/aztec.js';
import { LendingContract, PriceFeedContract, TokenContract } from '@aztec/noir-contracts';

import { jest } from '@jest/globals';

import { setup } from './fixtures/utils.js';
import { LendingAccount, LendingSimulator, TokenSimulator } from './simulators/index.js';

describe('e2e_lending_contract', () => {
  jest.setTimeout(100_000);
  let wallet: AccountWallet;
  let accounts: CompleteAddress[];
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let cc: CheatCodes;
  const TIME_JUMP = 100;

  let lendingContract: LendingContract;
  let priceFeedContract: PriceFeedContract;
  let collateralAsset: TokenContract;
  let stableCoin: TokenContract;

  let lendingAccount: LendingAccount;
  let lendingSim: LendingSimulator;

  const waitForSuccess = async (tx: SentTx) => {
    const receipt = await tx.wait();
    expect(receipt.status).toBe(TxStatus.MINED);
    return receipt;
  };

  const deployContracts = async () => {
    let lendingContract: LendingContract;
    let priceFeedContract: PriceFeedContract;

    let collateralAsset: TokenContract;
    let stableCoin: TokenContract;

    {
      logger(`Deploying price feed contract...`);
      const receipt = await waitForSuccess(PriceFeedContract.deploy(wallet).send());
      logger(`Price feed deployed to ${receipt.contractAddress}`);
      priceFeedContract = await PriceFeedContract.at(receipt.contractAddress!, wallet);
    }

    {
      logger(`Deploying collateral asset feed contract...`);
      const receipt = await waitForSuccess(
        TokenContract.deploy(wallet, accounts[0], 'TokenName', 'TokenSymbol', 18).send(),
      );
      logger(`Collateral asset deployed to ${receipt.contractAddress}`);
      collateralAsset = await TokenContract.at(receipt.contractAddress!, wallet);
    }

    {
      logger(`Deploying stable coin contract...`);
      const receipt = await waitForSuccess(
        TokenContract.deploy(wallet, accounts[0], 'TokenName', 'TokenSymbol', 18).send(),
      );
      logger(`Stable coin asset deployed to ${receipt.contractAddress}`);
      stableCoin = await TokenContract.at(receipt.contractAddress!, wallet);
    }

    {
      logger(`Deploying L2 public contract...`);
      const receipt = await waitForSuccess(LendingContract.deploy(wallet).send());
      logger(`CDP deployed at ${receipt.contractAddress}`);
      lendingContract = await LendingContract.at(receipt.contractAddress!, wallet);
    }

    await waitForSuccess(collateralAsset.methods.set_minter(lendingContract.address, true).send());
    await waitForSuccess(stableCoin.methods.set_minter(lendingContract.address, true).send());

    return { priceFeedContract, lendingContract, collateralAsset, stableCoin };
  };

  beforeAll(async () => {
    ({ teardown, logger, cheatCodes: cc, wallet, accounts } = await setup(1));
    ({ lendingContract, priceFeedContract, collateralAsset, stableCoin } = await deployContracts());

    lendingAccount = new LendingAccount(accounts[0].address, new Fr(42));

    // Also specified in `noir-contracts/contracts/lending_contract/src/main.nr`
    const rate = 1268391679n;
    lendingSim = new LendingSimulator(
      cc,
      lendingAccount,
      rate,
      lendingContract,
      new TokenSimulator(collateralAsset, logger, [lendingContract.address, ...accounts.map(a => a.address)]),
      new TokenSimulator(stableCoin, logger, [lendingContract.address, ...accounts.map(a => a.address)]),
    );
  }, 200_000);

  afterAll(() => teardown());

  afterEach(async () => {
    await lendingSim.check();
  });

  it('Mint assets for later usage', async () => {
    await waitForSuccess(priceFeedContract.methods.set_price(0n, 2n * 10n ** 9n).send());

    {
      const assets = [collateralAsset, stableCoin];
      const mintAmount = 10000n;
      for (const asset of assets) {
        const secret = Fr.random();
        const secretHash = computeMessageSecretHash(secret);

        const a = asset.methods.mint_public(lendingAccount.address, mintAmount).send();
        const b = asset.methods.mint_private(mintAmount, secretHash).send();
        await Promise.all([a, b].map(waitForSuccess));

        const storageSlot = new Fr(5);
        const note = new Note([new Fr(mintAmount), secretHash]);
        const txHash = await b.getTxHash();
        const extendedNote = new ExtendedNote(note, accounts[0].address, asset.address, storageSlot, txHash);
        await wallet.addNote(extendedNote);

        await waitForSuccess(asset.methods.redeem_shield(lendingAccount.address, mintAmount, secret).send());
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
    logger('Initializing contract');
    await waitForSuccess(
      lendingContract.methods.init(priceFeedContract.address, 8000, collateralAsset.address, stableCoin.address).send(),
    );
  });

  describe('Deposits', () => {
    it('Depositing 🥸 : 💰 -> 🏦', async () => {
      const depositAmount = 420n;
      const nonce = Fr.random();
      const messageHash = computeAuthWitMessageHash(
        lendingContract.address,
        collateralAsset.methods
          .unshield(lendingAccount.address, lendingContract.address, depositAmount, nonce)
          .request(),
      );

      await wallet.createAuthWitness(Fr.fromBuffer(messageHash));
      await lendingSim.progressTime(TIME_JUMP);
      lendingSim.depositPrivate(lendingAccount.address, lendingAccount.key(), depositAmount);

      // Make a private deposit of funds into own account.
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - increase the private collateral.
      logger('Depositing 🥸 : 💰 -> 🏦');
      await waitForSuccess(
        lendingContract.methods
          .deposit_private(
            lendingAccount.address,
            depositAmount,
            nonce,
            lendingAccount.secret,
            0n,
            collateralAsset.address,
          )
          .send(),
      );
    });

    it('Depositing 🥸 on behalf of recipient: 💰 -> 🏦', async () => {
      const depositAmount = 421n;
      const nonce = Fr.random();
      const messageHash = computeAuthWitMessageHash(
        lendingContract.address,
        collateralAsset.methods
          .unshield(lendingAccount.address, lendingContract.address, depositAmount, nonce)
          .request(),
      );
      await wallet.createAuthWitness(Fr.fromBuffer(messageHash));

      await lendingSim.progressTime(TIME_JUMP);
      lendingSim.depositPrivate(lendingAccount.address, lendingAccount.address.toField(), depositAmount);
      // Make a private deposit of funds into another account, in this case, a public account.
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - increase the public collateral.
      logger('Depositing 🥸 on behalf of recipient: 💰 -> 🏦');
      await waitForSuccess(
        lendingContract.methods
          .deposit_private(
            lendingAccount.address,
            depositAmount,
            nonce,
            0n,
            lendingAccount.address,
            collateralAsset.address,
          )
          .send(),
      );
    });

    it('Depositing: 💰 -> 🏦', async () => {
      const depositAmount = 211n;

      const nonce = Fr.random();
      const messageHash = computeAuthWitMessageHash(
        lendingContract.address,
        collateralAsset.methods
          .transfer_public(lendingAccount.address, lendingContract.address, depositAmount, nonce)
          .request(),
      );

      // Add it to the wallet as approved
      await wallet.setPublicAuth(messageHash, true).send().wait();

      await lendingSim.progressTime(TIME_JUMP);
      lendingSim.depositPublic(lendingAccount.address, lendingAccount.address.toField(), depositAmount);

      // Make a public deposit of funds into self.
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - increase the public collateral.

      logger('Depositing: 💰 -> 🏦');
      await waitForSuccess(
        lendingContract.methods
          .deposit_public(depositAmount, nonce, lendingAccount.address, collateralAsset.address)
          .send(),
      );
    });
  });

  describe('Borrow', () => {
    it('Borrow 🥸 : 🏦 -> 🍌', async () => {
      const borrowAmount = 69n;
      await lendingSim.progressTime(TIME_JUMP);
      lendingSim.borrow(lendingAccount.key(), lendingAccount.address, borrowAmount);

      // Make a private borrow using the private account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - increase the private debt.

      logger('Borrow 🥸 : 🏦 -> 🍌');
      await waitForSuccess(
        lendingContract.methods.borrow_private(lendingAccount.secret, lendingAccount.address, borrowAmount).send(),
      );
    });

    it('Borrow: 🏦 -> 🍌', async () => {
      const borrowAmount = 69n;
      await lendingSim.progressTime(TIME_JUMP);
      lendingSim.borrow(lendingAccount.address.toField(), lendingAccount.address, borrowAmount);

      // Make a public borrow using the private account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - increase the public debt.

      logger('Borrow: 🏦 -> 🍌');
      await waitForSuccess(lendingContract.methods.borrow_public(lendingAccount.address, borrowAmount).send());
    });
  });

  describe('Repay', () => {
    it('Repay 🥸 : 🍌 -> 🏦', async () => {
      const repayAmount = 20n;
      const nonce = Fr.random();
      const messageHash = computeAuthWitMessageHash(
        lendingContract.address,
        stableCoin.methods.burn(lendingAccount.address, repayAmount, nonce).request(),
      );
      await wallet.createAuthWitness(Fr.fromBuffer(messageHash));

      await lendingSim.progressTime(TIME_JUMP);
      lendingSim.repayPrivate(lendingAccount.address, lendingAccount.key(), repayAmount);

      // Make a private repay of the debt in the private account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - decrease the private debt.

      logger('Repay 🥸 : 🍌 -> 🏦');
      await waitForSuccess(
        lendingContract.methods
          .repay_private(lendingAccount.address, repayAmount, nonce, lendingAccount.secret, 0n, stableCoin.address)
          .send(),
      );
    });

    it('Repay 🥸  on behalf of public: 🍌 -> 🏦', async () => {
      const repayAmount = 21n;
      const nonce = Fr.random();
      const messageHash = computeAuthWitMessageHash(
        lendingContract.address,
        stableCoin.methods.burn(lendingAccount.address, repayAmount, nonce).request(),
      );
      await wallet.createAuthWitness(Fr.fromBuffer(messageHash));

      await lendingSim.progressTime(TIME_JUMP);
      lendingSim.repayPrivate(lendingAccount.address, lendingAccount.address.toField(), repayAmount);

      // Make a private repay of the debt in the public account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - decrease the public debt.

      logger('Repay 🥸  on behalf of public: 🍌 -> 🏦');
      await waitForSuccess(
        lendingContract.methods
          .repay_private(lendingAccount.address, repayAmount, nonce, 0n, lendingAccount.address, stableCoin.address)
          .send(),
      );
    });

    it('Repay: 🍌 -> 🏦', async () => {
      const repayAmount = 20n;

      const nonce = Fr.random();
      const messageHash = computeAuthWitMessageHash(
        lendingContract.address,
        stableCoin.methods.burn_public(lendingAccount.address, repayAmount, nonce).request(),
      );

      // Add it to the wallet as approved
      await wallet.setPublicAuth(messageHash, true).send().wait();

      await lendingSim.progressTime(TIME_JUMP);
      lendingSim.repayPublic(lendingAccount.address, lendingAccount.address.toField(), repayAmount);

      // Make a public repay of the debt in the public account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - decrease the public debt.

      logger('Repay: 🍌 -> 🏦');
      await waitForSuccess(
        lendingContract.methods.repay_public(repayAmount, nonce, lendingAccount.address, stableCoin.address).send(),
      );
    });
  });

  describe('Withdraw', () => {
    it('Withdraw: 🏦 -> 💰', async () => {
      const withdrawAmount = 42n;
      await lendingSim.progressTime(TIME_JUMP);
      lendingSim.withdraw(lendingAccount.address.toField(), lendingAccount.address, withdrawAmount);

      // Withdraw funds from the public account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - decrease the public collateral.

      logger('Withdraw: 🏦 -> 💰');
      await waitForSuccess(lendingContract.methods.withdraw_public(lendingAccount.address, withdrawAmount).send());
    });

    it('Withdraw 🥸 : 🏦 -> 💰', async () => {
      const withdrawAmount = 42n;
      await lendingSim.progressTime(TIME_JUMP);
      lendingSim.withdraw(lendingAccount.key(), lendingAccount.address, withdrawAmount);

      // Withdraw funds from the private account
      // This should:
      // - increase the interest accumulator
      // - increase last updated timestamp.
      // - decrease the private collateral.

      logger('Withdraw 🥸 : 🏦 -> 💰');
      await waitForSuccess(
        lendingContract.methods.withdraw_private(lendingAccount.secret, lendingAccount.address, withdrawAmount).send(),
      );
    });

    describe('failure cases', () => {
      it('withdraw more than possible to revert', async () => {
        // Withdraw more than possible to test the revert.
        logger('Withdraw: trying to withdraw more than possible');
        await expect(
          lendingContract.methods.withdraw_public(lendingAccount.address, 10n ** 9n).simulate(),
        ).rejects.toThrow();
      });
    });
  });
});
