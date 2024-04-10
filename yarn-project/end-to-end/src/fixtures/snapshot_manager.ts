import { SchnorrAccountContractArtifact, getSchnorrAccount } from '@aztec/accounts/schnorr';
import { type AztecNodeConfig, AztecNodeService, getConfigEnvVars } from '@aztec/aztec-node';
import {
  type AztecAddress,
  BatchCall,
  type CompleteAddress,
  type DebugLogger,
  EthCheatCodes,
  GrumpkinPrivateKey,
  type Wallet,
} from '@aztec/aztec.js';
import { deployInstance, registerContractClass } from '@aztec/aztec.js/deployment';
import { asyncMap } from '@aztec/foundation/async-map';
import { createDebugLogger } from '@aztec/foundation/log';
import { resolver, reviver } from '@aztec/foundation/serialize';
import { type PXEService, createPXEService, getPXEServiceConfig } from '@aztec/pxe';

import { type Anvil, createAnvil } from '@viem/anvil';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'fs';
import { copySync, removeSync } from 'fs-extra/esm';
import getPort from 'get-port';
import { join } from 'path';
import { mnemonicToAccount } from 'viem/accounts';

import { MNEMONIC } from './fixtures.js';
import { getACVMConfig } from './get_acvm_config.js';
import { setupL1Contracts } from './setup_l1_contracts.js';

export type SubsystemsContext = {
  anvil: Anvil;
  acvmConfig: any;
  aztecNode: AztecNodeService;
  aztecNodeConfig: AztecNodeConfig;
  pxe: PXEService;
};

type SnapshotEntry = {
  name: string;
  apply: (context: SubsystemsContext) => Promise<any>;
  restore: (snapshotData: any, context: SubsystemsContext) => Promise<any>;
  snapshotPath: string;
};

export class SnapshotManager {
  private snapshotStack: SnapshotEntry[] = [];
  private context?: SubsystemsContext;
  private livePath: string;
  private logger: DebugLogger;

  constructor(testName: string, private dataPath?: string) {
    this.livePath = this.dataPath ? join(this.dataPath, 'live', testName) : '';
    this.logger = createDebugLogger(`aztec:snapshot_manager:${testName}`);
  }

  public async snapshot<T>(
    name: string,
    apply: (context: SubsystemsContext) => Promise<T>,
    restore: (snapshotData: T, context: SubsystemsContext) => Promise<void> = () => Promise.resolve(),
  ) {
    if (!this.dataPath) {
      // We are running in disabled mode. Just apply the state.
      this.logger.verbose(`No data path given, will not persist any snapshots.`);
      this.context = await this.setupFromFresh();
      this.logger.verbose(`Applying state transition for ${name}...`);
      const snapshotData = await apply(this.context);
      this.logger.verbose(`State transition for ${name} complete.`);
      // Execute the restoration function.
      await restore(snapshotData, this.context);
      return;
    }

    const snapshotPath = join(this.dataPath, 'snapshots', ...this.snapshotStack.map(e => e.name), name, 'snapshot');

    if (existsSync(snapshotPath)) {
      // Snapshot exists. Record entry on stack but do nothing else as we're probably still descending the tree.
      // It's the tests responsibility to call setup() before a test to ensure subsystems get created.
      this.logger.verbose(`Snapshot exists at ${snapshotPath}. Continuing...`);
      this.snapshotStack.push({ name, apply, restore, snapshotPath });
      return;
    }

    // Snapshot didn't exist at snapshotPath, and by definition none of the child snapshots can exist.

    if (!this.context) {
      // We have no subsystem context yet, create it from the top of the snapshot stack (if it exists).
      this.context = await this.setup();
    }

    this.snapshotStack.push({ name, apply, restore, snapshotPath });

    // Apply current state transition.
    this.logger.verbose(`Applying state transition for ${name}...`);
    const snapshotData = await apply(this.context);
    this.logger.verbose(`State transition for ${name} complete.`);

    // Execute the restoration function.
    await restore(snapshotData, this.context);

    // Save the snapshot data.
    const ethCheatCodes = new EthCheatCodes(this.context.aztecNodeConfig.rpcUrl);
    const anvilStateFile = `${this.livePath}/anvil.dat`;
    await ethCheatCodes.dumpChainState(anvilStateFile);
    writeFileSync(`${this.livePath}/${name}.json`, JSON.stringify(snapshotData || {}, resolver));

    // Copy everything to snapshot path.
    // We want it to be atomic, in case multiple processes are racing to create the snapshot.
    this.logger.verbose(`Saving snapshot to ${snapshotPath}...`);
    if (mkdirSync(snapshotPath, { recursive: true })) {
      copySync(this.livePath, snapshotPath);
      this.logger.verbose(`Snapshot copied to ${snapshotPath}.`);
    } else {
      this.logger.verbose(`Snapshot already exists at ${snapshotPath}. Discarding our version.`);
      await this.teardown();
    }
  }

  /**
   * Creates and returns the subsystem context based on the current snapshot stack.
   * If the subsystem context already exists, just return it.
   * If you want to be sure to get a clean snapshot, be sure to call teardown() before calling setup().
   */
  public async setup() {
    // We have no subsystem context yet.
    // If one exists on the snapshot stack, create one from that snapshot.
    // Otherwise create a fresh one.
    if (!this.context) {
      removeSync(this.livePath);
      mkdirSync(this.livePath, { recursive: true });
      const previousSnapshotPath = this.snapshotStack[this.snapshotStack.length - 1]?.snapshotPath;
      if (previousSnapshotPath) {
        this.logger.verbose(`Copying snapshot from ${previousSnapshotPath} to ${this.livePath}...`);
        copySync(previousSnapshotPath, this.livePath);
        this.context = await this.setupFromState(this.livePath);
        // Execute each of the previous snapshots restoration functions in turn.
        await asyncMap(this.snapshotStack, async e => {
          const snapshotData = JSON.parse(readFileSync(`${e.snapshotPath}/${e.name}.json`, 'utf-8'), reviver);
          this.logger.verbose(`Executing restoration function for ${e.name}...`);
          await e.restore(snapshotData, this.context!);
          this.logger.verbose(`Restoration of ${e.name} complete.`);
        });
      } else {
        this.context = await this.setupFromFresh(this.livePath);
      }
    }
    return this.context;
  }

  /**
   * Destroys the current subsystem context.
   */
  public async teardown() {
    if (!this.context) {
      return;
    }
    await this.context.aztecNode.stop();
    await this.context.pxe.stop();
    await this.context.acvmConfig?.cleanup();
    await this.context.anvil.stop();
    this.context = undefined;
    removeSync(this.livePath);
  }

  /**
   * Initializes a fresh set of subsystems.
   * If given a statePath, the state will be written to the path.
   * If there is no statePath, in-memory and temporary state locations will be used.
   */
  private async setupFromFresh(statePath?: string): Promise<SubsystemsContext> {
    this.logger.verbose(`Initializing state...`);

    // Fetch the AztecNode config.
    // TODO: For some reason this is currently the union of a bunch of subsystems. That needs fixing.
    const aztecNodeConfig: AztecNodeConfig = getConfigEnvVars();
    aztecNodeConfig.dataDirectory = statePath;

    // Start anvil. We go via a wrapper script to ensure if the parent dies, anvil dies.
    this.logger.verbose('Starting anvil...');
    const ethereumHostPort = await getPort();
    aztecNodeConfig.rpcUrl = `http://localhost:${ethereumHostPort}`;
    const anvil = createAnvil({ anvilBinary: './scripts/anvil_kill_wrapper.sh', port: ethereumHostPort });
    await anvil.start();

    // Deploy our L1 contracts.
    this.logger.verbose('Deploying L1 contracts...');
    const hdAccount = mnemonicToAccount(MNEMONIC);
    const privKeyRaw = hdAccount.getHdKey().privateKey;
    const publisherPrivKey = privKeyRaw === null ? null : Buffer.from(privKeyRaw);
    const deployL1ContractsValues = await setupL1Contracts(aztecNodeConfig.rpcUrl, hdAccount, this.logger);
    aztecNodeConfig.publisherPrivateKey = `0x${publisherPrivKey!.toString('hex')}`;
    aztecNodeConfig.l1Contracts = deployL1ContractsValues.l1ContractAddresses;
    aztecNodeConfig.l1BlockPublishRetryIntervalMS = 100;

    const acvmConfig = await getACVMConfig(this.logger);
    if (acvmConfig) {
      aztecNodeConfig.acvmWorkingDirectory = acvmConfig.acvmWorkingDirectory;
      aztecNodeConfig.acvmBinaryPath = acvmConfig.expectedAcvmPath;
    }

    this.logger.verbose('Creating and synching an aztec node...');
    const aztecNode = await AztecNodeService.createAndSync(aztecNodeConfig);

    this.logger.verbose('Creating pxe...');
    const pxeConfig = getPXEServiceConfig();
    pxeConfig.dataDirectory = statePath;
    const pxe = await createPXEService(aztecNode, pxeConfig);

    if (statePath) {
      writeFileSync(`${statePath}/aztec_node_config.json`, JSON.stringify(aztecNodeConfig));
    }

    return {
      aztecNodeConfig,
      anvil,
      aztecNode,
      pxe,
      acvmConfig,
    };
  }

  /**
   * Given a statePath, setup the system starting from that state.
   */
  private async setupFromState(statePath: string): Promise<SubsystemsContext> {
    this.logger.verbose(`Initializing with saved state at ${statePath}...`);

    // Load config.
    // TODO: For some reason this is currently the union of a bunch of subsystems. That needs fixing.
    const aztecNodeConfig: AztecNodeConfig = JSON.parse(
      readFileSync(`${statePath}/aztec_node_config.json`, 'utf-8'),
      reviver,
    );
    aztecNodeConfig.dataDirectory = statePath;

    // Start anvil. We go via a wrapper script to ensure if the parent dies, anvil dies.
    const ethereumHostPort = await getPort();
    aztecNodeConfig.rpcUrl = `http://localhost:${ethereumHostPort}`;
    const anvil = createAnvil({ anvilBinary: './scripts/anvil_kill_wrapper.sh', port: ethereumHostPort });
    await anvil.start();
    // Load anvil state.
    const anvilStateFile = `${statePath}/anvil.dat`;
    const ethCheatCodes = new EthCheatCodes(aztecNodeConfig.rpcUrl);
    await ethCheatCodes.loadChainState(anvilStateFile);

    // TODO: Encapsulate this in a NativeAcvm impl.
    const acvmConfig = await getACVMConfig(this.logger);
    if (acvmConfig) {
      aztecNodeConfig.acvmWorkingDirectory = acvmConfig.acvmWorkingDirectory;
      aztecNodeConfig.acvmBinaryPath = acvmConfig.expectedAcvmPath;
    }

    this.logger.verbose('Creating aztec node...');
    const aztecNode = await AztecNodeService.createAndSync(aztecNodeConfig);

    this.logger.verbose('Creating pxe...');
    const pxeConfig = getPXEServiceConfig();
    pxeConfig.dataDirectory = statePath;
    const pxe = await createPXEService(aztecNode, pxeConfig);

    return {
      aztecNodeConfig,
      anvil,
      aztecNode,
      pxe,
      acvmConfig,
    };
  }
}

/**
 * Snapshot 'apply' helper function to add accounts.
 * The 'restore' function is not provided, as it must be a closure within the test context to capture the results.
 */
export const addAccounts =
  (numberOfAccounts: number, logger: DebugLogger) =>
  async ({ pxe }: SubsystemsContext) => {
    // Generate account keys.
    const accountKeys: [GrumpkinPrivateKey, GrumpkinPrivateKey][] = Array.from({ length: numberOfAccounts }).map(_ => [
      GrumpkinPrivateKey.random(),
      GrumpkinPrivateKey.random(),
    ]);

    logger.verbose('Simulating account deployment...');
    const accountManagers = await asyncMap(accountKeys, async ([encPk, signPk]) => {
      const account = getSchnorrAccount(pxe, encPk, signPk, 1);
      // Unfortunately the function below is not stateless and we call it here because it takes a long time to run and
      // the results get stored within the account object. By calling it here we increase the probability of all the
      // accounts being deployed in the same block because it makes the deploy() method basically instant.
      await account.getDeployMethod().then(d =>
        d.prove({
          contractAddressSalt: account.salt,
          skipClassRegistration: true,
          skipPublicDeployment: true,
          universalDeploy: true,
        }),
      );
      return account;
    });

    logger.verbose('Deploying accounts...');
    const txs = await Promise.all(accountManagers.map(account => account.deploy()));
    await Promise.all(txs.map(tx => tx.wait({ interval: 0.1 })));

    return { accountKeys };
  };

/**
 * Registers the contract class used for test accounts and publicly deploys the instances requested.
 * Use this when you need to make a public call to an account contract, such as for requesting a public authwit.
 * @param sender - Wallet to send the deployment tx.
 * @param accountsToDeploy - Which accounts to publicly deploy.
 */
export async function publicDeployAccounts(sender: Wallet, accountsToDeploy: (CompleteAddress | AztecAddress)[]) {
  const accountAddressesToDeploy = accountsToDeploy.map(a => ('address' in a ? a.address : a));
  const instances = await Promise.all(accountAddressesToDeploy.map(account => sender.getContractInstance(account)));
  const batch = new BatchCall(sender, [
    (await registerContractClass(sender, SchnorrAccountContractArtifact)).request(),
    ...instances.map(instance => deployInstance(sender, instance!).request()),
  ]);
  await batch.send().wait();
}
