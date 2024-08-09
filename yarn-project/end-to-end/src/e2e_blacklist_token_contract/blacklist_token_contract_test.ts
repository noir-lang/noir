import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import {
  type AccountWallet,
  type CompleteAddress,
  type DebugLogger,
  ExtendedNote,
  Fr,
  Note,
  type TxHash,
  computeSecretHash,
  createDebugLogger,
} from '@aztec/aztec.js';
import { DocsExampleContract, TokenBlacklistContract, type TokenContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import {
  type ISnapshotManager,
  type SubsystemsContext,
  addAccounts,
  createSnapshotManager,
  publicDeployAccounts,
} from '../fixtures/snapshot_manager.js';
import { TokenSimulator } from '../simulators/token_simulator.js';

const { E2E_DATA_PATH: dataPath } = process.env;

export class Role {
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

  toNoirStruct() {
    // We need to use lowercase identifiers as those are what the noir interface expects
    // eslint-disable-next-line camelcase
    return { is_admin: this.isAdmin, is_minter: this.isMinter, is_blacklisted: this.isBlacklisted };
  }
}

export class BlacklistTokenContractTest {
  // A low delay is really poor ux, but we need to keep it low for the tests to run "quickly".
  // This value MUST match the same value that we have in the contract
  static DELAY = 2;

  private snapshotManager: ISnapshotManager;
  logger: DebugLogger;
  wallets: AccountWallet[] = [];
  accounts: CompleteAddress[] = [];
  asset!: TokenBlacklistContract;
  tokenSim!: TokenSimulator;
  badAccount!: DocsExampleContract;

  admin!: AccountWallet;
  other!: AccountWallet;
  blacklisted!: AccountWallet;

  constructor(testName: string) {
    this.logger = createDebugLogger(`aztec:e2e_blacklist_token_contract:${testName}`);
    this.snapshotManager = createSnapshotManager(`e2e_blacklist_token_contract/${testName}`, dataPath);
  }

  async mineBlocks(amount: number = BlacklistTokenContractTest.DELAY) {
    for (let i = 0; i < amount; ++i) {
      await this.asset.methods.get_roles(this.admin.getAddress()).send().wait();
    }
  }

  /**
   * Adds two state shifts to snapshot manager.
   * 1. Add 3 accounts.
   * 2. Publicly deploy accounts, deploy token contract and a "bad account".
   */
  async applyBaseSnapshots() {
    // Adding a timeout of 2 minutes in here such that it is propagated to the underlying tests
    jest.setTimeout(120_000);

    await this.snapshotManager.snapshot('3_accounts', addAccounts(3, this.logger), async ({ accountKeys }, { pxe }) => {
      const accountManagers = accountKeys.map(ak => getSchnorrAccount(pxe, ak[0], ak[1], 1));
      this.wallets = await Promise.all(accountManagers.map(a => a.getWallet()));
      this.admin = this.wallets[0];
      this.other = this.wallets[1];
      this.blacklisted = this.wallets[2];
      this.accounts = await pxe.getRegisteredAccounts();
      this.wallets.forEach((w, i) => this.logger.verbose(`Wallet ${i} address: ${w.getAddress()}`));
    });

    await this.snapshotManager.snapshot(
      'e2e_blacklist_token_contract',
      async () => {
        // Create the token contract state.
        // Move this account thing to addAccounts above?
        this.logger.verbose(`Public deploy accounts...`);
        await publicDeployAccounts(this.wallets[0], this.accounts.slice(0, 3));

        this.logger.verbose(`Deploying TokenContract...`);
        this.asset = await TokenBlacklistContract.deploy(this.admin, this.admin.getAddress()).send().deployed();
        this.logger.verbose(`Token deployed to ${this.asset.address}`);

        this.logger.verbose(`Deploying bad account...`);
        this.badAccount = await DocsExampleContract.deploy(this.wallets[0]).send().deployed();
        this.logger.verbose(`Deployed to ${this.badAccount.address}.`);

        await this.mineBlocks();

        return { tokenContractAddress: this.asset.address, badAccountAddress: this.badAccount.address };
      },
      async ({ tokenContractAddress, badAccountAddress }) => {
        // Restore the token contract state.
        this.asset = await TokenBlacklistContract.at(tokenContractAddress, this.wallets[0]);
        this.logger.verbose(`Token contract address: ${this.asset.address}`);

        this.tokenSim = new TokenSimulator(
          this.asset as unknown as TokenContract,
          this.wallets[0],
          this.logger,
          this.accounts.map(a => a.address),
        );

        this.badAccount = await DocsExampleContract.at(badAccountAddress, this.wallets[0]);
        this.logger.verbose(`Bad account address: ${this.badAccount.address}`);

        expect(await this.asset.methods.get_roles(this.admin.getAddress()).simulate()).toEqual(
          new Role().withAdmin().toNoirStruct(),
        );
      },
    );
  }

  async setup() {
    await this.snapshotManager.setup();
  }

  snapshot = <T>(
    name: string,
    apply: (context: SubsystemsContext) => Promise<T>,
    restore: (snapshotData: T, context: SubsystemsContext) => Promise<void> = () => Promise.resolve(),
  ): Promise<void> => this.snapshotManager.snapshot(name, apply, restore);

  async teardown() {
    await this.snapshotManager.teardown();
  }

  async addPendingShieldNoteToPXE(accountIndex: number, amount: bigint, secretHash: Fr, txHash: TxHash) {
    const note = new Note([new Fr(amount), secretHash]);
    const extendedNote = new ExtendedNote(
      note,
      this.accounts[accountIndex].address,
      this.asset.address,
      TokenBlacklistContract.storage.pending_shields.slot,
      TokenBlacklistContract.notes.TransparentNote.id,
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

        const adminMinterRole = new Role().withAdmin().withMinter();
        await this.asset
          .withWallet(this.admin)
          .methods.update_roles(this.admin.getAddress(), adminMinterRole.toNoirStruct())
          .send()
          .wait();

        const blacklistRole = new Role().withBlacklisted();
        await this.asset
          .withWallet(this.admin)
          .methods.update_roles(this.blacklisted.getAddress(), blacklistRole.toNoirStruct())
          .send()
          .wait();

        await this.mineBlocks(); // This gets us past the block of change

        expect(await this.asset.methods.get_roles(this.admin.getAddress()).simulate()).toEqual(
          adminMinterRole.toNoirStruct(),
        );

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
