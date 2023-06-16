import { AztecNodeService } from '@aztec/aztec-node';
import { AztecAddress, AztecRPCServer, Contract, TxStatus } from '@aztec/aztec.js';
import { EthAddress } from '@aztec/foundation/eth-address';

import { Fr } from '@aztec/foundation/fields';
import { DebugLogger } from '@aztec/foundation/log';
import { delay, expectStorageSlot, setup } from './utils.js';
import { CrossChainTestHarness } from './cross_chain/test_harness.js';

describe('e2e_public_cross_chain_messaging', () => {
  let aztecNode: AztecNodeService;
  let aztecRpcServer: AztecRPCServer;
  let logger: DebugLogger;

  let l2Contract: Contract;
  let ethAccount: EthAddress;

  let underlyingERC20: any;
  let outbox: any;

  const initialBalance = 10n;
  let ownerAddress: AztecAddress;

  let crossChainTestHarness: CrossChainTestHarness;

  beforeEach(async () => {
    const {
      aztecNode: aztecNode_,
      aztecRpcServer: aztecRpcServer_,
      deployL1ContractsValues,
      accounts,
      logger: logger_,
    } = await setup(2);
    crossChainTestHarness = await CrossChainTestHarness.new(
      initialBalance,
      aztecNode_,
      aztecRpcServer_,
      deployL1ContractsValues,
      accounts,
      logger_,
    );

    l2Contract = crossChainTestHarness.l2Contract;
    ethAccount = crossChainTestHarness.ethAccount;
    ownerAddress = crossChainTestHarness.ownerAddress;
    underlyingERC20 = crossChainTestHarness.underlyingERC20;
    outbox = crossChainTestHarness.outbox;
    aztecRpcServer = crossChainTestHarness.aztecRpcServer;
    aztecNode = aztecNode_;

    logger = logger_;
    logger('Successfully deployed contracts and initialized portal');
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    await aztecRpcServer?.stop();
    await crossChainTestHarness?.stop();
  });

  const consumeMessageOnAztec = async (bridgeAmount: bigint, messageKey: Fr, secret: Fr) => {
    logger('Consuming messages on L2 Publicly');
    // Call the mint tokens function on the noir contract
    const consumptionTx = l2Contract.methods
      .mintPublic(bridgeAmount, ownerAddress, messageKey, secret, ethAccount.toField())
      .send({ from: ownerAddress });

    await consumptionTx.isMined(0, 0.1);
    const consumptionReceipt = await consumptionTx.getReceipt();

    expect(consumptionReceipt.status).toBe(TxStatus.MINED);
  };

  const withdrawFundsFromAztec = async (withdrawAmount: bigint) => {
    logger('Send L2 tx to withdraw funds');
    const withdrawTx = l2Contract.methods
      .withdrawPublic(withdrawAmount, ethAccount.toField())
      .send({ from: ownerAddress });

    await withdrawTx.isMined(0, 0.1);
    const withdrawReceipt = await withdrawTx.getReceipt();

    expect(withdrawReceipt.status).toBe(TxStatus.MINED);
  };

  it('Milestone 2: Deposit funds from L1 -> L2 and withdraw back to L1', async () => {
    // Generate a claim secret using pedersen
    const l1TokenBalance = 1000000n;
    const bridgeAmount = 100n;
    const publicBalanceSlot = 2n;

    const [secret, secretHash] = await crossChainTestHarness.generateClaimSecret();

    await crossChainTestHarness.mintTokensOnL1(l1TokenBalance);
    const messageKey = await crossChainTestHarness.sendTokensToPortal(bridgeAmount, secretHash);
    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(l1TokenBalance - bridgeAmount);

    // Wait for the archiver to process the message
    await delay(5000); /// waiting 5 seconds.

    // Perform another unrelated transaction on L2 to progress the rollup.
    const transferAmount = 1n;
    await crossChainTestHarness.performL2Transfer(transferAmount);

    await consumeMessageOnAztec(bridgeAmount, messageKey, secret);
    await expectStorageSlot(logger, aztecNode, l2Contract, publicBalanceSlot, ownerAddress.toField(), bridgeAmount);

    // time to withdraw the funds again!
    logger('Withdrawing funds from L2');
    const withdrawAmount = 9n;
    const entryKey = await crossChainTestHarness.checkEntryIsNotInOutbox(withdrawAmount);
    await withdrawFundsFromAztec(withdrawAmount);
    await expectStorageSlot(
      logger,
      aztecNode,
      l2Contract,
      publicBalanceSlot,
      ownerAddress.toField(),
      bridgeAmount - withdrawAmount,
    );

    // Check balance before and after exit.
    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(l1TokenBalance - bridgeAmount);
    await crossChainTestHarness.withdrawFundsFromBridgeOnL1(withdrawAmount, entryKey);
    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(
      l1TokenBalance - bridgeAmount + withdrawAmount,
    );

    expect(await outbox.read.contains([entryKey.toString(true)])).toBeFalsy();
  }, 120_000);
});
