import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { AztecAddress } from '@aztec/aztec.js';
import { EthAddress } from '@aztec/foundation/eth-address';
import { DebugLogger } from '@aztec/foundation/log';
import { NonNativeTokenContract } from '@aztec/noir-contracts/types';
import { AztecRPC, TxStatus } from '@aztec/types';

import { CrossChainTestHarness } from './fixtures/cross_chain_test_harness.js';
import { delay, setup } from './fixtures/utils.js';

describe('e2e_public_cross_chain_messaging', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  let logger: DebugLogger;

  let l2Contract: NonNativeTokenContract;
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
      wallet,
      logger: logger_,
      cheatCodes,
    } = await setup(2);
    crossChainTestHarness = await CrossChainTestHarness.new(
      initialBalance,
      aztecNode_,
      aztecRpcServer_,
      deployL1ContractsValues,
      accounts,
      wallet,
      logger_,
      cheatCodes,
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
    if (aztecRpcServer instanceof AztecRPCServer) {
      await aztecRpcServer?.stop();
    }
    await crossChainTestHarness?.stop();
  });

  const withdrawFundsFromAztec = async (withdrawAmount: bigint) => {
    logger('Send L2 tx to withdraw funds');
    const withdrawTx = l2Contract.methods
      .withdrawPublic(withdrawAmount, ethAccount.toField(), EthAddress.ZERO.toField())
      .send();

    await withdrawTx.isMined({ interval: 0.1 });
    const withdrawReceipt = await withdrawTx.getReceipt();

    expect(withdrawReceipt.status).toBe(TxStatus.MINED);
  };

  it('Milestone 2: Deposit funds from L1 -> L2 and withdraw back to L1', async () => {
    // Generate a claim secret using pedersen
    const l1TokenBalance = 1000000n;
    const bridgeAmount = 100n;

    const [secret, secretHash] = await crossChainTestHarness.generateClaimSecret();

    await crossChainTestHarness.mintTokensOnL1(l1TokenBalance);
    const messageKey = await crossChainTestHarness.sendTokensToPortal(bridgeAmount, secretHash);
    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(l1TokenBalance - bridgeAmount);

    // Wait for the archiver to process the message
    await delay(5000); /// waiting 5 seconds.

    // Perform another unrelated transaction on L2 to progress the rollup.
    const transferAmount = 1n;
    await crossChainTestHarness.performL2Transfer(transferAmount);

    await crossChainTestHarness.consumeMessageOnAztecAndMintPublicly(bridgeAmount, messageKey, secret);
    await crossChainTestHarness.expectPublicBalanceOnL2(ownerAddress, bridgeAmount);

    // time to withdraw the funds again!
    logger('Withdrawing funds from L2');
    const withdrawAmount = 9n;
    const entryKey = await crossChainTestHarness.checkEntryIsNotInOutbox(withdrawAmount);
    await withdrawFundsFromAztec(withdrawAmount);
    await crossChainTestHarness.expectPublicBalanceOnL2(ownerAddress, bridgeAmount - withdrawAmount);

    // Check balance before and after exit.
    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(l1TokenBalance - bridgeAmount);
    await crossChainTestHarness.withdrawFundsFromBridgeOnL1(withdrawAmount, entryKey);
    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(
      l1TokenBalance - bridgeAmount + withdrawAmount,
    );

    expect(await outbox.read.contains([entryKey.toString(true)])).toBeFalsy();
  }, 120_000);
});
