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
  type PXE,
  type TxHash,
  computeSecretHash,
  createDebugLogger, // TODO(#7373): Deploy honk solidity verifier
  // deployL1Contract,
} from '@aztec/aztec.js';
import { BBCircuitVerifier } from '@aztec/bb-prover';
// import { RollupAbi } from '@aztec/l1-artifacts';
import { TokenContract } from '@aztec/noir-contracts.js';
import { type PXEService } from '@aztec/pxe';

// TODO(#7373): Deploy honk solidity verifier
// // @ts-expect-error solc-js doesn't publish its types https://github.com/ethereum/solc-js/issues/689
// import solc from 'solc';
// import { getContract } from 'viem';
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
import { TokenSimulator } from '../simulators/token_simulator.js';

const { E2E_DATA_PATH: dataPath } = process.env;

const SALT = 1;

type ProvenSetup = {
  pxe: PXE;
  teardown: () => Promise<void>;
};

/**
 * Largely taken from the e2e_token_contract test file. We deploy 2 accounts and a token contract.
 * However, we then setup a second PXE with a full prover instance.
 * We configure this instance with all of the accounts and contracts.
 * We then prove and verify transactions created via this full prover PXE.
 */

export class FullProverTest {
  static TOKEN_NAME = 'USDC';
  static TOKEN_SYMBOL = 'USD';
  static TOKEN_DECIMALS = 18n;
  private snapshotManager: ISnapshotManager;
  logger: DebugLogger;
  keys: Array<[Fr, Fq]> = [];
  wallets: AccountWalletWithSecretKey[] = [];
  accounts: CompleteAddress[] = [];
  fakeProofsAsset!: TokenContract;
  tokenSim!: TokenSimulator;
  aztecNode!: AztecNode;
  pxe!: PXEService;
  private provenComponents: ProvenSetup[] = [];
  private bbConfigCleanup?: () => Promise<void>;
  private acvmConfigCleanup?: () => Promise<void>;
  circuitProofVerifier?: BBCircuitVerifier;
  provenAssets: TokenContract[] = [];
  private context!: SubsystemsContext;

  constructor(testName: string, private minNumberOfTxsPerBlock: number) {
    this.logger = createDebugLogger(`aztec:full_prover_test:${testName}`);
    this.snapshotManager = createSnapshotManager(`full_prover_integration/${testName}`, dataPath);
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
          FullProverTest.TOKEN_NAME,
          FullProverTest.TOKEN_SYMBOL,
          FullProverTest.TOKEN_DECIMALS,
        )
          .send()
          .deployed();
        this.logger.verbose(`Token deployed to ${asset.address}`);

        return { tokenContractAddress: asset.address };
      },
      async ({ tokenContractAddress }) => {
        // Restore the token contract state.
        this.fakeProofsAsset = await TokenContract.at(tokenContractAddress, this.wallets[0]);
        this.logger.verbose(`Token contract address: ${this.fakeProofsAsset.address}`);

        this.tokenSim = new TokenSimulator(
          this.fakeProofsAsset,
          this.wallets[0],
          this.logger,
          this.accounts.map(a => a.address),
        );

        expect(await this.fakeProofsAsset.methods.admin().simulate()).toBe(this.accounts[0].address.toBigInt());
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
      proverAgentConcurrency: 2,
      realProofs: true,
      minTxsPerBlock: this.minNumberOfTxsPerBlock,
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
      const asset = await TokenContract.at(this.fakeProofsAsset.address, provenWallet);
      this.provenComponents.push({
        pxe: result.pxe,
        teardown: result.teardown,
      });
      this.provenAssets.push(asset);
    }

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

    // Cleanup related to the full prover PXEs
    for (let i = 0; i < this.provenComponents.length; i++) {
      await this.provenComponents[i].teardown();
    }

    await this.bbConfigCleanup?.();
    await this.acvmConfigCleanup?.();
  }

  async addPendingShieldNoteToPXE(accountIndex: number, amount: bigint, secretHash: Fr, txHash: TxHash) {
    const note = new Note([new Fr(amount), secretHash]);
    const extendedNote = new ExtendedNote(
      note,
      this.accounts[accountIndex].address,
      this.fakeProofsAsset.address,
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
        const { fakeProofsAsset: asset, accounts } = this;
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
          fakeProofsAsset: asset,
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

  deployVerifier() {
    if (!this.circuitProofVerifier) {
      throw new Error('No verifier');
    }

    // TODO(#7373): Deploy honk solidity verifier
    return Promise.resolve();
    // const { walletClient, publicClient, l1ContractAddresses } = this.context.deployL1ContractsValues;

    // const contract = await this.circuitProofVerifier.generateSolidityContract(
    //   'RootRollupArtifact',
    //   'UltraVerifier.sol',
    // );

    // const input = {
    //   language: 'Solidity',
    //   sources: {
    //     'UltraVerifier.sol': {
    //       content: contract,
    //     },
    //   },
    //   settings: {
    //     // we require the optimizer
    //     optimizer: {
    //       enabled: true,
    //       runs: 200,
    //     },
    //     evmVersion: 'paris',
    //     outputSelection: {
    //       '*': {
    //         '*': ['evm.bytecode.object', 'abi'],
    //       },
    //     },
    //   },
    // };

    // const output = JSON.parse(solc.compile(JSON.stringify(input)));

    // const abi = output.contracts['UltraVerifier.sol']['UltraVerifier'].abi;
    // const bytecode: string = output.contracts['UltraVerifier.sol']['UltraVerifier'].evm.bytecode.object;

    // const verifierAddress = await deployL1Contract(walletClient, publicClient, abi, `0x${bytecode}`);

    // this.logger.info(`Deployed Real verifier at ${verifierAddress}`);

    // const rollup = getContract({
    //   abi: RollupAbi,
    //   address: l1ContractAddresses.rollupAddress.toString(),
    //   client: walletClient,
    // });

    // await rollup.write.setVerifier([verifierAddress.toString()]);
    // this.logger.info('Rollup only accepts valid proofs now');
  }
}
