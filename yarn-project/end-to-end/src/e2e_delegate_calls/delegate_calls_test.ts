import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { type AccountWallet, type DebugLogger, createDebugLogger } from '@aztec/aztec.js';
import { DelegatedOnContract, DelegatorContract } from '@aztec/noir-contracts.js';

import {
  type ISnapshotManager,
  type SubsystemsContext,
  addAccounts,
  createSnapshotManager,
} from '../fixtures/snapshot_manager.js';

const { E2E_DATA_PATH: dataPath } = process.env;

export class DelegateCallsTest {
  private snapshotManager: ISnapshotManager;
  logger: DebugLogger;
  wallet!: AccountWallet;
  delegatorContract!: DelegatorContract;
  delegatedOnContract!: DelegatedOnContract;

  constructor(testName: string) {
    this.logger = createDebugLogger(`aztec:e2e_delegate_calls:${testName}`);
    this.snapshotManager = createSnapshotManager(`e2e_delegate_calls/${testName}`, dataPath);
  }

  /**
   * Adds two state shifts to snapshot manager.
   * 1. Add 3 accounts.
   * 2. Publicly deploy accounts, deploy token contract and a "bad account".
   */
  async applyBaseSnapshots() {
    await this.snapshotManager.snapshot('accounts', addAccounts(1, this.logger), async ({ accountKeys }, { pxe }) => {
      const accountManager = getSchnorrAccount(pxe, accountKeys[0][0], accountKeys[0][1], 1);
      this.wallet = await accountManager.getWallet();
      this.logger.verbose(`Wallet address: ${this.wallet.getAddress()}`);
    });

    await this.snapshotManager.snapshot(
      'e2e_delegate_calls',
      async () => {
        this.logger.verbose(`Deploying DelegatorContract...`);
        this.delegatorContract = await DelegatorContract.deploy(this.wallet).send().deployed();
        this.logger.verbose(`Delegator deployed to ${this.delegatorContract.address}`);

        this.logger.verbose(`Deploying DelegatedOnContract...`);
        this.delegatedOnContract = await DelegatedOnContract.deploy(this.wallet).send().deployed();

        return {
          delegatorContractAddress: this.delegatorContract.address,
          delegatedOnContractAddress: this.delegatedOnContract.address,
        };
      },
      async ({ delegatorContractAddress, delegatedOnContractAddress }) => {
        // Restore the token contract state.
        this.delegatorContract = await DelegatorContract.at(delegatorContractAddress, this.wallet);
        this.logger.verbose(`Delegator contract address: ${this.delegatorContract.address}`);

        this.delegatedOnContract = await DelegatedOnContract.at(delegatedOnContractAddress, this.wallet);
        this.logger.verbose(`DelegatedOn address: ${this.delegatedOnContract.address}`);
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
}
