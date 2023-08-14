import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { AztecAddress } from '@aztec/aztec.js';
import { EthAddress } from '@aztec/foundation/eth-address';
import { DebugLogger } from '@aztec/foundation/log';
import { NonNativeTokenContract } from '@aztec/noir-contracts/types';
import { AztecRPC, TxStatus } from '@aztec/types';

import { CrossChainTestHarness } from './fixtures/cross_chain_test_harness.js';
import { delay, setup } from './fixtures/utils.js';

describe('e2e_cross_chain_messaging', () => {
  let aztecNode: AztecNodeService;
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
      aztecNode,
      aztecRpcServer: aztecRpcServer_,
      deployL1ContractsValues,
      accounts,
      wallet,
      logger: logger_,
      cheatCodes,
    } = await setup(2);
    crossChainTestHarness = await CrossChainTestHarness.new(
      initialBalance,
      aztecNode,
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

  const expectBalance = async (owner: AztecAddress, expectedBalance: bigint) => {
    const balance = await l2Contract.methods.getBalance(owner).view({ from: owner });
    logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const withdrawFundsFromAztec = async (withdrawAmount: bigint) => {
    logger('Send L2 tx to withdraw funds');
    const withdrawTx = l2Contract.methods
      .withdraw(withdrawAmount, ownerAddress, ethAccount, EthAddress.ZERO.toField())
      .send({ origin: ownerAddress });

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

    await crossChainTestHarness.consumeMessageOnAztecAndMintSecretly(bridgeAmount, messageKey, secret);
    await expectBalance(ownerAddress, bridgeAmount + initialBalance - transferAmount);

    // time to withdraw the funds again!
    logger('Withdrawing funds from L2');
    const withdrawAmount = 9n;
    const entryKey = await crossChainTestHarness.checkEntryIsNotInOutbox(withdrawAmount);
    await withdrawFundsFromAztec(withdrawAmount);
    await expectBalance(ownerAddress, bridgeAmount + initialBalance - transferAmount - withdrawAmount);

    // Check balance before and after exit.
    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(l1TokenBalance - bridgeAmount);
    await crossChainTestHarness.withdrawFundsFromBridgeOnL1(withdrawAmount, entryKey);
    expect(await underlyingERC20.read.balanceOf([ethAccount.toString()])).toBe(
      l1TokenBalance - bridgeAmount + withdrawAmount,
    );

    expect(await outbox.read.contains([entryKey.toString(true)])).toBeFalsy();
  }, 120_000);
});
