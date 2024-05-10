import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import {
  type AccountWallet,
  type AztecAddress,
  type AztecNode,
  type ContractArtifact,
  type ContractBase,
  type DebugLogger,
  Fr,
  type PXE,
  type Wallet,
  createDebugLogger,
  getContractInstanceFromDeployParams,
} from '@aztec/aztec.js';
import { type StatefulTestContract } from '@aztec/noir-contracts.js';

import { type ISnapshotManager, addAccounts, createSnapshotManager } from '../fixtures/snapshot_manager.js';

const { E2E_DATA_PATH: dataPath } = process.env;

export class DeployTest {
  private snapshotManager: ISnapshotManager;
  private wallets: AccountWallet[] = [];

  public logger: DebugLogger;
  public pxe!: PXE;
  public wallet!: AccountWallet;
  public aztecNode!: AztecNode;

  constructor(testName: string) {
    this.logger = createDebugLogger(`aztec:e2e_deploy_contract:${testName}`);
    this.snapshotManager = createSnapshotManager(`e2e_deploy_contract/${testName}`, dataPath);
  }

  async setup() {
    await this.applyInitialAccountSnapshot();
    const context = await this.snapshotManager.setup();
    ({ pxe: this.pxe, aztecNode: this.aztecNode } = context);
    return this;
  }

  async teardown() {
    await this.snapshotManager.teardown();
  }

  private async applyInitialAccountSnapshot() {
    await this.snapshotManager.snapshot(
      'initial_account',
      addAccounts(1, this.logger),
      async ({ accountKeys }, { pxe }) => {
        const accountManagers = accountKeys.map(ak => getSchnorrAccount(pxe, ak[0], ak[1], 1));
        this.wallets = await Promise.all(accountManagers.map(a => a.getWallet()));
        this.wallets.forEach((w, i) => this.logger.verbose(`Wallet ${i} address: ${w.getAddress()}`));
        this.wallet = this.wallets[0];
      },
    );
  }

  async registerContract<T extends ContractBase>(
    wallet: Wallet,
    contractArtifact: ContractArtifactClass<T>,
    opts: {
      salt?: Fr;
      publicKeysHash?: Fr;
      initArgs?: any[];
      constructorName?: string;
      deployer?: AztecAddress;
    } = {},
  ): Promise<T> {
    const { salt, publicKeysHash, initArgs, constructorName, deployer } = opts;
    const instance = getContractInstanceFromDeployParams(contractArtifact.artifact, {
      constructorArgs: initArgs ?? [],
      constructorArtifact: constructorName,
      salt,
      publicKeysHash,
      deployer,
    });
    await wallet.registerContract({ artifact: contractArtifact.artifact, instance });
    return contractArtifact.at(instance.address, wallet);
  }

  async registerRandomAccount(): Promise<AztecAddress> {
    const completeAddress = await this.pxe.registerAccount(Fr.random(), Fr.random());
    return completeAddress.address;
  }
}

export type StatefulContractCtorArgs = Parameters<StatefulTestContract['methods']['constructor']>;

export type ContractArtifactClass<T extends ContractBase> = {
  at(address: AztecAddress, wallet: Wallet): Promise<T>;
  artifact: ContractArtifact;
};
