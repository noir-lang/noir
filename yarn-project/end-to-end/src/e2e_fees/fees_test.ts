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
import { EthAddress, GasSettings, computePartialAddress } from '@aztec/circuits.js';
import { createL1Clients } from '@aztec/ethereum';
import { PortalERC20Abi } from '@aztec/l1-artifacts';
import {
  AppSubscriptionContract,
  TokenContract as BananaCoin,
  CounterContract,
  FPCContract,
  FeeJuiceContract,
  PrivateFPCContract,
  TokenWithRefundsContract,
} from '@aztec/noir-contracts.js';
import { getCanonicalFeeJuice } from '@aztec/protocol-contracts/fee-juice';

import { getContract } from 'viem';

import { MNEMONIC } from '../fixtures/fixtures.js';
import { type ISnapshotManager, addAccounts, createSnapshotManager } from '../fixtures/snapshot_manager.js';
import { type BalancesFn, deployCanonicalFeeJuice, getBalancesFn, publicDeployAccounts } from '../fixtures/utils.js';
import {
  FeeJuicePortalTestingHarnessFactory,
  type IGasBridgingTestHarness,
} from '../shared/gas_portal_test_harness.js';

const { E2E_DATA_PATH: dataPath } = process.env;

/**
 * Test fixture for testing fees. Provides the following snapshots:
 * InitialAccounts: Initializes 3 Schnorr account contracts.
 * PublicDeployAccounts: Deploys the accounts publicly.
 * DeployFeeJuice: Deploys the Fee Juice contract.
 * FPCSetup: Deploys BananaCoin and FPC contracts, and bridges gas from L1.
 * FundAlice: Mints private and public bananas to Alice.
 * SetupSubscription: Deploys a counter contract and a subscription contract, and mints Fee Juice to the subscription contract.
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
  public coinbase!: EthAddress;

  public gasSettings = GasSettings.default();
  public maxFee = this.gasSettings.getFeeLimit().toBigInt();

  public feeJuiceContract!: FeeJuiceContract;
  public bananaCoin!: BananaCoin;
  public bananaFPC!: FPCContract;
  public tokenWithRefunds!: TokenWithRefundsContract;
  public privateFPC!: PrivateFPCContract;
  public counterContract!: CounterContract;
  public subscriptionContract!: AppSubscriptionContract;
  public feeJuiceBridgeTestHarness!: IGasBridgingTestHarness;

  public getCoinbaseBalance!: () => Promise<bigint>;
  public getGasBalanceFn!: BalancesFn;
  public getBananaPublicBalanceFn!: BalancesFn;
  public getBananaPrivateBalanceFn!: BalancesFn;
  public getTokenWithRefundsBalanceFn!: BalancesFn;

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
    await context.aztecNode.setConfig({ feeRecipient: this.sequencerAddress, coinbase: this.coinbase });
    return this;
  }

  async teardown() {
    await this.snapshotManager.teardown();
  }

  /** Alice mints TokenWithRefunds  */
  async mintTokenWithRefunds(amount: bigint) {
    const balanceBefore = await this.tokenWithRefunds.methods.balance_of_private(this.aliceAddress).simulate();
    await this.tokenWithRefunds.methods.privately_mint_private_note(amount).send().wait();
    const balanceAfter = await this.tokenWithRefunds.methods.balance_of_private(this.aliceAddress).simulate();
    expect(balanceAfter).toEqual(balanceBefore + amount);
  }

  /** Alice mints bananaCoin tokens privately to the target address and redeems them. */
  async mintPrivateBananas(amount: bigint, address: AztecAddress) {
    const balanceBefore = await this.bananaCoin.methods.balance_of_private(address).simulate();
    const secret = await this.mintShieldedBananas(amount, address);
    await this.redeemShieldedBananas(amount, address, secret);
    const balanceAfter = await this.bananaCoin.methods.balance_of_private(address).simulate();
    expect(balanceAfter).toEqual(balanceBefore + amount);
  }

  /** Alice mints bananaCoin tokens privately to the target address but does not redeem them yet. */
  async mintShieldedBananas(amount: bigint, address: AztecAddress) {
    const secret = Fr.random();
    const secretHash = computeSecretHash(secret);
    this.logger.debug(`Minting ${amount} bananas privately for ${address} with secret ${secretHash.toString()}`);
    const receipt = await this.bananaCoin.methods.mint_private(amount, secretHash).send().wait();
    await this.addPendingShieldNoteToPXE(this.aliceAddress, amount, secretHash, receipt.txHash);
    return secret;
  }

  /** Redeemer (defaults to Alice) redeems shielded bananas for the target address. */
  async redeemShieldedBananas(amount: bigint, address: AztecAddress, secret: Fr, redeemer?: AccountWallet) {
    this.logger.debug(`Redeeming ${amount} bananas for ${address}`);
    const bananaCoin = redeemer ? this.bananaCoin.withWallet(redeemer) : this.bananaCoin;
    await bananaCoin.methods.redeem_shield(address, amount, secret).send().wait();
  }

  /** Adds a pending shield transparent node for the banana coin token contract to the pxe. */
  async addPendingShieldNoteToPXE(owner: AztecAddress | AccountWallet, amount: bigint, secretHash: Fr, txHash: TxHash) {
    const note = new Note([new Fr(amount), secretHash]);
    const ownerAddress = 'getAddress' in owner ? owner.getAddress() : owner;
    const extendedNote = new ExtendedNote(
      note,
      ownerAddress,
      this.bananaCoin.address,
      BananaCoin.storage.pending_shields.slot,
      BananaCoin.notes.TransparentNote.id,
      txHash,
    );
    await this.pxe.addNote(extendedNote, ownerAddress);
  }

  public async applyBaseSnapshots() {
    await this.applyInitialAccountsSnapshot();
    await this.applyPublicDeployAccountsSnapshot();
    await this.applyDeployFeeJuiceSnapshot();
    await this.applyDeployBananaTokenSnapshot();
  }

  async applyInitialAccountsSnapshot() {
    await this.snapshotManager.snapshot(
      'initial_accounts',
      addAccounts(3, this.logger),
      async ({ accountKeys }, { pxe, aztecNode, aztecNodeConfig }) => {
        this.pxe = pxe;
        this.aztecNode = aztecNode;
        const accountManagers = accountKeys.map(ak => getSchnorrAccount(pxe, ak[0], ak[1], 1));
        await Promise.all(accountManagers.map(a => a.register()));
        this.wallets = await Promise.all(accountManagers.map(a => a.getWallet()));
        this.wallets.forEach((w, i) => this.logger.verbose(`Wallet ${i} address: ${w.getAddress()}`));
        [this.aliceWallet, this.bobWallet] = this.wallets.slice(0, 2);
        [this.aliceAddress, this.bobAddress, this.sequencerAddress] = this.wallets.map(w => w.getAddress());
        this.feeJuiceContract = await FeeJuiceContract.at(getCanonicalFeeJuice().address, this.aliceWallet);
        const bobInstance = await this.bobWallet.getContractInstance(this.bobAddress);
        if (!bobInstance) {
          throw new Error('Bob instance not found');
        }
        await this.aliceWallet.registerAccount(accountKeys[1][0], computePartialAddress(bobInstance));
        this.coinbase = EthAddress.random();

        const { publicClient, walletClient } = createL1Clients(aztecNodeConfig.l1RpcUrl, MNEMONIC);
        this.feeJuiceBridgeTestHarness = await FeeJuicePortalTestingHarnessFactory.create({
          aztecNode: aztecNode,
          pxeService: pxe,
          publicClient: publicClient,
          walletClient: walletClient,
          wallet: this.aliceWallet,
          logger: this.logger,
          mockL1: false,
        });
      },
    );
  }

  async applyPublicDeployAccountsSnapshot() {
    await this.snapshotManager.snapshot('public_deploy_accounts', () =>
      publicDeployAccounts(this.aliceWallet, this.wallets),
    );
  }

  async applyDeployFeeJuiceSnapshot() {
    await this.snapshotManager.snapshot(
      'deploy_fee_juice',
      async context => {
        await deployCanonicalFeeJuice(
          new SignerlessWallet(
            context.pxe,
            new DefaultMultiCallEntrypoint(context.aztecNodeConfig.l1ChainId, context.aztecNodeConfig.version),
          ),
        );
      },
      async (_data, context) => {
        this.feeJuiceContract = await FeeJuiceContract.at(getCanonicalFeeJuice().address, this.aliceWallet);

        this.getGasBalanceFn = getBalancesFn('⛽', this.feeJuiceContract.methods.balance_of_public, this.logger);

        const { publicClient, walletClient } = createL1Clients(context.aztecNodeConfig.l1RpcUrl, MNEMONIC);
        this.feeJuiceBridgeTestHarness = await FeeJuicePortalTestingHarnessFactory.create({
          aztecNode: context.aztecNode,
          pxeService: context.pxe,
          publicClient: publicClient,
          walletClient: walletClient,
          wallet: this.aliceWallet,
          logger: this.logger,
          mockL1: false,
        });
      },
    );
  }

  async applyDeployBananaTokenSnapshot() {
    await this.snapshotManager.snapshot(
      'deploy_banana_token',
      async () => {
        const bananaCoin = await BananaCoin.deploy(this.aliceWallet, this.aliceAddress, 'BC', 'BC', 18n)
          .send()
          .deployed();
        this.logger.info(`BananaCoin deployed at ${bananaCoin.address}`);
        return { bananaCoinAddress: bananaCoin.address };
      },
      async ({ bananaCoinAddress }) => {
        this.bananaCoin = await BananaCoin.at(bananaCoinAddress, this.aliceWallet);
      },
    );
  }

  async applyTokenWithRefundsAndFPC() {
    await this.snapshotManager.snapshot(
      'token_with_refunds_and_private_fpc',
      async context => {
        // Deploy token/fpc flavors for private refunds
        const feeJuiceContract = this.feeJuiceBridgeTestHarness.l2Token;
        expect(await context.pxe.isContractPubliclyDeployed(feeJuiceContract.address)).toBe(true);

        const tokenWithRefunds = await TokenWithRefundsContract.deploy(
          this.aliceWallet,
          this.aliceAddress,
          'PVT',
          'PVT',
          18n,
        )
          .send()
          .deployed();

        this.logger.info(`TokenWithRefunds deployed at ${tokenWithRefunds.address}`);

        const privateFPCSent = PrivateFPCContract.deploy(
          this.bobWallet,
          tokenWithRefunds.address,
          this.bobWallet.getAddress(),
        ).send();
        const privateFPC = await privateFPCSent.deployed();

        this.logger.info(`PrivateFPC deployed at ${privateFPC.address}`);
        await this.feeJuiceBridgeTestHarness.bridgeFromL1ToL2(
          this.INITIAL_GAS_BALANCE,
          this.INITIAL_GAS_BALANCE,
          privateFPC.address,
        );

        return {
          tokenWithRefundsAddress: tokenWithRefunds.address,
          privateFPCAddress: privateFPC.address,
        };
      },
      async data => {
        this.privateFPC = await PrivateFPCContract.at(data.privateFPCAddress, this.bobWallet);
        this.tokenWithRefunds = await TokenWithRefundsContract.at(data.tokenWithRefundsAddress, this.aliceWallet);

        const logger = this.logger;
        this.getTokenWithRefundsBalanceFn = getBalancesFn(
          '🕵️.private',
          this.tokenWithRefunds.methods.balance_of_private,
          logger,
        );
      },
    );
  }

  public async applyFPCSetupSnapshot() {
    await this.snapshotManager.snapshot(
      'fpc_setup',
      async context => {
        const feeJuiceContract = this.feeJuiceBridgeTestHarness.l2Token;
        expect(await context.pxe.isContractPubliclyDeployed(feeJuiceContract.address)).toBe(true);

        const bananaCoin = this.bananaCoin;
        const bananaFPC = await FPCContract.deploy(this.aliceWallet, bananaCoin.address).send().deployed();

        this.logger.info(`BananaPay deployed at ${bananaFPC.address}`);

        await this.feeJuiceBridgeTestHarness.bridgeFromL1ToL2(
          this.INITIAL_GAS_BALANCE,
          this.INITIAL_GAS_BALANCE,
          bananaFPC.address,
        );

        return {
          bananaFPCAddress: bananaFPC.address,
          feeJuiceAddress: feeJuiceContract.address,
          l1FeeJuiceAddress: this.feeJuiceBridgeTestHarness.l1FeeJuiceAddress,
        };
      },
      async (data, context) => {
        const bananaFPC = await FPCContract.at(data.bananaFPCAddress, this.aliceWallet);
        this.bananaFPC = bananaFPC;

        const logger = this.logger;
        this.getBananaPublicBalanceFn = getBalancesFn('🍌.public', this.bananaCoin.methods.balance_of_public, logger);
        this.getBananaPrivateBalanceFn = getBalancesFn(
          '🍌.private',
          this.bananaCoin.methods.balance_of_private,
          logger,
        );

        this.getCoinbaseBalance = async () => {
          const { walletClient } = createL1Clients(context.aztecNodeConfig.l1RpcUrl, MNEMONIC);
          const gasL1 = getContract({
            address: data.l1FeeJuiceAddress.toString(),
            abi: PortalERC20Abi,
            client: walletClient,
          });
          return await gasL1.read.balanceOf([this.coinbase.toString()]);
        };
      },
    );
  }

  public async applyFundAliceWithBananas() {
    await this.snapshotManager.snapshot(
      'fund_alice',
      async () => {
        await this.mintPrivateBananas(this.ALICE_INITIAL_BANANAS, this.aliceAddress);
        await this.bananaCoin.methods.mint_public(this.aliceAddress, this.ALICE_INITIAL_BANANAS).send().wait();
      },
      () => Promise.resolve(),
    );
  }

  public async applyFundAliceWithTokens() {
    await this.snapshotManager.snapshot(
      'fund_alice_with_tokens',
      async () => {
        await this.mintTokenWithRefunds(this.ALICE_INITIAL_BANANAS);
      },
      () => Promise.resolve(),
    );
  }

  public async applyFundAliceWithFeeJuice() {
    await this.snapshotManager.snapshot(
      'fund_alice_with_fee_juice',
      async () => {
        await this.feeJuiceContract.methods.mint_public(this.aliceAddress, this.INITIAL_GAS_BALANCE).send().wait();
      },
      () => Promise.resolve(),
    );
  }

  public async applySetupSubscription() {
    await this.snapshotManager.snapshot(
      'setup_subscription',
      async () => {
        // Deploy counter contract for testing with Bob as owner
        // Emitting the outgoing logs to Bob below since we need someone to emit them to.
        const counterContract = await CounterContract.deploy(this.bobWallet, 0, this.bobAddress, this.bobAddress)
          .send()
          .deployed();

        // Deploy subscription contract, that allows subscriptions for SUBSCRIPTION_AMOUNT of bananas
        const subscriptionContract = await AppSubscriptionContract.deploy(
          this.bobWallet,
          counterContract.address,
          this.bobAddress,
          this.bananaCoin.address,
          this.SUBSCRIPTION_AMOUNT,
          this.APP_SPONSORED_TX_GAS_LIMIT,
        )
          .send()
          .deployed();

        // Mint some Fee Juice to the subscription contract
        // Could also use bridgeFromL1ToL2 from the harness, but this is more direct
        await this.feeJuiceContract.methods
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
}
