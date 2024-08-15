import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { type AztecNodeConfig } from '@aztec/aztec-node';
import {
  type AccountWallet,
  AztecAddress,
  type AztecNode,
  type CompleteAddress,
  type DebugLogger,
  EthAddress,
  type PXE,
  createDebugLogger,
} from '@aztec/aztec.js';
import { createL1Clients } from '@aztec/ethereum';
import { InboxAbi, OutboxAbi, PortalERC20Abi, RollupAbi, TokenPortalAbi } from '@aztec/l1-artifacts';
import { TokenBridgeContract, TokenContract } from '@aztec/noir-contracts.js';

import { type Chain, type HttpTransport, type PublicClient, getContract } from 'viem';

import { MNEMONIC } from '../fixtures/fixtures.js';
import {
  type ISnapshotManager,
  type SubsystemsContext,
  addAccounts,
  createSnapshotManager,
  publicDeployAccounts,
} from '../fixtures/snapshot_manager.js';
import { CrossChainTestHarness } from '../shared/cross_chain_test_harness.js';

const { E2E_DATA_PATH: dataPath } = process.env;

export class PublicCrossChainMessagingContractTest {
  private snapshotManager: ISnapshotManager;
  logger: DebugLogger;
  wallets: AccountWallet[] = [];
  accounts: CompleteAddress[] = [];
  aztecNode!: AztecNode;
  pxe!: PXE;
  aztecNodeConfig!: AztecNodeConfig;

  publicClient!: PublicClient<HttpTransport, Chain> | undefined;

  user1Wallet!: AccountWallet;
  user2Wallet!: AccountWallet;
  crossChainTestHarness!: CrossChainTestHarness;
  ethAccount!: EthAddress;
  ownerAddress!: AztecAddress;
  l2Token!: TokenContract;
  l2Bridge!: TokenBridgeContract;

  rollup!: any; // GetContractReturnType<typeof RollupAbi> | undefined;
  inbox!: any; // GetContractReturnType<typeof InboxAbi> | undefined;
  outbox!: any; // GetContractReturnType<typeof OutboxAbi> | undefined;

  constructor(testName: string) {
    this.logger = createDebugLogger(`aztec:e2e_public_cross_chain_messaging:${testName}`);
    this.snapshotManager = createSnapshotManager(`e2e_public_cross_chain_messaging/${testName}`, dataPath);
  }

  async assumeProven() {
    await this.rollup.write.setAssumeProvenUntilBlockNumber([await this.rollup.read.pendingBlockCount()]);
  }

  async setup() {
    const { aztecNode, pxe, aztecNodeConfig } = await this.snapshotManager.setup();
    this.aztecNode = aztecNode;
    this.pxe = pxe;
    this.aztecNodeConfig = aztecNodeConfig;
  }

  snapshot = <T>(
    name: string,
    apply: (context: SubsystemsContext) => Promise<T>,
    restore: (snapshotData: T, context: SubsystemsContext) => Promise<void> = () => Promise.resolve(),
  ): Promise<void> => this.snapshotManager.snapshot(name, apply, restore);

  async teardown() {
    await this.snapshotManager.teardown();
  }

  async applyBaseSnapshots() {
    // Note that we are using the same `pxe`, `aztecNodeConfig` and `aztecNode` across all snapshots.
    // This is to not have issues with different networks.

    await this.snapshotManager.snapshot(
      '3_accounts',
      addAccounts(3, this.logger),
      async ({ accountKeys }, { pxe, aztecNodeConfig, aztecNode, deployL1ContractsValues }) => {
        const accountManagers = accountKeys.map(ak => getSchnorrAccount(pxe, ak[0], ak[1], 1));
        this.wallets = await Promise.all(accountManagers.map(a => a.getWallet()));
        this.wallets.forEach((w, i) => this.logger.verbose(`Wallet ${i} address: ${w.getAddress()}`));
        this.accounts = await pxe.getRegisteredAccounts();

        this.rollup = getContract({
          address: deployL1ContractsValues.l1ContractAddresses.rollupAddress.toString(),
          abi: RollupAbi,
          client: deployL1ContractsValues.walletClient,
        });

        this.user1Wallet = this.wallets[0];
        this.user2Wallet = this.wallets[1];

        this.pxe = pxe;
        this.aztecNode = aztecNode;
        this.aztecNodeConfig = aztecNodeConfig;
      },
    );

    await this.snapshotManager.snapshot(
      'e2e_public_cross_chain_messaging',
      async () => {
        // Create the token contract state.
        // Move this account thing to addAccounts above?
        this.logger.verbose(`Public deploy accounts...`);
        await publicDeployAccounts(this.wallets[0], this.accounts.slice(0, 3));

        const { publicClient, walletClient } = createL1Clients(this.aztecNodeConfig.l1RpcUrl, MNEMONIC);

        this.logger.verbose(`Setting up cross chain harness...`);
        this.crossChainTestHarness = await CrossChainTestHarness.new(
          this.aztecNode,
          this.pxe,
          publicClient,
          walletClient,
          this.wallets[0],
          this.logger,
        );

        this.logger.verbose(`L2 token deployed to: ${this.crossChainTestHarness.l2Token.address}`);

        return this.toCrossChainContext();
      },
      async crossChainContext => {
        this.l2Token = await TokenContract.at(crossChainContext.l2Token, this.user1Wallet);
        this.l2Bridge = await TokenBridgeContract.at(crossChainContext.l2Bridge, this.user1Wallet);

        // There is an issue with the reviver so we are getting strings sometimes. Working around it here.
        this.ethAccount = EthAddress.fromString(crossChainContext.ethAccount.toString());
        this.ownerAddress = AztecAddress.fromString(crossChainContext.ownerAddress.toString());
        const tokenPortalAddress = EthAddress.fromString(crossChainContext.tokenPortal.toString());

        const { publicClient, walletClient } = createL1Clients(this.aztecNodeConfig.l1RpcUrl, MNEMONIC);

        const inbox = getContract({
          address: this.aztecNodeConfig.l1Contracts.inboxAddress.toString(),
          abi: InboxAbi,
          client: walletClient,
        });
        const outbox = getContract({
          address: this.aztecNodeConfig.l1Contracts.outboxAddress.toString(),
          abi: OutboxAbi,
          client: walletClient,
        });

        const tokenPortal = getContract({
          address: tokenPortalAddress.toString(),
          abi: TokenPortalAbi,
          client: walletClient,
        });
        const underlyingERC20 = getContract({
          address: crossChainContext.underlying.toString(),
          abi: PortalERC20Abi,
          client: walletClient,
        });

        this.crossChainTestHarness = new CrossChainTestHarness(
          this.aztecNode,
          this.pxe,
          this.logger,
          this.l2Token,
          this.l2Bridge,
          this.ethAccount,
          tokenPortalAddress,
          tokenPortal,
          underlyingERC20,
          inbox,
          outbox,
          publicClient,
          walletClient,
          this.ownerAddress,
          this.aztecNodeConfig.l1Contracts,
          this.user1Wallet,
        );

        this.publicClient = publicClient;
        this.inbox = inbox;
        this.outbox = outbox;
      },
    );
  }

  toCrossChainContext(): CrossChainContext {
    return {
      l2Token: this.crossChainTestHarness.l2Token.address,
      l2Bridge: this.crossChainTestHarness.l2Bridge.address,
      tokenPortal: this.crossChainTestHarness.tokenPortal.address,
      underlying: EthAddress.fromString(this.crossChainTestHarness.underlyingERC20.address),
      ethAccount: this.crossChainTestHarness.ethAccount,
      ownerAddress: this.crossChainTestHarness.ownerAddress,
      inbox: EthAddress.fromString(this.crossChainTestHarness.inbox.address),
      outbox: EthAddress.fromString(this.crossChainTestHarness.outbox.address),
    };
  }
}

type CrossChainContext = {
  l2Token: AztecAddress;
  l2Bridge: AztecAddress;
  tokenPortal: EthAddress;
  underlying: EthAddress;
  ethAccount: EthAddress;
  ownerAddress: AztecAddress;
  inbox: EthAddress;
  outbox: EthAddress;
};
