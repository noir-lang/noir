import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import {
  type AccountWallet,
  type AztecAddress,
  type AztecNode,
  type DebugLogger,
  ExtendedNote,
  Fr,
  Note,
  type PXE,
  SignerlessWallet,
  type TxHash,
  computeSecretHash,
  createDebugLogger,
} from '@aztec/aztec.js';
import { DefaultMultiCallEntrypoint } from '@aztec/aztec.js/entrypoint';
import { GasSettings } from '@aztec/circuits.js';
import { createL1Clients } from '@aztec/ethereum';
import {
  AppSubscriptionContract,
  TokenContract as BananaCoin,
  CounterContract,
  FPCContract,
  GasTokenContract,
} from '@aztec/noir-contracts.js';

import { MNEMONIC } from '../fixtures/fixtures.js';
import {
  type ISnapshotManager,
  type SubsystemsContext,
  addAccounts,
  createSnapshotManager,
} from '../fixtures/snapshot_manager.js';
import { type BalancesFn, deployCanonicalGasToken, getBalancesFn, publicDeployAccounts } from '../fixtures/utils.js';
import { GasPortalTestingHarnessFactory } from '../shared/gas_portal_test_harness.js';

const { E2E_DATA_PATH: dataPath } = process.env;

/**
 * Test fixture for testing fees. Provides the following snapshots:
 * InitialAccounts: Initializes 3 Schnorr account contracts.
 * PublicDeployAccounts: Deploys the accounts publicly.
 * DeployGasToken: Deploys the gas token contract.
 * FPCSetup: Deploys BananaCoin and FPC contracts, and bridges gas from L1.
 * FundAlice: Mints private and public bananas to Alice.
 * SetupSubscription: Deploys a counter contract and a subscription contract, and mints gas token to the subscription contract.
 */
export class FeesTest {
  private snapshotManager: ISnapshotManager;
  private wallets: AccountWallet[] = [];

  public logger: DebugLogger;
  public pxe!: PXE;
  public aztecNode!: AztecNode;

  public aliceWallet!: AccountWallet;
  public aliceAddress!: AztecAddress;
  public bobWallet!: AccountWallet;
  public bobAddress!: AztecAddress;
  public sequencerAddress!: AztecAddress;

  public gasSettings = GasSettings.default();
  public maxFee = this.gasSettings.getFeeLimit().toBigInt();

  public gasTokenContract!: GasTokenContract;
  public bananaCoin!: BananaCoin;
  public bananaFPC!: FPCContract;
  public counterContract!: CounterContract;
  public subscriptionContract!: AppSubscriptionContract;

  public gasBalances!: BalancesFn;
  public bananaPublicBalances!: BalancesFn;
  public bananaPrivateBalances!: BalancesFn;

  public readonly INITIAL_GAS_BALANCE = BigInt(1e15);
  public readonly ALICE_INITIAL_BANANAS = BigInt(1e12);
  public readonly SUBSCRIPTION_AMOUNT = 10_000n;
  public readonly APP_SPONSORED_TX_GAS_LIMIT = BigInt(10e9);

  constructor(testName: string) {
    this.logger = createDebugLogger(`aztec:e2e_fees:${testName}`);
    this.snapshotManager = createSnapshotManager(`e2e_fees/${testName}`, dataPath);
  }

  async setup() {
    const context = await this.snapshotManager.setup();
    await context.aztecNode.setConfig({ feeRecipient: this.sequencerAddress });
    ({ pxe: this.pxe, aztecNode: this.aztecNode } = context);
    return this;
  }

  async teardown() {
    await this.snapshotManager.teardown();
  }

  /** Alice mints bananaCoin tokens privately to the target address. */
  async mintPrivate(amount: bigint, address: AztecAddress) {
    const secret = Fr.random();
    const secretHash = computeSecretHash(secret);
    const balanceBefore = await this.bananaCoin.methods.balance_of_private(this.aliceAddress).simulate();
    this.logger.debug(`Minting ${amount} bananas privately for ${address} with secret ${secretHash.toString()}`);
    const receipt = await this.bananaCoin.methods.mint_private(amount, secretHash).send().wait();

    await this.addPendingShieldNoteToPXE(this.aliceWallet, amount, secretHash, receipt.txHash);
    await this.bananaCoin.methods.redeem_shield(address, amount, secret).send().wait();
    const balanceAfter = await this.bananaCoin.methods.balance_of_private(this.aliceAddress).simulate();
    expect(balanceAfter).toEqual(balanceBefore + amount);
  }

  async addPendingShieldNoteToPXE(wallet: AccountWallet, amount: bigint, secretHash: Fr, txHash: TxHash) {
    const note = new Note([new Fr(amount), secretHash]);
    const extendedNote = new ExtendedNote(
      note,
      wallet.getAddress(),
      this.bananaCoin.address,
      BananaCoin.storage.pending_shields.slot,
      BananaCoin.notes.TransparentNote.id,
      txHash,
    );
    await wallet.addNote(extendedNote);
  }

  public async applyBaseSnapshots() {
    await this.applyInitialAccountsSnapshot();
    await this.applyPublicDeployAccountsSnapshot();
    await this.applyDeployGasTokenSnapshot();
    await this.applyFPCSetupSnapshot();
  }

  private async applyInitialAccountsSnapshot() {
    await this.snapshotManager.snapshot(
      'initial_accounts',
      addAccounts(3, this.logger),
      async ({ accountKeys }, { pxe }) => {
        const accountManagers = accountKeys.map(ak => getSchnorrAccount(pxe, ak[0], ak[1], 1));
        await Promise.all(accountManagers.map(a => a.register()));
        this.wallets = await Promise.all(accountManagers.map(a => a.getWallet()));
        this.wallets.forEach((w, i) => this.logger.verbose(`Wallet ${i} address: ${w.getAddress()}`));
        [this.aliceWallet, this.bobWallet] = this.wallets.slice(0, 2);
        [this.aliceAddress, this.bobAddress, this.sequencerAddress] = this.wallets.map(w => w.getAddress());
      },
    );
  }

  private async applyPublicDeployAccountsSnapshot() {
    await this.snapshotManager.snapshot('public_deploy_accounts', () =>
      publicDeployAccounts(this.aliceWallet, this.wallets),
    );
  }

  private async applyDeployGasTokenSnapshot() {
    await this.snapshotManager.snapshot('deploy_gas_token', async context => {
      await deployCanonicalGasToken(
        new SignerlessWallet(
          context.pxe,
          new DefaultMultiCallEntrypoint(context.aztecNodeConfig.chainId, context.aztecNodeConfig.version),
        ),
      );
    });
  }

  private async applyFPCSetupSnapshot() {
    await this.snapshotManager.snapshot(
      'fpc_setup',
      async context => {
        const harness = await this.createGasBridgeTestHarness(context);
        const gasTokenContract = harness.l2Token;
        expect(await context.pxe.isContractPubliclyDeployed(gasTokenContract.address)).toBe(true);

        const bananaCoin = await BananaCoin.deploy(this.aliceWallet, this.aliceAddress, 'BC', 'BC', 18n)
          .send()
          .deployed();

        this.logger.info(`BananaCoin deployed at ${bananaCoin.address}`);

        const bananaFPC = await FPCContract.deploy(this.aliceWallet, bananaCoin.address, gasTokenContract.address)
          .send()
          .deployed();

        this.logger.info(`BananaPay deployed at ${bananaFPC.address}`);

        await harness.bridgeFromL1ToL2(this.INITIAL_GAS_BALANCE, this.INITIAL_GAS_BALANCE, bananaFPC.address);

        return {
          bananaCoinAddress: bananaCoin.address,
          bananaFPCAddress: bananaFPC.address,
          gasTokenAddress: gasTokenContract.address,
        };
      },
      async data => {
        const bananaFPC = await FPCContract.at(data.bananaFPCAddress, this.aliceWallet);
        const bananaCoin = await BananaCoin.at(data.bananaCoinAddress, this.aliceWallet);
        const gasTokenContract = await GasTokenContract.at(data.gasTokenAddress, this.aliceWallet);

        this.bananaCoin = bananaCoin;
        this.bananaFPC = bananaFPC;
        this.gasTokenContract = gasTokenContract;

        this.bananaPublicBalances = getBalancesFn('🍌.public', bananaCoin.methods.balance_of_public, this.logger);
        this.bananaPrivateBalances = getBalancesFn('🍌.private', bananaCoin.methods.balance_of_private, this.logger);
        this.gasBalances = getBalancesFn('⛽', gasTokenContract.methods.balance_of_public, this.logger);
      },
    );
  }

  public async applyFundAlice() {
    await this.snapshotManager.snapshot(
      'fund_alice',
      async () => {
        await this.mintPrivate(BigInt(this.ALICE_INITIAL_BANANAS), this.aliceAddress);
        await this.bananaCoin.methods.mint_public(this.aliceAddress, this.ALICE_INITIAL_BANANAS).send().wait();
      },
      () => Promise.resolve(),
    );
  }

  public async applySetupSubscription() {
    await this.snapshotManager.snapshot(
      'setup_subscription',
      async () => {
        // Deploy counter contract for testing with Bob as owner
        const counterContract = await CounterContract.deploy(this.bobWallet, 0, this.bobAddress).send().deployed();

        // Deploy subscription contract, that allows subscriptions for SUBSCRIPTION_AMOUNT of bananas
        const subscriptionContract = await AppSubscriptionContract.deploy(
          this.bobWallet,
          counterContract.address,
          this.bobAddress,
          this.bananaCoin.address,
          this.SUBSCRIPTION_AMOUNT,
          this.gasTokenContract.address,
          this.APP_SPONSORED_TX_GAS_LIMIT,
        )
          .send()
          .deployed();

        // Mint some gas tokens to the subscription contract
        // Could also use bridgeFromL1ToL2 from the harness, but this is more direct
        await this.gasTokenContract.methods
          .mint_public(subscriptionContract.address, this.INITIAL_GAS_BALANCE)
          .send()
          .wait();

        return {
          counterContractAddress: counterContract.address,
          subscriptionContractAddress: subscriptionContract.address,
        };
      },
      async ({ counterContractAddress, subscriptionContractAddress }) => {
        this.counterContract = await CounterContract.at(counterContractAddress, this.bobWallet);
        this.subscriptionContract = await AppSubscriptionContract.at(subscriptionContractAddress, this.bobWallet);
      },
    );
  }

  private createGasBridgeTestHarness(context: SubsystemsContext) {
    const { publicClient, walletClient } = createL1Clients(context.aztecNodeConfig.rpcUrl, MNEMONIC);

    return GasPortalTestingHarnessFactory.create({
      aztecNode: context.aztecNode,
      pxeService: context.pxe,
      publicClient: publicClient,
      walletClient: walletClient,
      wallet: this.aliceWallet,
      logger: this.logger,
      mockL1: false,
    });
  }
}
