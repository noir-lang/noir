import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer, Contract, TxStatus, computeMessageSecretHash } from '@aztec/aztec.js';
import { AztecAddress, EthAddress, Fr, Point } from '@aztec/circuits.js';
import { DeployL1Contracts } from '@aztec/ethereum';
import { DebugLogger } from '@aztec/foundation/log';
import { PublicClient, HttpTransport, Chain, getContract } from 'viem';
import { deployAndInitializeNonNativeL2TokenContracts, pointToPublicKey } from '../utils.js';
import { OutboxAbi } from '@aztec/l1-artifacts';
import { sha256ToField } from '@aztec/foundation/crypto';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';

/**
 * A Class for testing cross chain interactions, contains common interactions
 * shared between cross chain tests.
 */
export class CrossChainTestHarness {
  static async new(
    initialBalance: bigint,
    aztecNode: AztecNodeService,
    aztecRpcServer: AztecRPCServer,
    deployL1ContractsValues: DeployL1Contracts,
    accounts: AztecAddress[],
    logger: DebugLogger,
  ): Promise<CrossChainTestHarness> {
    const walletClient = deployL1ContractsValues.walletClient;
    const publicClient = deployL1ContractsValues.publicClient;

    const ethAccount = EthAddress.fromString((await walletClient.getAddresses())[0]);
    const [ownerAddress, receiver] = accounts;
    const ownerPub = await aztecRpcServer.getAccountPublicKey(ownerAddress);

    const outbox = getContract({
      address: deployL1ContractsValues.outboxAddress.toString(),
      abi: OutboxAbi,
      publicClient,
    });

    // Deploy and initialize all required contracts
    logger('Deploying Portal, initializing and deploying l2 contract...');
    const contracts = await deployAndInitializeNonNativeL2TokenContracts(
      aztecRpcServer,
      walletClient,
      publicClient,
      deployL1ContractsValues!.registryAddress,
      initialBalance,
      pointToPublicKey(ownerPub),
    );
    const l2Contract = contracts.l2Contract;
    const underlyingERC20 = contracts.underlyingERC20;
    const tokenPortal = contracts.tokenPortal;
    const tokenPortalAddress = contracts.tokenPortalAddress;
    // await expectBalance(accounts[0], initialBalance);
    logger('Successfully deployed contracts and initialized portal');

    return new CrossChainTestHarness(
      aztecNode,
      aztecRpcServer,
      accounts,
      logger,
      l2Contract,
      ethAccount,
      tokenPortalAddress,
      tokenPortal,
      underlyingERC20,
      outbox,
      publicClient,
      walletClient,
      ownerAddress,
      receiver,
      ownerPub,
    );
  }
  constructor(
    /** AztecNode. */
    public aztecNode: AztecNodeService,
    /** AztecRpcServer. */
    public aztecRpcServer: AztecRPCServer,
    /** Accounts. */
    public accounts: AztecAddress[],
    /** Logger. */
    public logger: DebugLogger,

    /** Testing aztec contract. */
    public l2Contract: Contract,
    /** Eth account to interact with. */
    public ethAccount: EthAddress,

    /** Portal address. */
    public tokenPortalAddress: EthAddress,
    /** Token portal instance. */
    public tokenPortal: any,
    /** Underlying token for portal tests. */
    public underlyingERC20: any,
    /** Message Bridge Outbox. */
    public outbox: any,
    /** Viem Public client instance. */
    public publicClient: PublicClient<HttpTransport, Chain>,
    /** Viem Walllet Client instance. */
    public walletClient: any,

    /** Aztec address to use in tests. */
    public ownerAddress: AztecAddress,
    /** Another Aztec Address to use in tests. */
    public receiver: AztecAddress,
    /** The owners public key. */
    public ownerPub: Point,
  ) {}

  async generateClaimSecret(): Promise<[Fr, Fr]> {
    this.logger("Generating a claim secret using pedersen's hash function");
    const secret = Fr.random();
    const secretHash = await computeMessageSecretHash(secret);
    this.logger('Generated claim secret: ', secretHash.toString(true));
    return [secret, secretHash];
  }

  async mintTokensOnL1(amount: bigint) {
    this.logger('Minting tokens on L1');
    await this.underlyingERC20.write.mint([this.ethAccount.toString(), amount], {} as any);
    expect(await this.underlyingERC20.read.balanceOf([this.ethAccount.toString()])).toBe(amount);
  }

  async sendTokensToPortal(bridgeAmount: bigint, secretHash: Fr) {
    await this.underlyingERC20.write.approve([this.tokenPortalAddress.toString(), bridgeAmount], {} as any);

    // Deposit tokens to the TokenPortal
    const deadline = 2 ** 32 - 1; // max uint32 - 1

    this.logger('Sending messages to L1 portal to be consumed privately');
    const args = [
      this.ownerAddress.toString(),
      bridgeAmount,
      deadline,
      secretHash.toString(true),
      this.ethAccount.toString(),
    ] as const;
    const { result: messageKeyHex } = await this.tokenPortal.simulate.depositToAztec(args, {
      account: this.ethAccount.toString(),
    } as any);
    await this.tokenPortal.write.depositToAztec(args, {} as any);

    return Fr.fromString(messageKeyHex);
  }

  async performL2Transfer(transferAmount: bigint) {
    // send a transfer tx to force through rollup with the message included
    const transferTx = this.l2Contract.methods
      .transfer(
        transferAmount,
        pointToPublicKey(await this.aztecRpcServer.getAccountPublicKey(this.ownerAddress)),
        pointToPublicKey(await this.aztecRpcServer.getAccountPublicKey(this.receiver)),
      )
      .send({ from: this.accounts[0] });

    await transferTx.isMined(0, 0.1);
    const transferReceipt = await transferTx.getReceipt();

    expect(transferReceipt.status).toBe(TxStatus.MINED);
  }

  async checkEntryIsNotInOutbox(withdrawAmount: bigint): Promise<Fr> {
    this.logger('Ensure that the entry is not in outbox yet');
    const contractInfo = await this.aztecNode.getContractInfo(this.l2Contract.address);
    // 0x00f714ce, selector for "withdraw(uint256,address)"
    const content = sha256ToField(
      Buffer.concat([
        Buffer.from([0x00, 0xf7, 0x14, 0xce]),
        toBufferBE(withdrawAmount, 32),
        this.ethAccount.toBuffer32(),
      ]),
    );
    const entryKey = sha256ToField(
      Buffer.concat([
        this.l2Contract.address.toBuffer(),
        new Fr(1).toBuffer(), // aztec version
        contractInfo?.portalContractAddress.toBuffer32() ?? Buffer.alloc(32, 0),
        new Fr(this.publicClient.chain.id).toBuffer(), // chain id
        content.toBuffer(),
      ]),
    );
    expect(await this.outbox.read.contains([entryKey.toString(true)])).toBeFalsy();

    return entryKey;
  }

  async withdrawFundsFromBridgeOnL1(withdrawAmount: bigint, entryKey: Fr) {
    this.logger('Send L1 tx to consume entry and withdraw funds');
    // Call function on L1 contract to consume the message
    const { request: withdrawRequest, result: withdrawEntryKey } = await this.tokenPortal.simulate.withdraw([
      withdrawAmount,
      this.ethAccount.toString(),
    ]);

    expect(withdrawEntryKey).toBe(entryKey.toString(true));
    expect(await this.outbox.read.contains([withdrawEntryKey])).toBeTruthy();

    await this.walletClient.writeContract(withdrawRequest);
    return withdrawEntryKey;
  }

  async stop() {
    await this.aztecNode?.stop();
    await this.aztecRpcServer?.stop();
  }
}
