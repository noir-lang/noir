import { SchnorrAccountContractArtifact, getSchnorrAccount } from '@aztec/accounts/schnorr';
import {
  type AccountWalletWithSecretKey,
  type AztecNode,
  type CompleteAddress,
  type DebugLogger,
  type Fq,
  type Fr,
  type PXE,
  createDebugLogger,
  deployL1Contract,
} from '@aztec/aztec.js';
import { BBCircuitVerifier } from '@aztec/bb-prover';
import { RollupAbi } from '@aztec/l1-artifacts';
import { AvmTestContract } from '@aztec/noir-contracts.js';
import { type PXEService } from '@aztec/pxe';

// @ts-expect-error solc-js doesn't publish its types https://github.com/ethereum/solc-js/issues/689
import solc from 'solc';
import { getContract } from 'viem';

import { waitRegisteredAccountSynced } from '../benchmarks/utils.js';
import { getACVMConfig } from '../fixtures/get_acvm_config.js';
import { getBBConfig } from '../fixtures/get_bb_config.js';
import {
  type ISnapshotManager,
  type SubsystemsContext,
  addAccounts,
  createSnapshotManager,
  publicDeployAccounts,
} from '../fixtures/snapshot_manager.js';
import { setupPXEService } from '../fixtures/utils.js';

const { E2E_DATA_PATH: dataPath } = process.env;

const SALT = 1;

type ProvenSetup = {
  pxe: PXE;
  teardown: () => Promise<void>;
};

/**
 * Simpler version of FullProverTest.
 */
export class FullProverTestAvm {
  private snapshotManager: ISnapshotManager;
  logger: DebugLogger;
  keys: Array<[Fr, Fq]> = [];
  wallets: AccountWalletWithSecretKey[] = [];
  accounts: CompleteAddress[] = [];
  fakeProofsAsset!: AvmTestContract;
  aztecNode!: AztecNode;
  pxe!: PXEService;
  private provenComponents: ProvenSetup[] = [];
  private bbConfigCleanup?: () => Promise<void>;
  private acvmConfigCleanup?: () => Promise<void>;
  circuitProofVerifier?: BBCircuitVerifier;
  provenAsset!: AvmTestContract;
  private context!: SubsystemsContext;

  constructor(testName: string) {
    this.logger = createDebugLogger(`aztec:full_prover_test:${testName}`);
    this.snapshotManager = createSnapshotManager(`full_prover_integration/${testName}`, dataPath);
  }

  /**
   * Adds two state shifts to snapshot manager.
   * 1. Add 2 accounts.
   * 2. Publicly deploy accounts, deploy contract
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

        this.logger.verbose(`Deploying AvmTestContract...`);
        const asset = await AvmTestContract.deploy(this.wallets[0]).send().deployed();
        this.logger.verbose(`AvmTestContract deployed to ${asset.address}`);

        return { contractAddress: asset.address };
      },
      async ({ contractAddress }) => {
        // Restore the token contract state.
        this.fakeProofsAsset = await AvmTestContract.at(contractAddress, this.wallets[0]);
        this.logger.verbose(`AvmTestContract address: ${this.fakeProofsAsset.address}`);
      },
    );
  }

  async setup() {
    this.context = await this.snapshotManager.setup();
    ({ pxe: this.pxe, aztecNode: this.aztecNode } = this.context);

    // Configure a full prover PXE
    const [acvmConfig, bbConfig] = await Promise.all([getACVMConfig(this.logger), getBBConfig(this.logger)]);
    if (!acvmConfig || !bbConfig) {
      throw new Error('Missing ACVM or BB config');
    }

    this.acvmConfigCleanup = acvmConfig.cleanup;
    this.bbConfigCleanup = bbConfig.cleanup;

    if (!bbConfig?.bbWorkingDirectory || !bbConfig?.bbBinaryPath) {
      throw new Error(`Test must be run with BB native configuration`);
    }

    this.circuitProofVerifier = await BBCircuitVerifier.new(bbConfig);

    this.logger.debug(`Configuring the node for real proofs...`);
    await this.aztecNode.setConfig({
      proverAgentConcurrency: 1,
      realProofs: true,
      minTxsPerBlock: 1,
    });

    this.logger.debug(`Main setup completed, initializing full prover PXE and Node...`);

    for (let i = 0; i < 2; i++) {
      const result = await setupPXEService(
        this.aztecNode,
        {
          proverEnabled: true,
          bbBinaryPath: bbConfig?.bbBinaryPath,
          bbWorkingDirectory: bbConfig?.bbWorkingDirectory,
        },
        undefined,
        true,
      );
      this.logger.debug(`Contract address ${this.fakeProofsAsset.address}`);
      await result.pxe.registerContract(this.fakeProofsAsset);

      for (let i = 0; i < 2; i++) {
        await waitRegisteredAccountSynced(
          result.pxe,
          this.keys[i][0],
          this.wallets[i].getCompleteAddress().partialAddress,
        );

        await waitRegisteredAccountSynced(
          this.pxe,
          this.keys[i][0],
          this.wallets[i].getCompleteAddress().partialAddress,
        );
      }

      const account = getSchnorrAccount(result.pxe, this.keys[0][0], this.keys[0][1], SALT);

      await result.pxe.registerContract({
        instance: account.getInstance(),
        artifact: SchnorrAccountContractArtifact,
      });

      const provenWallet = await account.getWallet();
      const asset = await AvmTestContract.at(this.fakeProofsAsset.address, provenWallet);
      this.provenComponents.push({
        pxe: result.pxe,
        teardown: result.teardown,
      });
      this.provenAsset = asset;
    }

    this.logger.debug(`Full prover PXE started!!`);
    return this;
  }

  async teardown() {
    await this.snapshotManager.teardown();

    // Cleanup related to the full prover PXEs
    for (let i = 0; i < this.provenComponents.length; i++) {
      await this.provenComponents[i].teardown();
    }

    await this.bbConfigCleanup?.();
    await this.acvmConfigCleanup?.();
  }

  async deployVerifier() {
    if (!this.circuitProofVerifier) {
      throw new Error('No verifier');
    }

    const { walletClient, publicClient, l1ContractAddresses } = this.context.deployL1ContractsValues;

    const contract = await this.circuitProofVerifier.generateSolidityContract(
      'RootRollupArtifact',
      'UltraVerifier.sol',
    );

    const input = {
      language: 'Solidity',
      sources: {
        'UltraVerifier.sol': {
          content: contract,
        },
      },
      settings: {
        // we require the optimizer
        optimizer: {
          enabled: true,
          runs: 200,
        },
        outputSelection: {
          '*': {
            '*': ['evm.bytecode.object', 'abi'],
          },
        },
      },
    };

    const output = JSON.parse(solc.compile(JSON.stringify(input)));

    const abi = output.contracts['UltraVerifier.sol']['UltraVerifier'].abi;
    const bytecode: string = output.contracts['UltraVerifier.sol']['UltraVerifier'].evm.bytecode.object;

    const verifierAddress = await deployL1Contract(walletClient, publicClient, abi, `0x${bytecode}`);

    this.logger.info(`Deployed Real verifier at ${verifierAddress}`);

    const rollup = getContract({
      abi: RollupAbi,
      address: l1ContractAddresses.rollupAddress.toString(),
      client: walletClient,
    });

    await rollup.write.setVerifier([verifierAddress.toString()]);
    this.logger.info('Rollup only accepts valid proofs now');
  }
}
