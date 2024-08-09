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
import { DocsExampleContract, TokenContract } from '@aztec/noir-contracts.js';

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

export class TokenContractTest {
  static TOKEN_NAME = 'USDC';
  static TOKEN_SYMBOL = 'USD';
  static TOKEN_DECIMALS = 18n;
  private snapshotManager: ISnapshotManager;
  logger: DebugLogger;
  wallets: AccountWallet[] = [];
  accounts: CompleteAddress[] = [];
  asset!: TokenContract;
  tokenSim!: TokenSimulator;
  badAccount!: DocsExampleContract;

  constructor(testName: string) {
    this.logger = createDebugLogger(`aztec:e2e_token_contract:${testName}`);
    this.snapshotManager = createSnapshotManager(`e2e_token_contract/${testName}`, dataPath);
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
      this.accounts = await pxe.getRegisteredAccounts();
      this.wallets.forEach((w, i) => this.logger.verbose(`Wallet ${i} address: ${w.getAddress()}`));
    });

    await this.snapshotManager.snapshot(
      'e2e_token_contract',
      async () => {
        // Create the token contract state.
        // Move this account thing to addAccounts above?
        this.logger.verbose(`Public deploy accounts...`);
        await publicDeployAccounts(this.wallets[0], this.accounts.slice(0, 2));

        this.logger.verbose(`Deploying TokenContract...`);
        const asset = await TokenContract.deploy(
          this.wallets[0],
          this.accounts[0],
          TokenContractTest.TOKEN_NAME,
          TokenContractTest.TOKEN_SYMBOL,
          TokenContractTest.TOKEN_DECIMALS,
        )
          .send()
          .deployed();
        this.logger.verbose(`Token deployed to ${asset.address}`);

        this.logger.verbose(`Deploying bad account...`);
        this.badAccount = await DocsExampleContract.deploy(this.wallets[0]).send().deployed();
        this.logger.verbose(`Deployed to ${this.badAccount.address}.`);

        return { tokenContractAddress: asset.address, badAccountAddress: this.badAccount.address };
      },
      async ({ tokenContractAddress, badAccountAddress }) => {
        // Restore the token contract state.
        this.asset = await TokenContract.at(tokenContractAddress, this.wallets[0]);
        this.logger.verbose(`Token contract address: ${this.asset.address}`);

        this.tokenSim = new TokenSimulator(
          this.asset,
          this.wallets[0],
          this.logger,
          this.accounts.map(a => a.address),
        );

        this.badAccount = await DocsExampleContract.at(badAccountAddress, this.wallets[0]);
        this.logger.verbose(`Bad account address: ${this.badAccount.address}`);

        expect(await this.asset.methods.admin().simulate()).toBe(this.accounts[0].address.toBigInt());
      },
    );

    // TokenContract.artifact.functions.forEach(fn => {
    //   const sig = decodeFunctionSignature(fn.name, fn.parameters);
    //   logger.verbose(`Function ${sig} and the selector: ${FunctionSelector.fromNameAndParameters(fn.name, fn.parameters)}`);
    // });
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
