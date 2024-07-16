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
  GasTokenContract,
  PrivateFPCContract,
  PrivateTokenContract,
} from '@aztec/noir-contracts.js';
import { getCanonicalGasToken } from '@aztec/protocol-contracts/gas-token';

import { getContract } from 'viem';

import { MNEMONIC } from '../fixtures/fixtures.js';
import { type ISnapshotManager, addAccounts, createSnapshotManager } from '../fixtures/snapshot_manager.js';
import { type BalancesFn, deployCanonicalGasToken, getBalancesFn, publicDeployAccounts } from '../fixtures/utils.js';
import { GasPortalTestingHarnessFactory, type IGasBridgingTestHarness } from '../shared/gas_portal_test_harness.js';

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
  public coinbase!: EthAddress;

  public gasSettings = GasSettings.default();
  public maxFee = this.gasSettings.getFeeLimit().toBigInt();

  public gasTokenContract!: GasTokenContract;
  public bananaCoin!: BananaCoin;
  public bananaFPC!: FPCContract;
  public privateToken!: PrivateTokenContract;
  public privateFPC!: PrivateFPCContract;
  public counterContract!: CounterContract;
  public subscriptionContract!: AppSubscriptionContract;
  public gasBridgeTestHarness!: IGasBridgingTestHarness;

  public getCoinbaseBalance!: () => Promise<bigint>;
  public getGasBalanceFn!: BalancesFn;
  public getBananaPublicBalanceFn!: BalancesFn;
  public getBananaPrivateBalanceFn!: BalancesFn;
  public getPrivateTokenBalanceFn!: BalancesFn;

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

  /** Alice mints PrivateToken  */
  async mintPrivateTokens(amount: bigint) {
    const balanceBefore = await this.privateToken.methods.balance_of_private(this.aliceAddress).simulate();
    await this.privateToken.methods.privately_mint_private_note(amount).send().wait();
    const balanceAfter = await this.privateToken.methods.balance_of_private(this.aliceAddress).simulate();
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
    await this.pxe.addNote(extendedNote);
  }

  public async applyBaseSnapshots() {
    await this.applyInitialAccountsSnapshot();
    await this.applyPublicDeployAccountsSnapshot();
    await this.applyDeployGasTokenSnapshot();
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
        this.gasTokenContract = await GasTokenContract.at(getCanonicalGasToken().address, this.aliceWallet);
        const bobInstance = await this.bobWallet.getContractInstance(this.bobAddress);
        if (!bobInstance) {
          throw new Error('Bob instance not found');
        }
        await this.aliceWallet.registerAccount(accountKeys[1][0], computePartialAddress(bobInstance));
        this.coinbase = EthAddress.random();

        const { publicClient, walletClient } = createL1Clients(aztecNodeConfig.rpcUrl, MNEMONIC);
        this.gasBridgeTestHarness = await GasPortalTestingHarnessFactory.create({
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

  async applyDeployGasTokenSnapshot() {
    await this.snapshotManager.snapshot(
      'deploy_gas_token',
      async context => {
        await deployCanonicalGasToken(
          new SignerlessWallet(
            context.pxe,
            new DefaultMultiCallEntrypoint(context.aztecNodeConfig.l1ChainId, context.aztecNodeConfig.version),
          ),
        );
      },
      async (_data, context) => {
        this.gasTokenContract = await GasTokenContract.at(getCanonicalGasToken().address, this.aliceWallet);

        this.getGasBalanceFn = getBalancesFn('⛽', this.gasTokenContract.methods.balance_of_public, this.logger);

        const { publicClient, walletClient } = createL1Clients(context.aztecNodeConfig.rpcUrl, MNEMONIC);
        this.gasBridgeTestHarness = await GasPortalTestingHarnessFactory.create({
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

  async applyPrivateTokenAndFPC() {
    await this.snapshotManager.snapshot(
      'private_token_and_private_fpc',
      async context => {
        // Deploy token/fpc flavors for private refunds
        const gasTokenContract = this.gasBridgeTestHarness.l2Token;
        expect(await context.pxe.isContractPubliclyDeployed(gasTokenContract.address)).toBe(true);

        const privateToken = await PrivateTokenContract.deploy(this.aliceWallet, this.aliceAddress, 'PVT', 'PVT', 18n)
          .send()
          .deployed();

        this.logger.info(`PrivateToken deployed at ${privateToken.address}`);
        const adminKeyHash = this.bobWallet.getCompleteAddress().publicKeys.masterNullifierPublicKey.hash();

        const privateFPCSent = PrivateFPCContract.deploy(this.bobWallet, privateToken.address, adminKeyHash).send();
        const privateFPC = await privateFPCSent.deployed();

        this.logger.info(`PrivateFPC deployed at ${privateFPC.address}`);
        await this.gasBridgeTestHarness.bridgeFromL1ToL2(
          this.INITIAL_GAS_BALANCE,
          this.INITIAL_GAS_BALANCE,
          privateFPC.address,
        );

        return {
          privateTokenAddress: privateToken.address,
          privateFPCAddress: privateFPC.address,
        };
      },
      async data => {
        this.privateFPC = await PrivateFPCContract.at(data.privateFPCAddress, this.bobWallet);
        this.privateToken = await PrivateTokenContract.at(data.privateTokenAddress, this.aliceWallet);

        const logger = this.logger;
        this.getPrivateTokenBalanceFn = getBalancesFn(
          '🕵️.private',
          this.privateToken.methods.balance_of_private,
          logger,
        );
      },
    );
  }

  public async applyFPCSetupSnapshot() {
    await this.snapshotManager.snapshot(
      'fpc_setup',
      async context => {
        const gasTokenContract = this.gasBridgeTestHarness.l2Token;
        expect(await context.pxe.isContractPubliclyDeployed(gasTokenContract.address)).toBe(true);

        const bananaCoin = this.bananaCoin;
        const bananaFPC = await FPCContract.deploy(this.aliceWallet, bananaCoin.address, gasTokenContract.address)
          .send()
          .deployed();

        this.logger.info(`BananaPay deployed at ${bananaFPC.address}`);

        await this.gasBridgeTestHarness.bridgeFromL1ToL2(
          this.INITIAL_GAS_BALANCE,
          this.INITIAL_GAS_BALANCE,
          bananaFPC.address,
        );

        return {
          bananaFPCAddress: bananaFPC.address,
          gasTokenAddress: gasTokenContract.address,
          l1GasTokenAddress: this.gasBridgeTestHarness.l1GasTokenAddress,
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
          const { walletClient } = createL1Clients(context.aztecNodeConfig.rpcUrl, MNEMONIC);
          const gasL1 = getContract({
            address: data.l1GasTokenAddress.toString(),
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
        await this.mintPrivateBananas(BigInt(this.ALICE_INITIAL_BANANAS), this.aliceAddress);
        await this.bananaCoin.methods.mint_public(this.aliceAddress, this.ALICE_INITIAL_BANANAS).send().wait();
      },
      () => Promise.resolve(),
    );
  }

  public async applyFundAliceWithPrivateTokens() {
    await this.snapshotManager.snapshot(
      'fund_alice_with_private_tokens',
      async () => {
        await this.mintPrivateTokens(BigInt(this.ALICE_INITIAL_BANANAS));
      },
      () => Promise.resolve(),
    );
  }

  public async applyFundAliceWithGasToken() {
    await this.snapshotManager.snapshot(
      'fund_alice_with_gas_token',
      async () => {
        await this.gasTokenContract.methods.mint_public(this.aliceAddress, this.INITIAL_GAS_BALANCE).send().wait();
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
}
