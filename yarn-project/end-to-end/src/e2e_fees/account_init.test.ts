import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import {
  type AccountManager,
  type AccountWallet,
  type DebugLogger,
  Fr,
  NativeFeePaymentMethod,
  NativeFeePaymentMethodWithClaim,
  type PXE,
  PrivateFeePaymentMethod,
  PublicFeePaymentMethod,
  Schnorr,
  type Wallet,
  computeSecretHash,
  deriveKeys,
} from '@aztec/aztec.js';
import { type AztecAddress, type CompleteAddress, Fq, type GasSettings } from '@aztec/circuits.js';
import { type TokenContract as BananaCoin, type FPCContract, SchnorrAccountContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { FeesTest } from './fees_test.js';

jest.setTimeout(300_000);

describe('e2e_fees account_init', () => {
  const t = new FeesTest('account_init');

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.applyFundAliceWithBananas();
    await t.applyFPCSetupSnapshot();
    ({ aliceAddress, aliceWallet, bananaCoin, bananaFPC, pxe, logger } = await t.setup());
  });

  afterAll(async () => {
    await t.teardown();
  });

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  let logger: DebugLogger;
  let pxe: PXE;
  let gasSettings: GasSettings;
  let maxFee: bigint;
  let bananaCoin: BananaCoin;
  let bananaFPC: FPCContract;

  // Alice pays for deployments when we need someone else to intervene
  let aliceWallet: Wallet;
  let aliceAddress: AztecAddress;

  // Bob is the account being created (a fresh account is generated for each test)
  let bobsSecretKey: Fr;
  let bobsPrivateSigningKey: Fq;
  let bobsAccountManager: AccountManager;
  let bobsCompleteAddress: CompleteAddress;
  let bobsAddress: AztecAddress;
  let bobsWallet: AccountWallet;

  // Seeded by initBalances below in a beforeEach hook
  let fpcsInitialGas: bigint;
  let fpcsInitialPublicBananas: bigint;

  async function initBalances() {
    [[fpcsInitialGas], [fpcsInitialPublicBananas]] = await Promise.all([
      t.getGasBalanceFn(bananaFPC.address),
      t.getBananaPublicBalanceFn(bananaFPC.address),
    ]);
  }

  beforeEach(async () => {
    bobsSecretKey = Fr.random();
    bobsPrivateSigningKey = Fq.random();
    bobsAccountManager = getSchnorrAccount(pxe, bobsSecretKey, bobsPrivateSigningKey, Fr.random());
    bobsCompleteAddress = bobsAccountManager.getCompleteAddress();
    bobsAddress = bobsCompleteAddress.address;
    bobsWallet = await bobsAccountManager.getWallet();

    gasSettings = t.gasSettings;
    maxFee = gasSettings.getFeeLimit().toBigInt();

    await bobsAccountManager.register();
    await initBalances();
  });

  describe('account pays its own fee', () => {
    it('pays natively in the gas token after Alice bridges funds', async () => {
      await t.gasTokenContract.methods.mint_public(bobsAddress, t.INITIAL_GAS_BALANCE).send().wait();
      const [bobsInitialGas] = await t.getGasBalanceFn(bobsAddress);
      expect(bobsInitialGas).toEqual(t.INITIAL_GAS_BALANCE);

      const paymentMethod = new NativeFeePaymentMethod(bobsAddress);
      const tx = await bobsAccountManager.deploy({ fee: { gasSettings, paymentMethod } }).wait();

      expect(tx.transactionFee!).toBeGreaterThan(0n);
      await expect(t.getGasBalanceFn(bobsAddress)).resolves.toEqual([bobsInitialGas - tx.transactionFee!]);
    });

    it('pays natively in the gas token by bridging funds themselves', async () => {
      const { secret } = await t.gasBridgeTestHarness.prepareTokensOnL1(
        t.INITIAL_GAS_BALANCE,
        t.INITIAL_GAS_BALANCE,
        bobsAddress,
      );

      const paymentMethod = new NativeFeePaymentMethodWithClaim(bobsAddress, t.INITIAL_GAS_BALANCE, secret);
      const tx = await bobsAccountManager.deploy({ fee: { gasSettings, paymentMethod } }).wait();
      expect(tx.transactionFee!).toBeGreaterThan(0n);
      await expect(t.getGasBalanceFn(bobsAddress)).resolves.toEqual([t.INITIAL_GAS_BALANCE - tx.transactionFee!]);
    });

    it('pays privately through an FPC', async () => {
      // Alice mints bananas to Bob
      const mintedBananas = BigInt(1e12);
      await t.mintPrivateBananas(mintedBananas, bobsAddress);

      // Bob deploys his account through the private FPC
      const rebateSecret = Fr.random();
      const paymentMethod = new PrivateFeePaymentMethod(
        bananaCoin.address,
        bananaFPC.address,
        await bobsAccountManager.getWallet(),
        rebateSecret,
      );

      const tx = await bobsAccountManager.deploy({ fee: { gasSettings, paymentMethod } }).wait();
      const actualFee = tx.transactionFee!;
      expect(actualFee).toBeGreaterThan(0n);

      // the new account should have paid the full fee to the FPC
      await expect(t.getBananaPrivateBalanceFn(bobsAddress)).resolves.toEqual([mintedBananas - maxFee]);

      // the FPC got paid through "unshield", so it's got a new public balance
      await expect(t.getBananaPublicBalanceFn(bananaFPC.address)).resolves.toEqual([
        fpcsInitialPublicBananas + actualFee,
      ]);

      // the FPC should have been the fee payer
      await expect(t.getGasBalanceFn(bananaFPC.address)).resolves.toEqual([fpcsInitialGas - actualFee]);

      // the new account should have received a refund
      await t.addPendingShieldNoteToPXE(bobsAddress, maxFee - actualFee, computeSecretHash(rebateSecret), tx.txHash);

      // and it can redeem the refund
      await bananaCoin.methods
        .redeem_shield(bobsAddress, maxFee - actualFee, rebateSecret)
        .send()
        .wait();

      await expect(t.getBananaPrivateBalanceFn(bobsAddress)).resolves.toEqual([mintedBananas - actualFee]);
    });

    it('pays publicly through an FPC', async () => {
      const mintedBananas = BigInt(1e12);
      await bananaCoin.methods.mint_public(bobsAddress, mintedBananas).send().wait();

      const paymentMethod = new PublicFeePaymentMethod(bananaCoin.address, bananaFPC.address, bobsWallet);
      const tx = await bobsAccountManager
        .deploy({
          skipPublicDeployment: false,
          fee: { gasSettings, paymentMethod },
        })
        .wait();

      const actualFee = tx.transactionFee!;
      expect(actualFee).toBeGreaterThan(0n);

      // we should have paid the fee to the FPC
      await expect(t.getBananaPublicBalanceFn(bobsAddress, bananaFPC.address)).resolves.toEqual([
        mintedBananas - actualFee,
        fpcsInitialPublicBananas + actualFee,
      ]);

      // the FPC should have paid the sequencer
      await expect(t.getGasBalanceFn(bananaFPC.address)).resolves.toEqual([fpcsInitialGas - actualFee]);
    });
  });

  describe('another account pays the fee', () => {
    it('pays natively in the gas token', async () => {
      // mint gas tokens to alice
      await t.gasTokenContract.methods.mint_public(aliceAddress, t.INITIAL_GAS_BALANCE).send().wait();
      const [alicesInitialGas] = await t.getGasBalanceFn(aliceAddress);

      // bob generates the private keys for his account on his own
      const bobsPublicKeysHash = deriveKeys(bobsSecretKey).publicKeys.hash();
      const bobsSigningPubKey = new Schnorr().computePublicKey(bobsPrivateSigningKey);
      const bobsInstance = bobsAccountManager.getInstance();

      // alice registers bobs keys in the pxe
      await pxe.registerRecipient(bobsCompleteAddress);

      // and deploys bob's account, paying the fee from her balance
      const paymentMethod = new NativeFeePaymentMethod(aliceAddress);
      const tx = await SchnorrAccountContract.deployWithPublicKeysHash(
        bobsPublicKeysHash,
        aliceWallet,
        bobsSigningPubKey.x,
        bobsSigningPubKey.y,
      )
        .send({
          contractAddressSalt: bobsInstance.salt,
          skipClassRegistration: true,
          skipPublicDeployment: true,
          skipInitialization: false,
          universalDeploy: true,
          fee: { gasSettings, paymentMethod },
        })
        .wait();

      // alice paid in gas tokens
      expect(tx.transactionFee!).toBeGreaterThan(0n);
      await expect(t.getGasBalanceFn(aliceAddress)).resolves.toEqual([alicesInitialGas - tx.transactionFee!]);

      // bob can now use his wallet for sending txs
      await bananaCoin.withWallet(bobsWallet).methods.transfer_public(bobsAddress, aliceAddress, 0n, 0n).send().wait();
    });
  });
});
