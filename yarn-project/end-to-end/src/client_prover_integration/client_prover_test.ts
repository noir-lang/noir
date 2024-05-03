import { SchnorrAccountContractArtifact, getSchnorrAccount } from '@aztec/accounts/schnorr';
import {
  type AccountWalletWithSecretKey,
  type AztecNode,
  type CompleteAddress,
  type DebugLogger,
  ExtendedNote,
  type Fq,
  Fr,
  Note,
  type TxHash,
  computeSecretHash,
  createDebugLogger,
} from '@aztec/aztec.js';
import { TokenContract } from '@aztec/noir-contracts.js';
import { BBNativeProofCreator, type PXEService } from '@aztec/pxe';

import * as fs from 'fs/promises';

import { waitRegisteredAccountSynced } from '../benchmarks/utils.js';
import {
  SnapshotManager,
  type SubsystemsContext,
  addAccounts,
  publicDeployAccounts,
} from '../fixtures/snapshot_manager.js';
import { getBBConfig, setupPXEService } from '../fixtures/utils.js';
import { TokenSimulator } from '../simulators/token_simulator.js';

const { E2E_DATA_PATH: dataPath } = process.env;

const SALT = 1;

/**
 * Largely taken from the e2e_token_contract test file. We deploy 2 accounts and a token contract.
 * However, we then setup a second PXE with a full prover instance.
 * We configure this instance with all of the accounts and contracts.
 * We then prove and verify transactions created via this full prover PXE.
 */

export class ClientProverTest {
  static TOKEN_NAME = 'Aztec Token';
  static TOKEN_SYMBOL = 'AZT';
  static TOKEN_DECIMALS = 18n;
  private snapshotManager: SnapshotManager;
  logger: DebugLogger;
  keys: Array<[Fr, Fq]> = [];
  wallets: AccountWalletWithSecretKey[] = [];
  accounts: CompleteAddress[] = [];
  asset!: TokenContract;
  tokenSim!: TokenSimulator;
  aztecNode!: AztecNode;
  pxe!: PXEService;
  fullProverPXE!: PXEService;
  provenAsset!: TokenContract;
  provenPXETeardown?: () => Promise<void>;
  private directoryToCleanup?: string;
  proofCreator?: BBNativeProofCreator;

  constructor(testName: string) {
    this.logger = createDebugLogger(`aztec:client_prover_test:${testName}`);
    this.snapshotManager = new SnapshotManager(`client_prover_integration/${testName}`, dataPath);
  }

  /**
   * Adds two state shifts to snapshot manager.
   * 1. Add 2 accounts.
   * 2. Publicly deploy accounts, deploy token contract
   */
  async applyBaseSnapshots() {
    await this.snapshotManager.snapshot('2_accounts', addAccounts(2, this.logger), async ({ accountKeys }, { pxe }) => {
      this.keys = accountKeys;
      const accountManagers = accountKeys.map(ak => getSchnorrAccount(pxe, ak[0], ak[1], SALT));
      this.wallets = await Promise.all(accountManagers.map(a => a.getWallet()));
      this.accounts = await pxe.getRegisteredAccounts();
      this.wallets.forEach((w, i) => this.logger.verbose(`Wallet ${i} address: ${w.getAddress()}`));
    });

    await this.snapshotManager.snapshot(
      'client_prover_integration',
      async () => {
        // Create the token contract state.
        // Move this account thing to addAccounts above?
        this.logger.verbose(`Public deploy accounts...`);
        await publicDeployAccounts(this.wallets[0], this.accounts.slice(0, 2));

        this.logger.verbose(`Deploying TokenContract...`);
        const asset = await TokenContract.deploy(
          this.wallets[0],
          this.accounts[0],
          ClientProverTest.TOKEN_NAME,
          ClientProverTest.TOKEN_SYMBOL,
          ClientProverTest.TOKEN_DECIMALS,
        )
          .send()
          .deployed();
        this.logger.verbose(`Token deployed to ${asset.address}`);

        return { tokenContractAddress: asset.address };
      },
      async ({ tokenContractAddress }) => {
        // Restore the token contract state.
        this.asset = await TokenContract.at(tokenContractAddress, this.wallets[0]);
        this.logger.verbose(`Token contract address: ${this.asset.address}`);

        this.tokenSim = new TokenSimulator(
          this.asset,
          this.logger,
          this.accounts.map(a => a.address),
        );

        expect(await this.asset.methods.admin().simulate()).toBe(this.accounts[0].address.toBigInt());
      },
    );
  }

  async setup() {
    const context = await this.snapshotManager.setup();
    ({ pxe: this.pxe, aztecNode: this.aztecNode } = context);

    // Configure a full prover PXE
    const bbConfig = await getBBConfig(this.logger);
    this.directoryToCleanup = bbConfig?.directoryToCleanup;

    if (!bbConfig?.bbWorkingDirectory || !bbConfig?.expectedBBPath) {
      throw new Error(`Test must be run with BB native configuration`);
    }

    this.proofCreator = new BBNativeProofCreator(bbConfig?.expectedBBPath, bbConfig?.bbWorkingDirectory);

    this.logger.debug(`Main setup completed, initializing full prover PXE...`);
    ({ pxe: this.fullProverPXE, teardown: this.provenPXETeardown } = await setupPXEService(
      0,
      this.aztecNode,
      {
        proverEnabled: false,
        bbBinaryPath: bbConfig?.expectedBBPath,
        bbWorkingDirectory: bbConfig?.bbWorkingDirectory,
      },
      undefined,
      true,
      this.proofCreator,
    ));
    this.logger.debug(`Contract address ${this.asset.address}`);
    await this.fullProverPXE.registerContract(this.asset);

    for (let i = 0; i < 2; i++) {
      await waitRegisteredAccountSynced(
        this.fullProverPXE,
        this.keys[i][0],
        this.wallets[i].getCompleteAddress().partialAddress,
      );

      await waitRegisteredAccountSynced(this.pxe, this.keys[i][0], this.wallets[i].getCompleteAddress().partialAddress);
    }

    const account = getSchnorrAccount(this.fullProverPXE, this.keys[0][0], this.keys[0][1], SALT);

    await this.fullProverPXE.registerContract({
      instance: account.getInstance(),
      artifact: SchnorrAccountContractArtifact,
    });

    const provenWallet = await account.getWallet();
    this.provenAsset = await TokenContract.at(this.asset.address, provenWallet);
    this.logger.debug(`Full prover PXE started!!`);
    return this;
  }

  snapshot = <T>(
    name: string,
    apply: (context: SubsystemsContext) => Promise<T>,
    restore: (snapshotData: T, context: SubsystemsContext) => Promise<void> = () => Promise.resolve(),
  ): Promise<void> => this.snapshotManager.snapshot(name, apply, restore);

  async teardown() {
    await this.snapshotManager.teardown();

    // Cleanup related to the second 'full prover' PXE
    await this.provenPXETeardown?.();

    if (this.directoryToCleanup) {
      await fs.rm(this.directoryToCleanup, { recursive: true, force: true });
    }
  }

  async addPendingShieldNoteToPXE(accountIndex: number, amount: bigint, secretHash: Fr, txHash: TxHash) {
    const note = new Note([new Fr(amount), secretHash]);
    const extendedNote = new ExtendedNote(
      note,
      this.accounts[accountIndex].address,
      this.asset.address,
      TokenContract.storage.pending_shields.slot,
      TokenContract.notes.TransparentNote.id,
      txHash,
    );
    await this.wallets[accountIndex].addNote(extendedNote);
  }

  async applyMintSnapshot() {
    await this.snapshotManager.snapshot(
      'mint',
      async () => {
        const { asset, accounts } = this;
        const amount = 10000n;

        this.logger.verbose(`Minting ${amount} publicly...`);
        await asset.methods.mint_public(accounts[0].address, amount).send().wait();

        this.logger.verbose(`Minting ${amount} privately...`);
        const secret = Fr.random();
        const secretHash = computeSecretHash(secret);
        const receipt = await asset.methods.mint_private(amount, secretHash).send().wait();

        await this.addPendingShieldNoteToPXE(0, amount, secretHash, receipt.txHash);
        const txClaim = asset.methods.redeem_shield(accounts[0].address, amount, secret).send();
        await txClaim.wait({ debug: true });
        this.logger.verbose(`Minting complete.`);

        return { amount };
      },
      async ({ amount }) => {
        const {
          asset,
          accounts: [{ address }],
          tokenSim,
        } = this;
        tokenSim.mintPublic(address, amount);

        const publicBalance = await asset.methods.balance_of_public(address).simulate();
        this.logger.verbose(`Public balance of wallet 0: ${publicBalance}`);
        expect(publicBalance).toEqual(this.tokenSim.balanceOfPublic(address));

        tokenSim.mintPrivate(amount);
        tokenSim.redeemShield(address, amount);
        const privateBalance = await asset.methods.balance_of_private(address).simulate();
        this.logger.verbose(`Private balance of wallet 0: ${privateBalance}`);
        expect(privateBalance).toEqual(tokenSim.balanceOfPrivate(address));

        const totalSupply = await asset.methods.total_supply().simulate();
        this.logger.verbose(`Total supply: ${totalSupply}`);
        expect(totalSupply).toEqual(tokenSim.totalSupply);

        return Promise.resolve();
      },
    );
  }
}
